from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest

from world._native import Habitat
from world.archive import SubjectArchive, read_subject
from world.ecosystem import EcosystemRuntime, write_json
from world.experiments.adaptation import fund
from world.experiments.ecosystem import ecosystem_preset, ablation
from world.experiments.evaluation import evaluate_subject
from world.taxonomy import classify


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/circle_rules.json").read_text())


def fixture(kinds=("grazer",), positions=None):
    rules, config = ecosystem_preset(RULES, terrain=False, revised=False)
    rules.update(
        world_width=16.0,
        world_height=16.0,
        grid_width=16,
        grid_height=16,
        minimum_reproduction_age=0.01,
    )
    config["producers"]["count"] = 0
    groups = {g["genome"]["body"]["diet"]: g for g in config["populations"]}
    config["populations"] = []
    for i, kind in enumerate(kinds):
        group = deepcopy(groups[kind])
        group.update(
            count=1,
            reserve_energy=32.0,
            reserve_nutrient=6.0,
            positions=[positions[i] if positions else {"x": 4.0 + i * 3, "y": 8.0}],
        )
        group["genome"]["body"]["circle"]["adaptations"]["anchored"] = False
        config["populations"].append(group)
    config["ecology"].update(reproduction=False, decomposition=False)
    config["evolution"]["mutation"].update(
        parameter_edits=0, body_edits=0, body_structure_rate=0, neural_structure_rate=0
    )
    fund(rules, config)
    return rules, config


def actions(world, values):
    frame = json.loads(world.agent_frame_json())
    return {
        "schema_version": 1,
        "tick": frame["tick"],
        "agents": [
            {
                "id": a["id"],
                "version": a["version"],
                "values": [
                    {
                        "port": p["port"],
                        "value": values.get((a["id"], p["channel"]), 0.0)
                        if p["active"]
                        else 0.0,
                    }
                    for p in a["outputs"]
                ],
            }
            for a in frame["agents"]
        ],
    }


def advance(world, values):
    world.advance_controlled(json.dumps(actions(world, values)))


def state(world):
    return json.loads(world.checkpoint_json())


class EcologyExtensionsTests(unittest.TestCase):
    def test_reserved_filament_material_does_not_become_decomposition_food(self):
        for seed in [73, 89, 107]:
            rules, config = ecosystem_preset(RULES, seed)
            runtime = EcosystemRuntime.create(rules, config)
            runtime.advance(60)
            self.assertAlmostEqual(runtime.audit()["nutrient_error"], 0.0, places=8)

    def test_experiment_rules_hold_even_when_genes_request_disabled_capabilities(self):
        rules, config = fixture()
        evolution = config["evolution"]
        evolution.update(
            learned_weight_updates=False,
            neural_development=False,
            allow_communication=False,
            allow_filaments=False,
            development_interval_ticks=6,
        )
        a = config["populations"][0]["genome"]["body"]["circle"]["adaptations"]
        a.update(communication=1.0, filament=1.0, neural_growth=1.0)
        world = Habitat(json.dumps(rules), json.dumps(config))
        frame = json.loads(world.agent_frame_json())
        self.assertEqual(frame["agents"][0]["learning_rate"], 0.0)
        advance(world, {(1, 104): 1.0, (1, 106): 1.0, (1, 108): 1.0})
        saved = state(world)
        self.assertEqual(saved["interactions"]["filaments"], [])
        self.assertEqual(saved["interactions"]["signals"], [])
        self.assertNotIn("expressed_brain", saved["ledger"]["organisms"][0])

    def test_topology_ablation_disables_birth_and_lifetime_structural_changes(self):
        _, config = ecosystem_preset(RULES)
        control = ablation(config, "fixed_topology")
        self.assertFalse(control["evolution"]["neural_development"])
        self.assertEqual(control["evolution"]["mutation"]["neural_structure_rate"], 0.0)

    def test_agency_can_withhold_growth_and_reproduction(self):
        rules, config = fixture()
        config["populations"][0]["tissue_nutrient"] = 4.0
        config["populations"][0].update(reserve_energy=19.0, reserve_nutrient=3.0)
        fund(rules, config)
        off = Habitat(json.dumps(rules), json.dumps(config))
        on = off.fork()
        advance(off, {})
        advance(on, {(1, 100): 1.0})
        self.assertEqual(
            state(off)["ledger"]["organisms"][0]["tissue"]["nutrient"], 4.0
        )
        self.assertGreater(
            state(on)["ledger"]["organisms"][0]["tissue"]["nutrient"], 4.0
        )
        rules, config = fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"]["online_learning"] = False
        off = Habitat(json.dumps(rules), json.dumps(config))
        on = off.fork()
        advance(off, {})
        advance(on, {(1, 103): 1.0})
        self.assertEqual(json.loads(off.summary_json())["ecology"]["births"], 0)
        self.assertEqual(json.loads(on.summary_json())["ecology"]["births"], 1)

    def test_neural_development_preserves_birth_dna_and_controller_state(self):
        rules, config = fixture()
        config["evolution"]["development_interval_ticks"] = 6
        genome = config["populations"][0]["genome"]
        genome["body"]["circle"]["learning_rate"] = 0.0
        for node in genome["brain"]["nodes"]:
            if node["port"] == 360:
                node["bias"] = 2.0
            if node["port"] == 359:
                node["bias"] = -2.0
        runtime = EcosystemRuntime.create(rules, config)
        before = runtime.snapshot()
        weights = runtime.brains.weights.clone()
        runtime.advance()
        after = runtime.snapshot()
        o = after["world"]["ledger"]["organisms"][0]
        self.assertEqual(
            o["genome"], before["world"]["ledger"]["organisms"][0]["genome"]
        )
        self.assertEqual(
            len(o["expressed_brain"]["nodes"]), len(o["genome"]["brain"]["nodes"]) + 1
        )
        self.assertTrue((runtime.brains.weights[: len(weights)] == weights).all())
        restored = EcosystemRuntime.from_snapshot(after)
        runtime.advance(3)
        restored.advance(3)
        self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_predation_creates_measured_food_flow(self):
        rules, config = fixture(
            ("predator", "grazer"), [{"x": 8, "y": 8}, {"x": 8.86, "y": 8}]
        )
        config["populations"][0]["reserve_energy"] = 20.0
        config["populations"][0]["reserve_nutrient"] = 1.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0, (1, 121): 0.5})
        web = state(world)["interactions"]["food_web"]
        self.assertGreater(web["flows"]["grazer->predator"], 0)
        self.assertAlmostEqual(web["trophic"]["1"], 3.0)
        self.assertGreater(
            json.loads(world.summary_json())["diagnostics"]["attack_hits"], 0
        )

    def test_expressed_structure_inheritance_is_explicit_and_newborn_memory_resets(
        self,
    ):
        rules, config = fixture()
        config["evolution"]["development_interval_ticks"] = 6
        genome = config["populations"][0]["genome"]
        genome["body"]["circle"]["adaptations"]["inherit_structure"] = True
        for node in genome["brain"]["nodes"]:
            if node["port"] in {359, 360}:
                node["bias"] = 2.0
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance()
        payload = runtime.snapshot()
        inherited_count = len(
            payload["world"]["ledger"]["organisms"][0]["expressed_brain"]["nodes"]
        )
        payload["world"]["config"]["ecology"]["reproduction"] = True
        runtime = EcosystemRuntime.from_snapshot(payload)
        runtime.advance()
        child = next(
            o
            for o in runtime.snapshot()["world"]["ledger"]["organisms"]
            if o["parent_id"] == 1
        )
        self.assertEqual(len(child["genome"]["brain"]["nodes"]), inherited_count)
        self.assertTrue(
            all(
                runtime.brains.memory[index].item() == 0
                for (owner, _), index in runtime.brains.index.items()
                if owner == child["id"]
            )
        )

    def test_pruning_removes_only_expressed_hidden_structure(self):
        rules, config = fixture()
        config["evolution"]["development_interval_ticks"] = 6
        world = Habitat(json.dumps(rules), json.dumps(config))
        before = state(world)["ledger"]["organisms"][0]["genome"]
        advance(world, {(1, 105): 1.0})
        after = state(world)["ledger"]["organisms"][0]
        self.assertEqual(after["genome"], before)
        self.assertEqual(
            len(after["expressed_brain"]["nodes"]), len(before["brain"]["nodes"]) - 1
        )

    def test_filaments_spend_resources_capture_decay_and_resume(self):
        rules, config = fixture(
            ("predator", "grazer"), [{"x": 8, "y": 8}, {"x": 9.2, "y": 8}]
        )
        config["populations"][0]["genome"]["body"]["circle"]["adaptations"][
            "filament"
        ] = 1.0
        world = Habitat(json.dumps(rules), json.dumps(config))
        checkpoint = state(world)
        checkpoint["motion"][0]["heading"] = 0.0
        world = Habitat.from_checkpoint_json(json.dumps(checkpoint))
        advance(world, {(1, 106): 1.0})
        payload = state(world)
        self.assertEqual(len(payload["interactions"]["filaments"]), 1)
        self.assertEqual(payload["interactions"]["filaments"][0]["captured"], 2)
        self.assertLess(payload["ledger"]["organisms"][0]["reserves"]["nutrient"], 6.0)
        restored = Habitat.from_checkpoint_json(json.dumps(payload))
        advance(world, {(1, 107): 1.0})
        advance(restored, {(1, 107): 1.0})
        self.assertEqual(state(world), state(restored))
        self.assertEqual(state(world)["interactions"]["filaments"], [])
        self.assertAlmostEqual(
            json.loads(world.audit_json())["nutrient_error"], 0.0, places=8
        )

    def test_local_signals_are_paid_and_cost_matched_control_hides_reception(self):
        rules, config = fixture(
            ("grazer", "grazer"), [{"x": 6, "y": 8}, {"x": 8, "y": 8}]
        )
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 108): 1.0})
        row = json.loads(world.agent_frame_json())["agents"][1]
        self.assertGreater(
            next(v["value"] for v in row["inputs"] if v["port"] % 256 == 24), 0
        )
        payload = state(world)
        payload["config"]["evolution"]["receive_signals"] = False
        control = Habitat.from_checkpoint_json(json.dumps(payload))
        row = json.loads(control.agent_frame_json())["agents"][1]
        self.assertEqual(
            next(v["value"] for v in row["inputs"] if v["port"] % 256 == 24), 0
        )
        self.assertGreater(
            json.loads(world.summary_json())["diagnostics"]["costs"]["communication"], 0
        )

    def test_camouflage_and_rock_occlusion_change_sensing(self):
        rules, config = fixture(
            ("predator", "grazer"), [{"x": 4, "y": 8}, {"x": 8, "y": 8}]
        )
        config["evolution"]["terrain"] = [
            {
                "kind": "vegetation",
                "center": {"x": 8, "y": 8},
                "radius": 2.0,
                "density": 1.0,
            }
        ]
        target = config["populations"][1]["genome"]["body"]["circle"]["adaptations"]
        target.update(camouflage=1.0, pigment=0.55)
        hidden = Habitat(json.dumps(rules), json.dumps(config))
        target["camouflage"] = 0.0
        visible = Habitat(json.dumps(rules), json.dumps(config))

        def food(world):
            return next(
                v["value"]
                for v in json.loads(world.agent_frame_json())["agents"][0]["inputs"]
                if v["port"] % 256 == 12
            )

        self.assertEqual(food(hidden), 0.0)
        self.assertGreater(food(visible), 0.0)
        config["evolution"]["terrain"] = [
            {"kind": "rock", "center": {"x": 6, "y": 8}, "radius": 0.6, "density": 1.0}
        ]
        self.assertEqual(food(Habitat(json.dumps(rules), json.dumps(config))), 0.0)

    def test_scavenger_and_recycler_consume_existing_detritus(self):
        for kind in ["scavenger", "recycler"]:
            rules, config = fixture((kind,))
            config["populations"][0].update(reserve_energy=20.0, reserve_nutrient=1.0)
            fund(rules, config)
            world = Habitat(json.dumps(rules), json.dumps(config))
            payload = state(world)
            o = payload["ledger"]["organisms"][0]
            amount = 0.5
            energy = amount * rules["tissue_energy_per_nutrient"]
            o["reserves"]["nutrient"] -= amount
            o["reserves"]["energy"] -= energy
            cell = 8 * 16 + 4
            payload["grid"]["cells"][cell]["detritus"] = {
                "nutrient": amount,
                "energy": energy,
            }
            payload["ledger"]["pools"]["detritus"] = {
                "nutrient": amount,
                "energy": energy,
            }
            world = Habitat.from_checkpoint_json(json.dumps(payload))
            sensory = json.loads(world.agent_frame_json())["agents"][0]["inputs"]
            self.assertGreater(
                next(v["value"] for v in sensory if v["port"] % 256 == 12), 0.0
            )
            advance(world, {(1, 120): 1.0})
            self.assertGreater(
                state(world)["interactions"]["food_web"]["flows"][f"detritus->{kind}"],
                0,
            )

    def test_archives_export_learned_parameters_and_detect_tampering(self):
        runtime = EcosystemRuntime.create(*fixture())
        runtime.advance(3)
        with tempfile.TemporaryDirectory() as folder:
            archive = SubjectArchive(folder, limit=1)
            path = archive.capture(runtime, 1)
            subject = read_subject(path)
            self.assertEqual(
                [e["weight"] for e in subject["expressed_graph"]["connections"]],
                runtime.brains.weights.tolist(),
            )
            result = evaluate_subject(
                path,
                {
                    "schema_version": 1,
                    "seeds": [1301],
                    "tasks": ["memory", "foraging"],
                    "decisions": 2,
                },
            )
            self.assertEqual(len(result["results"]), 3)
            self.assertTrue(result["memory"]["available"])
            subject["reason"] = "changed"
            write_json(path, subject)
            with self.assertRaises(ValueError):
                read_subject(path)

    def test_death_archive_keeps_final_parameters_before_active_state_is_removed(self):
        rules, config = fixture()
        config["populations"][0].update(reserve_energy=0.0, reserve_nutrient=0.0)
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        expected = runtime.brains.weights.tolist()
        with tempfile.TemporaryDirectory() as folder:
            runtime.archive = SubjectArchive(folder)
            runtime.advance()
            self.assertEqual(len(runtime.brains.keys), 0)
            self.assertEqual(len(runtime.archive.paths), 1)
            subject = read_subject(runtime.archive.paths[0])
            self.assertEqual(
                [e["weight"] for e in subject["expressed_graph"]["connections"]],
                expected,
            )
            self.assertFalse(subject["origin"]["organism"]["alive"])

    def test_immutable_graph_cache_and_descriptive_clusters(self):
        runtime = EcosystemRuntime.create(*fixture())
        with self.assertRaises(TypeError):
            runtime._frame["agents"][0]["brain"]["nodes"][0]["bias"] = 1
        public = runtime.frame()
        public["agents"][0]["brain"]["nodes"][0]["bias"] = 1
        with self.assertRaises(ValueError):
            runtime.brains.validate(public)
        self.assertEqual(
            classify(runtime.snapshot()["world"]["ledger"]["organisms"])[
                "cluster_count"
            ],
            1,
        )


if __name__ == "__main__":
    unittest.main()
