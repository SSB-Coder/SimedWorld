from copy import deepcopy
import json
from pathlib import Path
import runpy
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import torch

from world._native import Habitat
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.experiments.ecosystem import ecosystem_preset
from test_ecology_extensions import RULES, fixture, advance, state


ROOT = Path(__file__).resolve().parents[1]


def killed_world(diet="scavenger"):
    rules, config = fixture(
        ("grazer", "predator", diet),
        [{"x": 8.74, "y": 8}, {"x": 9.8, "y": 8}, {"x": 10.38, "y": 8}],
    )
    config["evolution"]["embodied_ecology"] = True
    for group, tissue, radius, energy, nutrient, offense in [
        (config["populations"][0], 24.2, 0.8, 90, 1, 1.0),
        (config["populations"][1], 2.4, 0.25, 8, 1, 0.1),
        (config["populations"][2], 3.2, 0.29, 3, 0, 0.1),
    ]:
        group.update(tissue_nutrient=tissue, reserve_energy=energy, reserve_nutrient=nutrient)
        group["genome"]["body"]["modules"][0]["radius"] = radius
        group["genome"]["body"]["circle"]["adaptations"]["offense"] = offense
    fund(rules, config)
    world = Habitat(json.dumps(rules), json.dumps(config))
    for _ in range(48):
        advance(world, {(1, 121): 1.0, (1, 111): 1.0})
        if not state(world)["ledger"]["organisms"][1]["alive"]:
            break
    return world


def corpses(saved):
    return [corpse for cell in saved["grid"]["cells"] for corpse in cell.get("corpses", {}).values()]


class CorpseEcologyTests(unittest.TestCase):
    def test_lethal_bite_routes_exact_remains_to_target_cell_without_round_trip(self):
        for reserve_energy in [0.1, 3.344737564475914, 4.9]:
            rules, config = fixture(("predator", "grazer"), [{"x": 9.48, "y": 8}, {"x": 10.1, "y": 8}])
            config["evolution"]["embodied_ecology"] = True
            config["populations"][0].update(reserve_nutrient=0, reserve_energy=1)
            config["populations"][1].update(tissue_nutrient=1.1, reserve_nutrient=0.7, reserve_energy=reserve_energy)
            fund(rules, config)
            world = Habitat(json.dumps(rules), json.dumps(config))
            advance(world, {(1, 120): 1.0})
            saved = state(world)
            self.assertFalse(saved["ledger"]["organisms"][1]["alive"])
            self.assertEqual(corpses(saved)[0]["source"], 2)
            self.assertAlmostEqual(corpses(saved)[0]["remaining"]["nutrient"], 1.5)
            self.assertEqual(saved["grid"]["cells"][8 * 16 + 10]["detritus"]["nutrient"], 0)
            for error in ["energy_error", "nutrient_error"]:
                self.assertAlmostEqual(json.loads(world.audit_json())[error], 0, places=8)

    def test_kill_creates_real_cross_cell_food_without_duplicating_inventory(self):
        for diet in ["scavenger", "recycler"]:
            with self.subTest(diet=diet):
                world = killed_world(diet)
                before = state(world)
                corpse = corpses(before)[0]
                self.assertEqual(corpse["source"], 2)
                self.assertEqual(corpse["cause"], "damage")
                display = json.loads(world.display_json(None))
                self.assertEqual(display["corpses"][0]["remaining"], corpse["remaining"])
                before_energy = before["ledger"]["organisms"][2]["reserves"]["energy"]
                before_minerals = sum(c["mineral_nutrient"] for c in before["grid"]["cells"])
                advance(world, {(3, 120): 1.0})
                after = state(world)
                self.assertLess(corpses(after)[0]["remaining"]["nutrient"], corpse["remaining"]["nutrient"])
                self.assertGreater(after["ledger"]["organisms"][2]["reserves"]["energy"], before_energy)
                self.assertIn(f"corpse->{diet}", after["interactions"]["food_web"]["flows"])
                if diet == "recycler":
                    self.assertGreater(sum(c["mineral_nutrient"] for c in after["grid"]["cells"]), before_minerals)
                for error in ["energy_error", "nutrient_error"]:
                    self.assertAlmostEqual(json.loads(world.audit_json())[error], 0, places=8)

    def test_corpse_is_observed_and_contact_signal_reaches_the_network(self):
        world = killed_world()
        saved = state(world)
        organism = saved["ledger"]["organisms"][2]
        from world.presets import neural_blueprint

        organism["genome"]["body"]["circle"]["adaptations"].update(social_senses=True, foraging_choices=True)
        organism["genome"]["brain"] = neural_blueprint(organism["genome"]["body"])
        world = Habitat.from_checkpoint_json(json.dumps(saved))
        agent = next(a for a in json.loads(world.agent_frame_json())["agents"] if a["id"] == 3)
        inputs = {v["port"] % 256: v["value"] for v in agent["inputs"]}
        self.assertGreater(inputs[12], 0)
        self.assertEqual(inputs[40], 1)

    def test_corpses_decay_save_restore_and_reject_invalid_sources(self):
        saved = state(killed_world())
        saved["config"]["ecology"].update(decomposition=True, decomposition_fraction_per_time=0.5)
        first = Habitat.from_checkpoint_json(json.dumps(saved))
        second = Habitat.from_checkpoint_json(first.checkpoint_json())
        before = corpses(saved)[0]["remaining"]["energy"]
        advance(first, {})
        advance(second, {})
        self.assertEqual(state(first), state(second))
        self.assertLess(corpses(state(first))[0]["remaining"]["energy"], before)
        self.assertGreater(corpses(state(first))[0]["age"], 0)
        bad = deepcopy(saved)
        next(cell for cell in bad["grid"]["cells"] if cell.get("corpses"))["corpses"]["2"]["source"] = 1
        with self.assertRaisesRegex(RuntimeError, "corpse does not match"):
            Habitat.from_checkpoint_json(json.dumps(bad))

    def test_starvation_creates_corpse_and_zero_energy_detritus_is_not_eaten(self):
        rules, config = fixture(("scavenger",))
        config["evolution"]["embodied_ecology"] = True
        config["cell_overrides"] = [{"column": 4, "row": 8, "detritus": {"nutrient": 8, "energy": 0}}]
        config["populations"][0].update(tissue_nutrient=1.01, reserve_energy=0.01, reserve_nutrient=0)
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(24):
            advance(world, {(1, 120): 1.0})
        saved = state(world)
        self.assertFalse(saved["ledger"]["organisms"][0]["alive"])
        self.assertEqual(corpses(saved)[0]["cause"], "starvation")
        self.assertNotIn("detritus->scavenger", saved["interactions"]["food_web"]["flows"])
        self.assertAlmostEqual(json.loads(world.audit_json())["energy_error"], 0, places=8)

    def test_interactive_preset_matches_factory_and_producer_controls_are_validated(self):
        rules, config = ecosystem_preset(json.loads((ROOT / "configs/circle_rules.json").read_text()))
        self.assertEqual(rules, json.loads((ROOT / "configs/food_web_rules.json").read_text()))
        self.assertEqual(config, json.loads((ROOT / "configs/food_web_habitat.json").read_text()))
        for key in ["producer_minimum_reproduction_age", "producer_reproduction_cooldown", "producer_reproduction_reserve_fraction"]:
            invalid = deepcopy(config)
            invalid["ecology"][key] = -1
            with self.assertRaises(ValueError):
                Habitat(json.dumps(rules), json.dumps(invalid))

    def test_producer_delay_and_cooldown_do_not_delay_consumer_births(self):
        rules, config = fixture()
        config["producers"].update(count=1, positions=[{"x": 12, "y": 12}])
        config["ecology"].update(reproduction=True, producer_minimum_reproduction_age=1.0,
            producer_reproduction_cooldown=60.0)
        config["evolution"]["inherit_learning"] = False
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(2, 103): 1.0})
        saved = state(world)
        self.assertEqual(saved["diagnostics"]["births_by_role"].get("producer", 0), 0)
        self.assertEqual(saved["diagnostics"]["births_by_role"].get("consumer", 0), 1)
        for _ in range(12):
            advance(world, {})
        saved = state(world)
        self.assertGreater(saved["diagnostics"]["births_by_role"]["producer"], 0)
        parent = saved["ledger"]["organisms"][0]
        self.assertGreaterEqual(parent["next_reproduction_age"], 61)
        consumer = saved["ledger"]["organisms"][1]
        self.assertLess(consumer["next_reproduction_age"], 60)

    def test_plant_litter_is_paid_from_actual_tissue_and_can_feed_neighbouring_cells(self):
        rules, config = fixture(("scavenger",), [{"x": 9.26, "y": 8}])
        config["producers"].update(count=1, positions=[{"x": 8.4, "y": 8}])
        config["ecology"]["producer_litter_fraction_per_time"] = 0.1
        config["cell_overrides"] = []
        config["populations"][0]["reserve_energy"] = 1.0
        config["populations"][0]["reserve_nutrient"] = 0.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        before = state(world)
        self.assertGreater(before["totals"]["producer_litter_nutrient"], 0)
        self.assertGreater(sum(c["detritus"]["nutrient"] for c in before["grid"]["cells"]), 0)
        advance(world, {(2, 120): 1.0})
        after = state(world)
        self.assertIn("detritus->scavenger", after["interactions"]["food_web"]["flows"])
        self.assertAlmostEqual(json.loads(world.audit_json())["energy_error"], 0, places=8)
        self.assertAlmostEqual(json.loads(world.audit_json())["nutrient_error"], 0, places=8)

    def test_recycled_corpse_minerals_fund_nearby_plant_growth(self):
        rules, config = fixture(("grazer", "predator", "recycler"),
            [{"x": 8.74, "y": 8}, {"x": 9.8, "y": 8}, {"x": 10.38, "y": 8}])
        config.update(mineral_nutrient_per_cell=0.0, light_energy_per_cell_per_time=3.0)
        config["evolution"]["embodied_ecology"] = True
        config["producers"].update(count=1, tissue_nutrient=1.05, reserve_nutrient=0,
            reserve_energy=4.0, positions=[{"x": 10.85, "y": 8.65}])
        for group, tissue, radius, energy, nutrient, offense in [
            (config["populations"][0], 24.2, 0.8, 90, 1, 1.0),
            (config["populations"][1], 2.4, 0.25, 8, 1, 0.1),
            (config["populations"][2], 3.2, 0.29, 3, 0, 0.1),
        ]:
            group.update(tissue_nutrient=tissue, reserve_energy=energy, reserve_nutrient=nutrient)
            group["genome"]["body"]["modules"][0]["radius"] = radius
            group["genome"]["body"]["circle"]["adaptations"]["offense"] = offense
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(48):
            advance(world, {(2, 121): 1.0, (2, 111): 1.0})
            if not state(world)["ledger"]["organisms"][2]["alive"]:
                break
        before = state(world)["ledger"]["organisms"][0]["tissue"]["nutrient"]
        for _ in range(5):
            advance(world, {(4, 120): 1.0})
        saved = state(world)
        self.assertIn("corpse->recycler", saved["interactions"]["food_web"]["flows"])
        self.assertGreater(saved["ledger"]["organisms"][0]["tissue"]["nutrient"], before)
        self.assertAlmostEqual(json.loads(world.audit_json())["nutrient_error"], 0, places=8)

    def test_depleted_plant_does_not_keep_grazer_stopped_at_contact(self):
        rules, config = ecosystem_preset(RULES, terrain=False)
        config["producers"].update(count=1, tissue_nutrient=1.05, reserve_nutrient=0, reserve_energy=1,
            positions=[{"x": 8, "y": 8}])
        grazer = deepcopy(config["populations"][0])
        grazer.update(count=1, positions=[{"x": 8.9, "y": 8}])
        config["populations"] = [grazer]
        config["cell_overrides"] = []
        fund(rules, config)
        saved = state(Habitat(json.dumps(rules), json.dumps(config)))
        saved["ledger"]["organisms"][0]["body"]["modules"][0]["developed"] = 7.0
        world = Habitat.from_checkpoint_json(json.dumps(saved))
        inputs = {v["port"] % 256: v["value"] for v in json.loads(world.agent_frame_json())["agents"][0]["inputs"]}
        self.assertEqual(inputs[40], 0)

    def test_regularized_learning_round_trip_preserves_weights_and_anchors(self):
        rules, config = fixture()
        config["evolution"].update(embodied_ecology=True, plasticity_decay_per_time=0.1)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.brains.weights += 0.2
        runtime.brains.biases[runtime.brains.input_indices] = 0
        with TemporaryDirectory() as folder:
            path = Path(folder) / "state.json"
            runtime.save(path)
            restored = EcosystemRuntime.load(path)
            runtime.advance(3)
            restored.advance(3)
            self.assertTrue(torch.equal(runtime.brains.weights, restored.brains.weights))
            self.assertTrue(torch.equal(runtime.brains.inherited_weights, restored.brains.inherited_weights))

    def test_delta_frames_preserve_every_controller_setting(self):
        rules, config = fixture()
        config["evolution"].update(plasticity_decay_per_time=0.1, temporal_feedback=True)
        runtime = EcosystemRuntime.create(rules, config)
        full = runtime.frame()
        delta = json.loads(runtime.world.agent_frame_delta_json("{}"))
        self.assertEqual(full, delta)

    def test_regularization_survives_a_birth_and_exact_restoration(self):
        rules, config = fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"].update(plasticity_decay_per_time=0.1, inherit_learning=False)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance()
        self.assertGreater(runtime.summary()["ecology"]["births"], 0)
        self.assertEqual(runtime.brains.plasticity_decay, 0.1)
        restored = EcosystemRuntime.from_snapshot(runtime.snapshot())
        runtime.advance(2)
        restored.advance(2)
        self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_regularized_empty_population_can_continue_and_restore(self):
        rules, config = fixture()
        config["populations"] = []
        config["evolution"]["plasticity_decay_per_time"] = 0.1
        fund(rules, config)
        for controller in ["sparse", "ltc"]:
            runtime = EcosystemRuntime.create(rules, config, controller=controller)
            runtime.advance(2)
            restored = EcosystemRuntime.from_snapshot(runtime.snapshot())
            runtime.advance()
            restored.advance()
            self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_liquid_direction_synapse_is_neutral_at_zero_and_respects_sign(self):
        rules, config = fixture()
        graph = config["populations"][0]["genome"]["brain"]
        ports = {n["port"] % 256: n["id"] for n in graph["nodes"] if n["port"] is not None}
        for node in graph["nodes"]:
            node["bias"] = 0.0
        graph["connections"] = [{"id": 1, "source": ports[10], "target": ports[111], "weight": 2.0}]
        runtime = EcosystemRuntime.create(rules, config, controller="ltc")
        brain = runtime.brains
        outputs = []
        for value in [-1.0, 0.0, 1.0]:
            previous = torch.zeros_like(brain.memory)
            previous[brain.index[(1, ports[10])]] = value
            result = brain.transition(previous, previous, torch.zeros_like(previous), 0.1)
            outputs.append(float(result[brain.index[(1, ports[111])]]))
            self.assertLessEqual(float(result.abs().max()), 1)
        self.assertLess(outputs[0], 0)
        self.assertEqual(outputs[1], 0)
        self.assertGreater(outputs[2], 0)
        self.assertAlmostEqual(outputs[0], -outputs[2])

    def test_previous_liquid_versions_keep_exact_continuation(self):
        for controller in ["ltc_v1", "ltc_v2"]:
            runtime = EcosystemRuntime.create(*fixture(), controller=controller)
            runtime.advance(2)
            restored = EcosystemRuntime.from_snapshot(runtime.snapshot())
            runtime.advance(2)
            restored.advance(2)
            self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_new_liquid_subjects_can_be_archived_and_evaluated(self):
        from world.archive import SubjectArchive, read_subject
        from world.experiments.evaluation import evaluate_subject

        runtime = EcosystemRuntime.create(*fixture(), controller="ltc")
        runtime.advance(2)
        with TemporaryDirectory() as folder:
            path = SubjectArchive(folder).capture(runtime, 1)
            self.assertEqual(read_subject(path)["controller_kind"], "plastic_sparse_ltc_v3")
            result = evaluate_subject(path, {"schema_version": 1, "seeds": [89],
                "tasks": ["foraging", "memory"], "decisions": 1})
            self.assertTrue(result["results"])
            self.assertTrue(result["memory"]["available"])
            self.assertEqual(result["memory"]["controller"], "plastic_sparse_ltc_v3")

    def test_actual_ltc_entrypoint_publishes_display_and_reports_viewer_failure(self):
        entry = runpy.run_path(str(ROOT / "examples/evolution_demo.py"))["run"]
        for exit_code in [0, 7]:
            with self.subTest(exit_code=exit_code), TemporaryDirectory() as folder:
                root = Path(folder)
                rules, config = fixture()
                for name, value in [("rules", rules), ("config", config)]:
                    (root / f"{name}.json").write_text(json.dumps(value))
                args = SimpleNamespace(rules=root / "rules.json", config=root / "config.json", resume=None,
                    device="cpu", controller="ltc", checkpoint=root / "state.json", stream=root / "observer.json",
                    commands=root / "commands", paused=True, viewer=True, viewer_executable=Path(__file__),
                    output=root / "run.json", events=root / "events.jsonl", interactive=True,
                    decisions=0, sample_every=1)
                viewer = SimpleNamespace(returncode=exit_code)
                def poll():
                    envelope = json.loads(args.stream.read_text())
                    self.assertEqual(envelope["kind"], "world.display")
                    self.assertTrue(envelope["paused"])
                    return exit_code
                viewer.poll = poll
                with patch("subprocess.Popen", return_value=viewer):
                    if exit_code:
                        with self.assertRaisesRegex(RuntimeError, "Viewer exited with code 7"):
                            entry(args)
                    else:
                        entry(args)
                self.assertEqual(json.loads(args.checkpoint.read_text())["neural"]["kind"], "plastic_sparse_ltc_v3")
                self.assertEqual(json.loads(args.output.read_text())["decisions_executed"], 0)


if __name__ == "__main__":
    unittest.main()
