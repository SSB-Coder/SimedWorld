import json
import math
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from world._native import Habitat
from world.control import RunControl
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.experiments.learning import feeding_task
from world.presets import circle_preset
from world.telemetry import RunRecorder

ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text())


def fixture():
    rules, config = circle_preset(RULES, consumers=1, producers=1)
    config["ecology"]["reproduction"] = False
    fund(rules, config)
    return rules, config


class RunDiagnosticsTests(unittest.TestCase):
    def test_load_preserves_valid_neural_inspection(self):
        runtime = EcosystemRuntime.create(*fixture())
        with TemporaryDirectory() as directory:
            checkpoint = Path(directory) / "checkpoint.json"
            runtime.save(checkpoint)
            control = RunControl(runtime, checkpoint)
            control.selected = 2
            runtime.advance()
            control.apply(
                {
                    "schema_version": 1,
                    "session": control.session,
                    "id": "load-selected",
                    "kind": "load",
                    "path": "",
                }
            )
            self.assertEqual(control.selected, 2)
            self.assertEqual(
                control.runtime.display(control.selected)["brain"]["id"], 2
            )
            self.assertEqual(control.runtime.tick, 0)

    def test_wounds_are_repaired_without_inflating_adult_development(self):
        rules, config = feeding_task(RULES)
        config["evolution"].update(online_learning=False, repair_nutrient_per_time=0.01)
        rules["max_ingestion_nutrient_per_time"] = 3.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        initial = config["producers"]["tissue_nutrient"]
        for _ in range(50):
            frame = json.loads(world.agent_frame_json())
            world.advance_controlled(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tick": frame["tick"],
                        "agents": [
                            {
                                "id": a["id"],
                                "version": a["version"],
                                "values": [
                                    {
                                        "port": p["port"],
                                        "value": 1.0 if p["channel"] == 120 else 0.0,
                                    }
                                    for p in a["outputs"]
                                ],
                            }
                            for a in frame["agents"]
                        ],
                    }
                )
            )
        producer = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
        self.assertLess(producer["tissue"]["nutrient"], initial)
        self.assertLessEqual(
            sum(m["developed"] for m in producer["body"]["modules"]), initial + 1e-8
        )
        world.audit_json()

    def test_contact_growth_moves_only_within_the_paid_expansion(self):
        rules, config = feeding_task(RULES)
        config["evolution"]["online_learning"] = False
        consumer = config["populations"][0]
        consumer.update(tissue_nutrient=2.5, reserve_nutrient=2.5, reserve_energy=12.0)
        density = 12.0 * math.pi
        old_radius = math.sqrt(2.5 / density)
        producer_radius = math.sqrt(config["producers"]["tissue_nutrient"] / density)
        x = 8.0 - producer_radius - old_radius
        consumer["positions"] = [{"x": x, "y": 8.0}]
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(
            json.dumps(
                {
                    "schema_version": 1,
                    "tick": frame["tick"],
                    "agents": [
                        {
                            "id": a["id"],
                            "version": a["version"],
                            "values": [
                                {"port": p["port"], "value": 0.0} for p in a["outputs"]
                            ],
                        }
                        for a in frame["agents"]
                    ],
                }
            )
        )
        state = json.loads(world.checkpoint_json())
        after = state["ledger"]["organisms"][1]
        self.assertGreater(after["tissue"]["nutrient"], 2.5)
        new_x = state["positions"][1]["position"]["x"]
        new_radius = math.sqrt(after["tissue"]["nutrient"] / density)
        self.assertLess(new_x, x)
        self.assertLessEqual(x - new_x, new_radius - old_radius + 1e-9)
        self.assertGreater(state["diagnostics"]["costs"]["growth_displacement"], 0)
        world.audit_json()

    def test_grazing_leaves_declared_tissue_but_does_not_grant_immortality(self):
        for light, expected_alive in [(8.0, True), (0.0, False)]:
            rules, config = feeding_task(RULES)
            config["evolution"].update(
                online_learning=False, repair_nutrient_per_time=1e-10
            )
            config["ecology"]["grazing_residual_fraction"] = 0.15
            config["light_energy_per_cell_per_time"] = light
            if not light:
                config["producers"]["reserve_energy"] = 0.03
            fund(rules, config)
            world = Habitat(json.dumps(rules), json.dumps(config))
            for _ in range(800 if light else 10):
                frame = json.loads(world.agent_frame_json())
                world.advance_controlled(
                    json.dumps(
                        {
                            "schema_version": 1,
                            "tick": frame["tick"],
                            "agents": [
                                {
                                    "id": a["id"],
                                    "version": a["version"],
                                    "values": [
                                        {
                                            "port": p["port"],
                                            "value": 1.0
                                            if p["channel"] == 120
                                            else 0.0,
                                        }
                                        for p in a["outputs"]
                                    ],
                                }
                                for a in frame["agents"]
                            ],
                        }
                    )
                )
            state = json.loads(world.checkpoint_json())
            producer = state["ledger"]["organisms"][0]
            self.assertEqual(producer["alive"], expected_alive)
            if light:
                self.assertGreater(state["totals"]["ingested_nutrient"], 5)
                developed = sum(m["developed"] for m in producer["body"]["modules"])
                self.assertGreaterEqual(
                    producer["tissue"]["nutrient"], developed * 0.15 - 1e-9
                )
            world.audit_json()

    def test_full_consumers_do_not_destroy_food_they_cannot_store(self):
        amounts = []
        for full in [True, False]:
            rules, config = feeding_task(RULES)
            rules["sensor_energy_per_time"] = 1e-12
            config["evolution"].update(
                online_learning=False,
                neuron_energy_per_time=1e-12,
                connection_energy_per_time=1e-12,
            )
            consumer = config["populations"][0]
            consumer["reserve_nutrient"] = (
                consumer["tissue_nutrient"]
                * rules["reserve_nutrient_capacity_per_tissue_nutrient"]
            )
            if full:
                consumer["reserve_energy"] = (
                    consumer["tissue_nutrient"]
                    * rules["reserve_energy_capacity_per_tissue_nutrient"]
                )
            fund(rules, config)
            world = Habitat(json.dumps(rules), json.dumps(config))
            frame = json.loads(world.agent_frame_json())
            actions = {
                "schema_version": 1,
                "tick": frame["tick"],
                "agents": [
                    {
                        "id": a["id"],
                        "version": a["version"],
                        "values": [
                            {
                                "port": p["port"],
                                "value": 1.0 if p["channel"] == 120 else 0.0,
                            }
                            for p in a["outputs"]
                        ],
                    }
                    for a in frame["agents"]
                ],
            }
            world.advance_controlled(json.dumps(actions))
            amounts.append(
                json.loads(world.checkpoint_json())["totals"]["ingested_nutrient"]
            )
            world.audit_json()
        self.assertLess(amounts[0], 1e-8)
        self.assertGreater(amounts[1], 0.01)

    def test_telemetry_is_read_only_and_accounts_for_all_nutrients(self):
        runtime = EcosystemRuntime.create(*fixture())
        runtime.advance(3)
        before = runtime.snapshot()
        row = runtime.telemetry()
        total = sum(
            r["reserve_nutrient"] + r["tissue_nutrient"] for r in row["roles"].values()
        )
        total += (
            row["resources"]["mineral_nutrient"] + row["resources"]["detritus_nutrient"]
        )
        self.assertAlmostEqual(
            total, before["world"]["config"]["initial_nutrient"], places=8
        )
        self.assertLessEqual(
            row["resources"]["producer_cell_mineral"],
            row["resources"]["mineral_nutrient"],
        )
        self.assertEqual(before, runtime.snapshot())
        self.assertEqual(runtime.display()["summary"]["health"], row)

    def test_birth_rejections_are_partitioned_by_role(self):
        rules, config = fixture()
        rules["minimum_reproduction_age"] = 0.1
        config["ecology"].update(reproduction=True, max_population=2)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        row = runtime.telemetry()
        self.assertGreater(
            row["roles"]["producer"]["birth_blocks"]["population_limit"], 0
        )
        self.assertGreater(
            row["roles"]["consumer"]["birth_blocks"]["population_limit"], 0
        )
        diagnostic = runtime.snapshot()["world"]["diagnostics"]
        self.assertEqual(
            sum(sum(r.values()) for r in diagnostic["birth_blocks_by_role"].values()),
            diagnostic["failed_births"],
        )

    def test_recorder_bounds_autosaves_and_preserves_exact_continuation(self):
        runtime = EcosystemRuntime.create(*fixture())
        with TemporaryDirectory() as directory:
            output = Path(directory) / "run.json"
            recorder = RunRecorder(runtime, output, sample_ticks=6, checkpoint_ticks=6)
            recorder.sample(runtime)
            self.assertEqual(
                len(recorder.path("-telemetry.jsonl").read_text().splitlines()), 1
            )
            for _ in range(5):
                runtime.advance()
                recorder.sample(runtime)
            self.assertEqual(len(list(Path(directory).glob("run-autosave-*.json"))), 3)
            manifest = json.loads(
                recorder.path(f"-manifest-{recorder.run_id}-0.json").read_text()
            )
            self.assertTrue(
                any(p.endswith((".pyd", ".so")) for p in manifest["source_sha256"])
            )
            latest = max(
                Path(directory).glob("run-autosave-*.json"),
                key=lambda p: json.loads(p.read_text())["world"]["clock"]["tick"],
            )
            restored = EcosystemRuntime.load(latest)
            self.assertEqual(runtime.snapshot(), restored.snapshot())
            runtime.advance(2)
            restored.advance(2)
            self.assertEqual(runtime.snapshot(), restored.snapshot())
            recorder.sample(restored)
            self.assertEqual(recorder.segment, 1)
            self.assertTrue(
                recorder.path(f"-manifest-{recorder.run_id}-1.json").exists()
            )

    def test_recorder_captures_extinction_without_reseeding(self):
        rules, config = fixture()
        config["populations"][0].update(
            reserve_energy=0.03, positions=[{"x": 35.0, "y": 19.0}]
        )
        config["producers"]["positions"] = [{"x": 2.0, "y": 2.0}]
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        with TemporaryDirectory() as directory:
            recorder = RunRecorder(
                runtime, Path(directory) / "run.json", sample_ticks=6
            )
            runtime.advance(10)
            row = recorder.sample(runtime)
            self.assertEqual(row["roles"]["consumer"]["count"], 0)
            saved = recorder.path(
                f"-{recorder.run_id}-segment-0-consumer-extinction.json"
            )
            self.assertEqual(
                runtime.snapshot(), EcosystemRuntime.load(saved).snapshot()
            )
            self.assertEqual(runtime.summary()["consumer_count"], 0)


if __name__ == "__main__":
    unittest.main()
