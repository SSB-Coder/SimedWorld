from copy import deepcopy
import json
import math
import unittest
import tempfile
from pathlib import Path

from world._native import Habitat
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.experiments.ecosystem import ecosystem_preset, ablation
from world.presets import neural_blueprint

from test_ecology_extensions import RULES, fixture, advance, state


class SurvivalBehaviourTests(unittest.TestCase):
    def test_liquid_observer_uses_display_stream_after_creation_and_restore(self):
        from world.control import RunControl

        runtime = EcosystemRuntime.create(*fixture(), controller="ltc")
        runtime.advance(2)
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            control = RunControl(
                runtime,
                root / "checkpoint.json",
                stream=root / "observer.json",
                paused=True,
            )
            control.selected = 1
            runtime.save(control.checkpoint)
            control.publish()
            frame = json.loads(control.stream.read_text())
            self.assertEqual(frame["kind"], "world.display")
            self.assertEqual(frame["state"]["brain"]["id"], 1)
            control.apply(
                {
                    "schema_version": 1,
                    "session": control.session,
                    "id": "restore-liquid",
                    "kind": "load",
                    "path": "",
                }
            )
            control.publish()
            self.assertEqual(control.runtime.brains.kind, "plastic_sparse_ltc_v3")
            self.assertEqual(
                json.loads(control.stream.read_text())["kind"], "world.display"
            )

    def test_collective_defense_accumulates_real_individual_damage(self):
        rules, config = fixture(
            ("grazer", "grazer", "grazer", "predator"),
            [
                {"x": 6.75, "y": 8},
                {"x": 9.25, "y": 8},
                {"x": 8, "y": 6.75},
                {"x": 8, "y": 8},
            ],
        )
        config["evolution"]["embodied_ecology"] = True
        for group in config["populations"]:
            group["genome"]["body"]["circle"]["adaptations"]["offense"] = 0.7
        config["populations"][-1].update(tissue_nutrient=24.2, reserve_energy=100.0)
        config["populations"][-1]["genome"]["body"]["modules"][0]["radius"] = 0.8
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(240):
            advance(
                world,
                {
                    (identity, channel): 1.0
                    for identity in [1, 2, 3]
                    for channel in [111, 120, 121]
                },
            )
        saved = state(world)
        self.assertFalse(saved["ledger"]["organisms"][-1]["alive"])
        contributors = {
            e["actor"]
            for e in saved["diagnostics"]["events"]
            if e["kind"] == "damage" and e["target"] == 4
        }
        self.assertGreaterEqual(len(contributors), 2)
        self.assertNotIn("detritus->grazer", saved["interactions"]["food_web"]["flows"])
        self.assertAlmostEqual(
            json.loads(world.audit_json())["energy_error"], 0, places=8
        )

    def test_herbivore_can_kill_predator_without_eating_it(self):
        rules, config = fixture(
            ("grazer", "predator"), [{"x": 8, "y": 8}, {"x": 9.06, "y": 8}]
        )
        config["evolution"]["embodied_ecology"] = True
        herb, predator = config["populations"]
        for group, tissue, radius, energy, offense in [
            (herb, 24.2, 0.8, 90, 1.0),
            (predator, 2.4, 0.25, 8, 0.1),
        ]:
            group.update(
                tissue_nutrient=tissue, reserve_energy=energy, reserve_nutrient=1.0
            )
            group["genome"]["body"]["modules"][0]["radius"] = radius
            group["genome"]["body"]["circle"]["adaptations"]["offense"] = offense
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(48):
            advance(world, {(1, 121): 1.0, (1, 120): 1.0, (1, 111): 1.0})
        saved = state(world)
        organisms = saved["ledger"]["organisms"]
        self.assertTrue(organisms[0]["alive"])
        self.assertFalse(organisms[1]["alive"])
        self.assertEqual(organisms[1]["death_cause"], "damage")
        self.assertLessEqual(organisms[0]["reserves"]["nutrient"], 1.0)
        self.assertNotIn("detritus->grazer", saved["interactions"]["food_web"]["flows"])
        self.assertGreater(
            sum(c["detritus"]["nutrient"] for c in saved["grid"]["cells"]), 0
        )
        self.assertAlmostEqual(
            json.loads(world.audit_json())["nutrient_error"], 0, places=8
        )

    def test_social_information_does_not_force_movement(self):
        rules, config = fixture(
            ("grazer", "grazer"), [{"x": 8, "y": 8}, {"x": 9.1, "y": 8}]
        )
        for group in config["populations"]:
            body = group["genome"]["body"]
            body["circle"]["adaptations"]["social_senses"] = True
            body["circle"]["adaptations"]["sociality"] = 1.0
            group["genome"]["brain"] = neural_blueprint(body)
        world = Habitat(json.dumps(rules), json.dumps(config))
        frame = json.loads(world.agent_frame_json())
        inputs = {i["port"] % 256: i["value"] for i in frame["agents"][0]["inputs"]}
        self.assertGreater(inputs[32], 0)
        self.assertGreater(math.hypot(inputs[30], inputs[31]), 0)
        before = state(world)["positions"]
        advance(world, {})
        self.assertEqual(before, state(world)["positions"])
        config["evolution"]["social_observations"] = False
        off = Habitat(json.dumps(rules), json.dumps(config))
        inputs = {
            i["port"] % 256: i["value"]
            for i in json.loads(off.agent_frame_json())["agents"][0]["inputs"]
        }
        self.assertTrue(all(inputs[c] == 0 for c in range(30, 40)))

    def test_new_neurons_require_material_and_preserve_existing_parameters(self):
        rules, config = fixture()
        config["evolution"].update(embodied_ecology=True, development_interval_ticks=6)
        config["populations"][0]["reserve_nutrient"] = 0.0
        fund(rules, config)
        off = Habitat(json.dumps(rules), json.dumps(config))
        advance(off, {(1, 104): 1.0})
        self.assertNotIn("expressed_brain", state(off)["ledger"]["organisms"][0])
        config["populations"][0]["reserve_nutrient"] = 1.0
        fund(rules, config)
        on = Habitat(json.dumps(rules), json.dumps(config))
        before = state(on)["ledger"]["organisms"][0]
        advance(on, {(1, 104): 1.0})
        after = state(on)["ledger"]["organisms"][0]
        self.assertGreater(after["tissue"]["nutrient"], before["tissue"]["nutrient"])
        self.assertEqual(
            after["expressed_brain"]["connections"][:-2],
            before["genome"]["brain"]["connections"],
        )
        self.assertEqual(after["expressed_brain"]["connections"][-1]["weight"], 0.0)
        self.assertAlmostEqual(
            json.loads(on.audit_json())["nutrient_error"], 0, places=8
        )

    def test_capture_scales_with_actual_photosynthetic_size(self):
        rules, config = ecosystem_preset(RULES, terrain=False)
        config["cell_overrides"] = []
        config["populations"] = []
        config["producers"].update(
            count=1,
            reserve_energy=1.0,
            reserve_nutrient=0.0,
            positions=[{"x": 8, "y": 8}],
        )
        config["ecology"]["reproduction"] = False
        capture = []
        for tissue in [2.0, 7.0]:
            config["producers"]["tissue_nutrient"] = tissue
            fund(rules, config)
            world = Habitat(json.dumps(rules), json.dumps(config))
            advance(world, {})
            capture.append(state(world)["last_update"]["light_captured"])
        self.assertGreater(capture[1], capture[0] * 2)

    def test_initial_organic_material_is_funded(self):
        rules, config = ecosystem_preset(RULES)
        runtime = EcosystemRuntime.create(rules, config)
        self.assertGreater(
            runtime.telemetry()["resources"]["free_detritus_nutrient"], 0
        )
        self.assertAlmostEqual(runtime.audit()["nutrient_error"], 0, places=8)
        bad = deepcopy(config)
        bad["initial_energy"] -= 1
        with self.assertRaises((ValueError, RuntimeError)):
            EcosystemRuntime.create(rules, bad)

    def test_liquid_learning_topology_and_exact_continuation(self):
        rules, config = fixture()
        runtime = EcosystemRuntime.create(rules, config, controller="ltc")
        runtime.advance(4)
        self.assertGreater(runtime.brains.updates[1], 0)
        self.assertTrue(runtime.brains.memory.abs().max() <= 1)
        restored = EcosystemRuntime.from_snapshot(runtime.snapshot())
        self.assertEqual(restored.brains.kind, "plastic_sparse_ltc_v3")
        runtime.advance(4)
        restored.advance(4)
        self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_liquid_new_zero_influence_connections_can_start_learning(self):
        rules, config = fixture()
        config["evolution"].update(embodied_ecology=True, development_interval_ticks=6)
        config["populations"][0]["genome"]["body"]["circle"]["adaptations"][
            "neural_growth"
        ] = 1.0
        for node in config["populations"][0]["genome"]["brain"]["nodes"]:
            if node["port"] == 360:
                node["bias"] = 8.0
        runtime = EcosystemRuntime.create(rules, config, controller="ltc")
        graph = None
        for _ in range(20):
            runtime.advance()
            graph = runtime.snapshot()["world"]["ledger"]["organisms"][0].get(
                "expressed_brain"
            )
            if graph is not None:
                break
        self.assertIsNotNone(graph)
        identity = graph["connections"][-1]["id"]
        self.assertEqual(
            float(runtime.brains.weights[runtime.brains.edge_index[(1, identity)]]), 0
        )
        runtime.advance(2)
        self.assertGreater(
            abs(
                float(runtime.brains.weights[runtime.brains.edge_index[(1, identity)]])
            ),
            0,
        )

    def test_previous_liquid_dynamics_restore_without_silent_conversion(self):
        runtime = EcosystemRuntime.create(*fixture(), controller="ltc_v1")
        runtime.advance(3)
        saved = runtime.snapshot()
        restored = EcosystemRuntime.from_snapshot(saved)
        self.assertEqual(restored.brains.kind, "plastic_sparse_ltc_v1")
        runtime.advance(3)
        restored.advance(3)
        self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_temporal_value_learning_and_activity_survive_restore(self):
        rules, config = fixture()
        config["evolution"].update(temporal_feedback=True, embodied_ecology=True)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(4)
        self.assertTrue(any(abs(v) > 0 for v in runtime.brains.critic[1]))
        self.assertGreater(float(runtime.brains.activity.sum()), 0)
        checkpoint = runtime.snapshot()
        restored = EcosystemRuntime.from_snapshot(checkpoint)
        runtime.advance(4)
        restored.advance(4)
        self.assertEqual(runtime.snapshot(), restored.snapshot())
        checkpoint["neural"]["critic"][0][1][0] = float("nan")
        with self.assertRaises(ValueError):
            EcosystemRuntime.from_snapshot(checkpoint)

    def test_genetic_only_inheritance_does_not_block_reproduction(self):
        rules, config = fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"]["inherit_learning"] = False
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 103): 1.0})
        self.assertEqual(json.loads(world.summary_json())["ecology"]["births"], 1)

    def test_invalid_activity_proposal_rolls_back(self):
        from test_ecology_extensions import actions

        rules, config = fixture()
        world = Habitat(json.dumps(rules), json.dumps(config))
        before = state(world)
        batch = actions(world, {})
        batch["agents"][0]["activity"] = [[1, 2.0]]
        with self.assertRaises(ValueError):
            world.advance_controlled(json.dumps(batch))
        self.assertEqual(state(world), before)

    def test_official_liquid_candidates_learn_and_inherit(self):
        import torch
        import importlib.util

        if importlib.util.find_spec("ncps") is None:
            self.skipTest("optional ncps comparison dependency")
        from world.experiments.liquid import compare_online_liquids

        torch.set_num_threads(1)
        result = compare_online_liquids(RULES, seeds=[73], decisions=2)
        self.assertEqual(len(result["trials"]), 4)
        for row in result["trials"]:
            self.assertEqual(row["parameter_change"] > 0, row["learning"])
            self.assertTrue(row["inherited_parameters_equal"])
            self.assertTrue(row["newborn_memory_reset"])
            self.assertTrue(row["restored_next_actions_equal"])
            self.assertAlmostEqual(row["audit"]["energy_error"], 0, places=8)

    def test_small_recycler_can_fund_viable_offspring(self):
        rules, config = ecosystem_preset(RULES, terrain=False)
        rules["minimum_reproduction_age"] = 0.01
        config["evolution"].update(founder_age_spread=0.0)
        config["cell_overrides"] = []
        config["producers"]["count"] = 0
        recycler = deepcopy(config["populations"][-1])
        recycler.update(count=1, reserve_energy=12.0, reserve_nutrient=2.4)
        graph = recycler["genome"]["brain"]
        graph["connections"] = []
        for node in graph["nodes"]:
            if node["kind"] == "output":
                node["bias"] = 2.0 if node["port"] % 256 == 103 else -8.0
        config["populations"] = [recycler]
        config["evolution"]["mutation"].update(
            parameter_edits=0,
            body_edits=0,
            neural_structure_rate=0,
            body_structure_rate=0,
        )
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        self.assertGreater(runtime.summary()["ecology"]["births"], 0)
        self.assertAlmostEqual(runtime.audit()["energy_error"], 0, places=8)
        for organism in runtime.snapshot()["world"]["ledger"]["organisms"]:
            self.assertTrue(organism["alive"])
            self.assertGreaterEqual(
                organism["tissue"]["nutrient"], rules["minimum_viable_tissue_nutrient"]
            )

    def test_ann_can_choose_between_local_food_candidates(self):
        rules, config = ecosystem_preset(RULES, terrain=False)
        config["cell_overrides"] = []
        config["producers"].update(
            count=1,
            tissue_nutrient=2.0,
            reserve_nutrient=0.0,
            reserve_energy=1.0,
            positions=[{"x": 8, "y": 8}],
        )
        other = deepcopy(config["producers"])
        other.update(tissue_nutrient=7.0, positions=[{"x": 11, "y": 8}])
        herb = deepcopy(config["populations"][0])
        herb.update(count=1, positions=[{"x": 7, "y": 8}])
        config["populations"] = [other, herb]
        config["ecology"]["reproduction"] = False
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))

        def inputs():
            a = json.loads(world.agent_frame_json())["agents"][0]
            return {v["port"] % 256: v["value"] for v in a["inputs"]}

        before = inputs()
        advance(world, {(3, 124): 1.0})
        after = inputs()
        self.assertLess(after[12], before[12])
        self.assertGreater(after[13], before[13])
        self.assertAlmostEqual(after[50], before[12], places=6)

    def test_live_feeding_is_perceived_as_an_assault(self):
        rules, config = fixture(
            ("predator", "grazer"), [{"x": 8, "y": 8}, {"x": 8.86, "y": 8}]
        )
        body = config["populations"][1]["genome"]["body"]
        body["circle"]["adaptations"]["social_senses"] = True
        config["populations"][1]["genome"]["brain"] = neural_blueprint(body)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        behavior = state(world)["diagnostics"]["behavior"]["2"]
        self.assertEqual(behavior["attacker"], 1)
        self.assertGreater(behavior["damage_received"], 0)
        agent = next(
            a for a in json.loads(world.agent_frame_json())["agents"] if a["id"] == 2
        )
        inputs = {v["port"] % 256: v["value"] for v in agent["inputs"]}
        self.assertGreater(inputs[42], 0)

    def test_new_controls_are_explicit(self):
        _, config = ecosystem_preset(RULES)
        self.assertFalse(
            ablation(config, "no_social")["evolution"]["social_observations"]
        )
        self.assertFalse(
            ablation(config, "genetic_only")["evolution"]["inherit_learning"]
        )


if __name__ == "__main__":
    unittest.main()
