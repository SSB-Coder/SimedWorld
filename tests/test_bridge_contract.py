from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest

import numpy as np
import torch

from world import FixtureWorld
from world.brains.contracts import ActionBatch, ObservationBatch
from world.brains.registry import ControllerRegistry
from world.runtime import BridgeRuntime


CONFIG = Path(__file__).resolve().parents[1] / "configs/bridge_fixture.json"


def fixture():
    return FixtureWorld(CONFIG.read_text(encoding="utf-8"))


def action_payload(world):
    observations = world.observe()
    return {**observations, "values": np.array([[0.5, 0.25], [-0.75, 0.5], [0, 1]], dtype=np.float32)}


class BridgeContractTests(unittest.TestCase):
    def test_owned_observations_keep_integer_identity_and_do_not_alias_world(self):
        world = fixture()
        before = world.snapshot_json()
        batch = world.observe()
        self.assertEqual(batch["ids"].dtype, np.uint64)
        self.assertEqual(int(batch["ids"][-1]), 2**53 + 1)
        self.assertEqual(batch["values"].shape, (3, 3))
        self.assertEqual(batch["values"].dtype, np.float32)
        batch["ids"][:] = 0
        batch["values"][:] = 0
        self.assertEqual(world.snapshot_json(), before)
        del world
        self.assertEqual(batch["values"].sum(), 0)

    def test_reordered_action_rows_reach_correct_organisms(self):
        world = fixture()
        payload = action_payload(world)
        actions = ActionBatch.from_native(payload).reordered([2, 0, 1])
        world.apply_actions(**actions.to_native())
        snapshot = json.loads(world.snapshot_json())
        self.assertEqual(world.tick, 1)
        self.assertEqual([item["last_action"] for item in snapshot["organisms"]], payload["values"].tolist())
        self.assertEqual([item["action_total"] for item in snapshot["organisms"]], payload["values"].tolist())

    def test_rejected_batches_preserve_all_native_state(self):
        world = fixture()
        good = action_payload(world)
        cases = []
        for field, value in [("tick", 1), ("schema_version", 2), ("tick", False), ("schema_version", True)]:
            bad = deepcopy(good); bad[field] = value; cases.append(bad)
        for field, value in [("ids", 17), ("ids", 999), ("topology_versions", 99)]:
            bad = deepcopy(good); bad[field][-1] = value; cases.append(bad)
        for value in [float("nan"), float("inf"), 1.1, -1.1]:
            bad = deepcopy(good); bad["values"][-1, 0] = value; cases.append(bad)
        bad = deepcopy(good); bad["values"][-1, 1] = -0.1; cases.append(bad)
        for field in ["ids", "topology_versions", "values"]:
            bad = deepcopy(good); bad[field] = bad[field][:-1].copy(); cases.append(bad)
        for payload in cases:
            with self.subTest(payload=payload):
                before = world.snapshot_json()
                with self.assertRaises(ValueError):
                    world.apply_actions(**payload)
                self.assertEqual(world.snapshot_json(), before)
        world.apply_actions(**good)
        before = world.snapshot_json()
        with self.assertRaises(ValueError):
            world.apply_actions(**good)
        self.assertEqual(world.snapshot_json(), before)

    def test_dtype_shape_and_stride_errors_do_not_coerce_data(self):
        world = fixture()
        good = action_payload(world)
        for field, array in [
            ("ids", good["ids"].astype(np.float64)),
            ("ids", good["ids"].astype(">u8")),
            ("topology_versions", good["topology_versions"].astype(np.int64)),
            ("values", good["values"].astype(np.float64)),
            ("values", good["values"].astype(">f4")),
            ("values", good["values"].reshape(-1)),
            ("values", np.zeros((3, 3), dtype=np.float32)),
            ("values", np.zeros((3, 4), dtype=np.float32)[:, ::2]),
            ("ids", np.zeros(6, dtype=np.uint64)[::2]),
        ]:
            with self.subTest(field=field, dtype=array.dtype, shape=array.shape):
                before = world.snapshot_json()
                with self.assertRaises((TypeError, ValueError)):
                    world.apply_actions(**{**good, field: array})
                self.assertEqual(world.snapshot_json(), before)

    def test_python_batches_copy_inputs_and_validate_the_same_contract(self):
        payload = action_payload(fixture())
        batch = ActionBatch.from_native(payload)
        payload["values"][:] = 0
        self.assertEqual(batch.values[0, 0], 0.5)
        self.assertFalse(batch.values.flags.writeable)
        for order in [[0, 0, 2], [0, 1], [0, 1, 3], [0.0, 1.0, 2.0]]:
            with self.subTest(order=order), self.assertRaises(ValueError):
                batch.reordered(order)
        for field, value in [("tick", True), ("tick", -1), ("schema_version", True)]:
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                ActionBatch.from_native({**batch.to_native(), field: value})

    def test_recurrent_controller_matches_independent_scalar_calculation(self):
        runtime = BridgeRuntime.from_config(CONFIG)
        observations = runtime.world.observe()
        memory = np.zeros((3, 2), dtype=np.float32)
        total = np.zeros((3, 2), dtype=np.float64)
        signals = observations["values"]
        for _ in range(6):
            expected = np.array([[np.clip(right - left + np.float32(0.25) * previous[0], -1, 1),
                                  np.clip(level + np.float32(0.25) * previous[1], 0, 1)]
                                 for (left, right, level), previous in zip(signals, memory, strict=True)],
                                dtype=np.float32)
            actual = runtime.advance()
            np.testing.assert_array_equal(actual.values, expected)
            total += expected
            memory = expected
        state = runtime.snapshot()
        np.testing.assert_array_equal([item["action_total"] for item in state["world"]["organisms"]], total)
        self.assertEqual(runtime.world.tick, 6)

    def test_registry_matches_memory_by_id_after_observation_reordering(self):
        observations = ObservationBatch.from_native(fixture().observe())
        first = ControllerRegistry.initialize(observations)
        _, registry = first.propose(observations)
        following = ObservationBatch(1, observations.ids, observations.topology_versions, observations.values)
        direct, direct_registry = registry.propose(following)
        shuffled, shuffled_registry = registry.propose(following.reordered([2, 0, 1]))
        np.testing.assert_array_equal(direct.values, shuffled.reordered([1, 2, 0]).values)
        self.assertEqual(direct_registry.snapshot(), shuffled_registry.snapshot())
        self.assertEqual(first.tick, 0)
        self.assertEqual(registry.tick, 1)

    def test_registry_rejects_stale_metadata_without_changing_memory(self):
        observations = ObservationBatch.from_native(fixture().observe())
        registry = ControllerRegistry.initialize(observations)
        before = registry.snapshot()
        for field, value in [("tick", 1), ("ids", np.array([17, 42, 99], dtype=np.uint64)),
                             ("topology_versions", np.ones(3, dtype=np.uint64))]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                registry.propose(ObservationBatch.from_native({**observations.to_native(), field: value}))
            self.assertEqual(registry.snapshot(), before)

    def test_native_rejection_does_not_commit_proposed_controller_memory(self):
        class RejectOnce:
            def __init__(self):
                self.inner = fixture()
                self.reject = True

            def observe(self):
                return self.inner.observe()

            def snapshot_json(self):
                return self.inner.snapshot_json()

            def apply_actions(self, **payload):
                if self.reject:
                    payload["tick"] += 1
                self.inner.apply_actions(**payload)

        world = RejectOnce()
        registry = ControllerRegistry.initialize(ObservationBatch.from_native(world.observe()))
        runtime = BridgeRuntime(world, registry)
        before = runtime.snapshot()
        with self.assertRaises(ValueError):
            runtime.advance()
        self.assertEqual(runtime.snapshot(), before)
        world.reject = False
        runtime.advance()
        control = BridgeRuntime.from_config(CONFIG)
        control.advance()
        self.assertEqual(runtime.snapshot(), control.snapshot())

    def test_cpu_checkpoint_continuation_matches_uninterrupted_execution(self):
        control = BridgeRuntime.from_config(CONFIG)
        split = BridgeRuntime.from_config(CONFIG)
        for _ in range(2):
            control.advance(); split.advance()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "checkpoint.json"
            split.save(path)
            split = BridgeRuntime.load(path)
            for _ in range(5):
                control.advance(); split.advance()
                self.assertEqual(split.snapshot(), control.snapshot())

    def test_inconsistent_or_malformed_checkpoints_are_rejected(self):
        runtime = BridgeRuntime.from_config(CONFIG)
        runtime.advance()
        original = runtime.snapshot()
        cases = []
        for field, value in [("tick", 0), ("ids", [17, 42, 99]),
                             ("topology_versions", [0, 3, 9]),
                             ("memory", [[0, 0], [0, 0], [0, 0]]),
                             ("memory", [[float("nan"), 0], [0, 0], [0, 0]]),
                             ("memory", [[1.00000001, 0], [0, 0], [0, 0]])]:
            payload = deepcopy(original); payload["controller"][field] = value; cases.append(payload)
        payload = deepcopy(original); payload["world"]["organisms"][0]["action_total"][0] = float("nan"); cases.append(payload)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "checkpoint.json"
            for payload in cases:
                path.write_text(json.dumps(payload), encoding="utf-8")
                with self.subTest(payload=payload), self.assertRaises(ValueError):
                    BridgeRuntime.load(path)
                self.assertEqual(runtime.snapshot(), original)

    def test_empty_cohort_advances_and_resumes(self):
        world = FixtureWorld('{"schema_version":1,"organisms":[]}')
        observations = ObservationBatch.from_native(world.observe())
        self.assertEqual(observations.values.shape, (0, 3))
        runtime = BridgeRuntime(world, ControllerRegistry.initialize(observations))
        self.assertEqual(runtime.advance().values.shape, (0, 2))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "empty.json"
            runtime.save(path)
            restored = BridgeRuntime.load(path)
            restored.advance()
            self.assertEqual(restored.world.tick, 2)

    def test_invalid_fixture_configs_and_snapshots_are_rejected(self):
        config = json.loads(CONFIG.read_text())
        for update in [lambda item: item.update(schema_version=2),
                       lambda item: item["organisms"].append(deepcopy(item["organisms"][0])),
                       lambda item: item["organisms"][0].update(signals=[0, 0, 2]),
                       lambda item: item.update(extra=0)]:
            value = deepcopy(config); update(value)
            with self.assertRaises(ValueError):
                FixtureWorld(json.dumps(value))
        with self.assertRaises(ValueError):
            FixtureWorld.from_snapshot_json('{"schema_version":1}')

    def test_tick_overflow_does_not_wrap(self):
        snapshot = json.loads(fixture().snapshot_json())
        snapshot["tick"] = 2**64 - 1
        world = FixtureWorld.from_snapshot_json(json.dumps(snapshot))
        before = world.snapshot_json()
        with self.assertRaises(ValueError):
            world.apply_actions(**action_payload(world))
        self.assertEqual(world.snapshot_json(), before)

    @unittest.skipUnless(torch.cuda.is_available(), "CUDA is unavailable")
    def test_cuda_and_cpu_produce_matching_fixture_trajectories(self):
        cpu = BridgeRuntime.from_config(CONFIG, "cpu")
        cuda = BridgeRuntime.from_config(CONFIG, "cuda")
        for _ in range(6):
            np.testing.assert_allclose(cpu.advance().values, cuda.advance().values, rtol=1e-6, atol=1e-7)
        np.testing.assert_allclose(cpu.registry.snapshot()["memory"], cuda.registry.snapshot()["memory"],
                                   rtol=1e-6, atol=1e-7)


if __name__ == "__main__":
    unittest.main()
