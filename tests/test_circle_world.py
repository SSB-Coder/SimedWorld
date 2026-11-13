from copy import deepcopy
import json
from pathlib import Path
import runpy
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import torch

from world._native import Habitat
from world.brains.plastic import PlasticPopulation
from world.ecosystem import EcosystemRuntime
from world.control import RunControl, RunLease
from world.experiments.adaptation import fund
from world.experiments.learning import compare_learning, feeding_task
from world.presets import circle_genome, circle_preset


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text())


def setup(reproduction=False):
    rules, config = circle_preset(RULES, consumers=2, producers=4)
    rules.update(world_width=16, world_height=16, grid_width=16, grid_height=16)
    config["ecology"].update(reproduction=reproduction, max_population=32)
    config["evolution"]["mutation"].update(
        parameter_edits=0,
        body_edits=0,
        body_structure_rate=0,
        neural_structure_rate=0,
        brain_creation_rate=0,
    )
    fund(rules, config)
    return rules, config


class CircleWorldTests(unittest.TestCase):
    def test_missing_viewer_fails_before_starting_or_publishing_a_world(self):
        entry = runpy.run_path(str(ROOT / "examples/evolution_demo.py"))["run"]
        with patch("pathlib.Path.exists", return_value=False):
            with self.assertRaisesRegex(
                FileNotFoundError, "viewer executable is missing"
            ):
                entry(SimpleNamespace(viewer=True))

    def test_observer_chooses_a_valid_protocol_after_controller_family_changes(self):
        runtime = EcosystemRuntime.create(*setup())
        control = RunControl(runtime, ROOT / "artifacts/unused-checkpoint.json")
        self.assertTrue(control.lightweight)
        rules, config = setup()
        config["evolution"]["online_learning"] = False
        fixed = EcosystemRuntime.create(rules, config)
        with patch("world.control.EcosystemRuntime.load", return_value=runtime):
            control.lightweight = False
            control.selected = 999
            control.apply(
                {
                    "schema_version": 1,
                    "session": control.session,
                    "id": "load-plastic",
                    "kind": "load",
                    "path": "unused",
                }
            )
        self.assertTrue(control.lightweight)
        self.assertIsNone(control.selected)
        with patch("world.control.EcosystemRuntime.load", return_value=fixed):
            control.apply(
                {
                    "schema_version": 1,
                    "session": control.session,
                    "id": "load-fixed",
                    "kind": "load",
                    "path": "unused",
                }
            )
        self.assertFalse(control.lightweight)

    def test_plain_output_decodes_raw_utf8_and_preserves_failure_code(self):
        command = [
            sys.executable,
            str(ROOT / "scripts/plain_command.py"),
            "--",
            sys.executable,
            "-c",
            "import sys;sys.stdout.buffer.write(bytes([240,159,141,185])+b' Build ok\\n'+b'-'*12+b'\\n');sys.exit(7)",
        ]
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 7)
        self.assertEqual(result.stdout, b" Build ok\n")
        self.assertEqual(result.stderr, b"")

    def test_circle_geometry_allows_approach_and_continued_feeding(self):
        rules, config = feeding_task(RULES)
        config["populations"][0]["positions"] = [{"x": 6.0, "y": 8.0}]
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        payload = runtime.snapshot()["world"]
        for item in payload["motion"]:
            item["heading"] = 0.0
        world = Habitat.from_checkpoint_json(json.dumps(payload))
        for _ in range(160):
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
                                        "value": 0.0 if p["channel"] == 111 else 1.0,
                                    }
                                    for p in a["outputs"]
                                ],
                            }
                            for a in frame["agents"]
                        ],
                    }
                )
            )
        before = json.loads(world.checkpoint_json())
        self.assertGreater(before["totals"]["distance_moved"], 0.5)
        fed = before["totals"]["feeding_events"]
        for _ in range(30):
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
                                        "value": 0.0 if p["channel"] == 111 else 1.0,
                                    }
                                    for p in a["outputs"]
                                ],
                            }
                            for a in frame["agents"]
                        ],
                    }
                )
            )
        after = json.loads(world.checkpoint_json())
        self.assertTrue(after["ledger"]["organisms"][1]["alive"])
        self.assertGreater(after["totals"]["feeding_events"], fed + 20)
        self.assertEqual(
            len(after["ledger"]["organisms"][1]["genome"]["body"]["modules"]), 1
        )
        world.audit_json()

    def test_online_learning_improves_a_real_feeding_task(self):
        report = compare_learning(RULES, seeds=[73], decisions=400)
        plastic, frozen = report["runs"]
        self.assertTrue(plastic["alive"])
        self.assertGreater(plastic["parameter_change"], 0)
        self.assertEqual(frozen["parameter_change"], 0)
        self.assertGreater(plastic["food_ingested"], frozen["food_ingested"] * 1.25)

    def test_neural_controller_moves_to_food_from_a_separated_start(self):
        rules, config = setup()
        config["producers"].update(count=1, positions=[{"x": 8.0, "y": 8.0}])
        config["populations"][0].update(count=1, positions=[{"x": 6.0, "y": 8.0}])
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        state = runtime.snapshot()
        for heading in state["world"]["motion"]:
            heading["heading"] = 0.0
        runtime = EcosystemRuntime.from_snapshot(state)
        runtime.advance(120)
        summary = runtime.summary()
        self.assertGreater(summary["ecology"]["distance_moved"], 0.5)
        self.assertGreater(summary["ecology"]["feeding_events"], 5)
        self.assertEqual(summary["consumer_count"], 1)

    def test_independent_noise_does_not_depend_on_batch_membership(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        complete, _ = runtime.brains.propose(frame)
        for index, agent in enumerate(frame["agents"]):
            single = deepcopy(frame)
            single["agents"] = [agent]
            model = PlasticPopulation(single)
            result, _ = model.propose(single)
            for a, b in zip(
                complete["agents"][index]["values"],
                result["agents"][0]["values"],
                strict=True,
            ):
                self.assertAlmostEqual(a["value"], b["value"], places=13)

    def test_birth_inherits_learned_weights_and_repacking_keeps_parent_learning(self):
        rules, config = setup(True)
        rules.update(minimum_reproduction_age=0.2, reproduction_cooldown=0.2)
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        runtime.brains.weights = runtime.brains.weights + 0.5
        before_weights = {
            key: runtime.brains.weights[i].item()
            for key, i in runtime.brains.edge_index.items()
        }
        proposal, _ = runtime.brains.propose(runtime.frame())
        inherited = {
            a["id"]: a["learned"] for a in proposal["agents"] if "learned" in a
        }
        runtime.advance()
        state = runtime.snapshot()
        children = [
            o
            for o in state["world"]["ledger"]["organisms"]
            if o["parent_id"] in inherited
        ]
        self.assertTrue(children)
        for child in children:
            source = inherited[child["parent_id"]]
            self.assertEqual(
                {e["id"]: e["weight"] for e in child["genome"]["brain"]["connections"]},
                dict(source["weights"]),
            )
            self.assertEqual(
                {n["id"]: n["bias"] for n in child["genome"]["brain"]["nodes"]},
                dict(source["biases"]),
            )
            self.assertEqual(runtime.brains.updates[child["id"]], 0)
            self.assertTrue(
                all(
                    runtime.brains.memory[i] == 0
                    for (identity, _), i in runtime.brains.index.items()
                    if identity == child["id"]
                )
            )
        for identity in inherited:
            self.assertGreater(runtime.brains.changes[identity], 0)
        for key, index in runtime.brains.edge_index.items():
            if key in before_weights:
                self.assertLessEqual(
                    abs(runtime.brains.weights[index].item() - before_weights[key]),
                    0.030000000001,
                )
        restored = EcosystemRuntime.from_snapshot(state)
        runtime.advance(12)
        restored.advance(12)
        self.assertEqual(runtime.snapshot(), restored.snapshot())

    def test_learning_failure_is_atomic_with_world_memory_weights_and_traces(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(3)
        before = runtime.snapshot()
        with patch.object(
            PlasticPopulation,
            "reconcile",
            side_effect=RuntimeError("injected learning failure"),
        ):
            with self.assertRaises(RuntimeError):
                runtime.advance()
        self.assertEqual(runtime.snapshot(), before)

    def test_conflicting_run_owners_are_rejected_and_lease_is_reusable(self):
        import tempfile

        with tempfile.TemporaryDirectory() as directory:
            stream = Path(directory) / "stream.json"
            with RunLease(stream):
                with self.assertRaises(RuntimeError):
                    with RunLease(stream):
                        pass
            with RunLease(stream):
                pass

    def test_bad_learned_inheritance_payload_cannot_change_the_world(self):
        rules, config = setup(True)
        rules["minimum_reproduction_age"] = 0.1
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(1)
        batch, _ = runtime.brains.propose(runtime.frame())
        selected = next(i for i, a in enumerate(batch["agents"]) if "learned" in a)
        before = runtime.snapshot()
        for kind in ["duplicate", "missing", "nonfinite"]:
            invalid = deepcopy(batch)
            weights = invalid["agents"][selected]["learned"]["weights"]
            if kind == "duplicate":
                weights[1] = weights[0]
            elif kind == "missing":
                weights.pop()
            else:
                weights[0][1] = float("nan")
            with self.assertRaises(ValueError):
                runtime.world.advance_controlled(json.dumps(invalid))
            self.assertEqual(runtime.snapshot(), before)

    def test_learning_checkpoint_rejects_bad_parameter_state(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(2)
        state = runtime.snapshot()
        for field, value in [
            ("weights", [float("nan")] * len(state["neural"]["weights"])),
            ("bias_trace", [11.0] * len(state["neural"]["bias_trace"])),
            ("tick", False),
            ("seed", True),
            ("noise_kind", "unknown"),
        ]:
            with self.subTest(field=field):
                broken = deepcopy(state)
                broken["neural"][field] = value
                with self.assertRaises(ValueError):
                    EcosystemRuntime.from_snapshot(broken)
        broken = deepcopy(state)
        broken["neural"]["keys"][0][0] = True
        with self.assertRaises(ValueError):
            EcosystemRuntime.from_snapshot(broken)

    def test_first_brain_is_created_at_a_funded_birth(self):
        rules, config = setup(True)
        config["populations"] = []
        config["evolution"]["mutation"]["brain_creation_rate"] = 1.0
        rules.update(minimum_reproduction_age=0.2, reproduction_cooldown=0.2)
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        self.assertEqual(runtime.frame()["agents"], [])
        runtime.advance(3)
        state = runtime.snapshot()["world"]
        self.assertTrue(
            any(
                o["parent_id"] and o["genome"].get("brain")
                for o in state["ledger"]["organisms"]
            )
        )
        self.assertTrue(
            any(
                e["kind"] == "birth"
                and any(v["kind"] == "brain_create" for v in e["edits"])
                for e in state["diagnostics"]["events"]
            )
        )

    def test_reproduction_preserves_a_maintenance_reserve(self):
        counts = []
        for horizon in [24.0, 0.0]:
            rules, config = setup(True)
            config["producers"]["count"] = 0
            config["populations"][0].update(count=1, reserve_energy=14.5)
            rules.update(minimum_reproduction_age=0.1, reproduction_cooldown=5.0)
            config["evolution"]["maintenance_reserve_time"] = horizon
            fund(rules, config)
            runtime = EcosystemRuntime.create(rules, config)
            runtime.advance(3)
            counts.append(runtime.summary()["ecology"]["births"])
        self.assertEqual(counts[0], 0)
        self.assertGreater(counts[1], 0)

    def test_capacity_limited_birth_priority_does_not_permanently_favor_producers(self):
        rules, config = setup()
        rules["minimum_reproduction_age"] = 0.1
        config["producers"]["count"] = 2
        config["ecology"]["max_population"] = 5
        config["evolution"]["maintenance_reserve_time"] = 0.1
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance()
        state = runtime.snapshot()
        state["world"]["config"]["ecology"]["reproduction"] = True
        runtime = EcosystemRuntime.from_snapshot(state)
        runtime.advance()
        children = [
            o
            for o in runtime.snapshot()["world"]["ledger"]["organisms"]
            if o["parent_id"]
        ]
        self.assertEqual(len(children), 1)
        self.assertEqual(children[0]["parent_id"], 3)

    def test_incremental_event_export_rejects_gaps_instead_of_dropping_events(self):
        rules, config = setup(True)
        rules["minimum_reproduction_age"] = 0.2
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(3)
        receipt = runtime.events_since(0)
        self.assertGreater(len(receipt["events"]), 1)
        self.assertEqual(
            [e["sequence"] for e in receipt["events"]],
            list(range(receipt["next_event"])),
        )
        with self.assertRaises(ValueError):
            runtime.events_since(2**63)
        state = runtime.snapshot()
        state["world"]["diagnostics"]["events"] = state["world"]["diagnostics"][
            "events"
        ][1:]
        resumed = EcosystemRuntime.from_snapshot(state)
        with self.assertRaises(ValueError):
            resumed.events_since(0)

    def test_large_brain_cost_has_feasible_food_headroom(self):
        rules, config = setup()
        config["populations"][0]["genome"] = circle_genome(hidden=1000)
        fund(rules, config)
        brain = config["populations"][0]["genome"]["brain"]
        cost = (
            len(brain["nodes"]) * config["evolution"]["neuron_energy_per_time"]
            + len(brain["connections"])
            * config["evolution"]["connection_energy_per_time"]
        )
        food = rules["max_ingestion_nutrient_per_time"] * (
            rules["energy_assimilation_fraction"] * rules["tissue_energy_per_nutrient"]
            - rules["digestion_energy_per_ingested_nutrient"]
        )
        self.assertLess(
            cost + 6.1 * rules["maintenance_per_tissue_nutrient_per_time"], food
        )
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        self.assertTrue(torch.isfinite(runtime.brains.weights).all())

    def test_display_packets_exclude_genomes_and_show_current_learned_parameters(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(3)
        identity = runtime.frame()["agents"][0]["id"]
        packet = runtime.display(identity)
        self.assertNotIn("world", packet)
        self.assertNotIn("genome", json.dumps(packet))
        self.assertLess(len(json.dumps(packet)), len(json.dumps(runtime.snapshot())))
        self.assertGreater(packet["brain"]["parameter_change"], 0)
        self.assertEqual(packet["brain"]["updates"], 3)

    @unittest.skipUnless(torch.cuda.is_available(), "CUDA is unavailable")
    def test_plastic_cpu_cuda_proposals_match(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        cpu, proposal = runtime.brains.propose(frame)
        gpu, other = PlasticPopulation(frame, "cuda").propose(frame)
        self.assertTrue(
            torch.allclose(proposal.memory, other.memory.cpu(), atol=1e-11, rtol=1e-11)
        )
        for a, b in zip(cpu["agents"], gpu["agents"], strict=True):
            for x, y in zip(a["values"], b["values"], strict=True):
                self.assertAlmostEqual(x["value"], y["value"], places=10)


if __name__ == "__main__":
    unittest.main()
