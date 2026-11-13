import json
from pathlib import Path
import tempfile
import unittest

from world.ecosystem import EcosystemRuntime
from world.control import RunControl
from world.archive import read_subject
from world.experiments.batch import run_batch
from world.experiments.ecosystem import hunting_scenario, ablation


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/circle_rules.json").read_text())


class StudyWorkflowTests(unittest.TestCase):
    def test_owner_export_command_produces_a_complete_subject(self):
        from world.experiments.ecosystem import ecosystem_preset

        runtime = EcosystemRuntime.create(*ecosystem_preset(RULES))
        with tempfile.TemporaryDirectory() as folder:
            control = RunControl(runtime, Path(folder) / "checkpoint.json", paused=True)
            control.selected = runtime.frame()["agents"][0]["id"]
            before = runtime.snapshot()
            control.apply(
                {
                    "schema_version": 1,
                    "session": control.session,
                    "id": "export-test",
                    "kind": "export",
                    "path": "",
                }
            )
            paths = list((Path(folder) / "subjects").glob("*.json"))
            self.assertEqual(len(paths), 1)
            self.assertEqual(
                read_subject(paths[0])["origin"]["organism"]["id"], control.selected
            )
            self.assertEqual(runtime.snapshot(), before)

    def test_batch_writes_checkpoints_diagnostics_and_real_subjects(self):
        with tempfile.TemporaryDirectory() as folder:
            request = {
                "rules": RULES,
                "seed": 73,
                "output": str(Path(folder) / "trial"),
                "decisions": 2,
            }
            receipt = run_batch([request])[0]
            self.assertEqual(receipt["status"], "completed", receipt)
            result = json.loads(Path(receipt["result"]).read_text())
            self.assertEqual(result["end_tick"], 12)
            self.assertGreater(result["subjects_retained"], 0)
            restored = EcosystemRuntime.load(
                Path(request["output"]) / "checkpoint.json"
            )
            self.assertEqual(restored.tick, 12)
            with self.assertRaises(ValueError):
                run_batch([request])
            with self.assertRaises(ValueError):
                run_batch([request, request])

    def test_cost_matched_signal_control_keeps_emitters(self):
        rules, config = hunting_scenario(RULES)
        controlled = ablation(config, "ineffective_signals")
        self.assertFalse(controlled["evolution"]["receive_signals"])
        self.assertEqual(controlled["populations"], config["populations"])
        world = EcosystemRuntime.create(rules, controlled)
        world.advance(2)
        self.assertAlmostEqual(world.audit()["nutrient_error"], 0, places=8)


if __name__ == "__main__":
    unittest.main()
