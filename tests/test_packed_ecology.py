import json
import unittest

import numpy as np
import torch

from world._native import Habitat
from world.brains.packing import compiled_graph
from world.brains.plastic import PlasticPopulation
from world.ecosystem import EcosystemRuntime
from test_circle_world import setup


class LegacyWorld:
    def __init__(self, inner):
        self.inner = inner

    def __getattr__(self, name):
        if name == "propose_controlled_packed":
            raise AttributeError(name)
        return getattr(self.inner, name)

    def propose_controlled(self, actions):
        return LegacyWorld(self.inner.propose_controlled(actions))


class PackedEcologyTests(unittest.TestCase):
    def runtime(self, controller="sparse", reproduction=True):
        rules, config = setup(reproduction)
        rules.update(minimum_reproduction_age=0.1, reproduction_cooldown=0.2)
        return EcosystemRuntime.create(rules, config, controller=controller)

    def packet(self, runtime):
        actions, _ = runtime.brains.propose(runtime._frame, packed=True)
        return [json.dumps(actions), runtime.brains.noise_ids.copy(),
            runtime.brains.biases.cpu().numpy().copy(), runtime.brains.parameter_edge_ids.copy(),
            runtime.brains.weights.cpu().numpy().copy()]

    def test_packed_and_json_paths_match_through_births_learning_and_restore(self):
        for controller in ["sparse", "ltc", "ltc_v1", "ltc_v2"]:
            packed = self.runtime(controller)
            packed.brains.weights += 0.125
            legacy = EcosystemRuntime.from_snapshot(packed.snapshot())
            legacy.world = LegacyWorld(legacy.world)
            for _ in range(15):
                packed.advance()
                legacy.advance()
                self.assertEqual(packed.snapshot(), legacy.snapshot())
            self.assertGreater(packed.summary()["ecology"]["births"], 0)
            restored = EcosystemRuntime.from_snapshot(packed.snapshot())
            restored.advance(2)
            packed.advance(2)
            self.assertEqual(packed.snapshot(), restored.snapshot())

    def test_empty_population_matches_json_path(self):
        runtime = self.runtime()
        saved = runtime.snapshot()
        for organism in saved["world"]["ledger"]["organisms"]:
            organism["genome"].pop("brain", None)
        world = Habitat.from_checkpoint_json(json.dumps(saved["world"]))
        packet = {"schema_version": 1, "tick": world.tick, "agents": []}
        legacy = world.propose_controlled(json.dumps(packet))
        packed = world.propose_controlled_packed(json.dumps(packet), np.empty((0, 2), dtype=np.uint64),
            np.empty(0, dtype=np.float64), np.empty((0, 2), dtype=np.uint64), np.empty(0, dtype=np.float64))
        self.assertEqual(legacy.checkpoint_json(), packed.checkpoint_json())

    def test_corrupt_packed_parameters_are_rejected_atomically(self):
        runtime = self.runtime()
        before = runtime.snapshot()
        mutations = [
            lambda p: p[1].__setitem__((0, 0), 2**63 + 1),
            lambda p: p[1].__setitem__((0, 1), 2**63 + 1),
            lambda p: p[3].__setitem__((0, 1), 2**63 + 1),
            lambda p: p[2].__setitem__(0, 1.0),
            lambda p: p[4].__setitem__(0, float("nan")),
            lambda p: p[4].__setitem__(0, 8.01),
            lambda p: p.__setitem__(1, p[1][:, :1]),
            lambda p: p.__setitem__(3, p[3][::-1]),
            lambda p: p.__setitem__(4, p[4][::2]),
        ]
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                packet = self.packet(runtime)
                mutate(packet)
                with self.assertRaises(ValueError):
                    runtime.world.propose_controlled_packed(*packet)
                self.assertEqual(runtime.snapshot(), before)
        for index, dtype in [(1, np.int64), (2, np.float32), (3, np.float64)]:
            packet = self.packet(runtime)
            packet[index] = packet[index].astype(dtype)
            with self.assertRaises(TypeError):
                runtime.world.propose_controlled_packed(*packet)
            self.assertEqual(runtime.snapshot(), before)

    def test_stale_or_ambiguous_action_batches_are_rejected(self):
        runtime = self.runtime()
        for kind in ["tick", "version", "learned"]:
            packet = self.packet(runtime)
            batch = json.loads(packet[0])
            if kind == "tick":
                batch["tick"] += 6
            elif kind == "version":
                batch["agents"][0]["version"] += 1
            else:
                batch["agents"][0]["learned"] = {"weights": [], "biases": []}
            packet[0] = json.dumps(batch)
            before = runtime.world.checkpoint_json()
            with self.assertRaises(ValueError):
                runtime.world.propose_controlled_packed(*packet)
            self.assertEqual(runtime.world.checkpoint_json(), before)

    def test_full_width_organism_ids_preserve_parameter_routing(self):
        runtime = self.runtime(reproduction=False)
        saved = runtime.snapshot()["world"]
        old_ids = [a["id"] for a in runtime._frame["agents"]]
        mapping = dict(zip(old_ids, [2**53 + 1, 2**63 + 3], strict=True))
        for field in ["positions", "motion"]:
            for item in saved[field]:
                item["id"] = mapping.get(item["id"], item["id"])
        for item in saved["ledger"]["organisms"]:
            item["id"] = mapping.get(item["id"], item["id"])
        saved["next_id"] = max(mapping.values()) + 1
        world = Habitat.from_checkpoint_json(json.dumps(saved))
        frame = json.loads(world.agent_frame_json())
        runtime = EcosystemRuntime(world, PlasticPopulation(frame))
        packet = self.packet(runtime)
        actions, _ = runtime.brains.propose(runtime._frame)
        packed = runtime.world.propose_controlled_packed(*packet)
        legacy = runtime.world.propose_controlled(json.dumps(actions))
        self.assertEqual(packed.checkpoint_json(), legacy.checkpoint_json())

    def test_compiled_graphs_do_not_alias_mutable_parameters(self):
        first = self.runtime()
        original = first.brains.weights.clone()
        second = PlasticPopulation(first._frame)
        first.brains.weights += 1
        self.assertTrue(torch.equal(second.weights, original))
        brain = json.loads(json.dumps(first._frame["agents"][0]["brain"]))
        before = compiled_graph(brain)["weights"][0]
        brain["connections"][0]["weight"] += 0.5
        self.assertEqual(compiled_graph(brain)["weights"][0], before + 0.5)

    def test_compact_progress_keeps_totals_and_preserves_saved_histories(self):
        runtime = self.runtime()
        runtime.advance(3)
        before = runtime.snapshot()
        full = runtime.summary()
        compact = runtime.summary(compact=True)
        for key in ["events", "behavior", "energy_history", "death_history"]:
            full["diagnostics"].pop(key, None)
        self.assertEqual(compact, full)
        self.assertEqual(runtime.snapshot(), before)

    def test_cached_audits_match_fresh_validation_after_every_commit(self):
        runtime = self.runtime()
        previous = json.loads(runtime.world.audit_json())["energy"]
        changed = False
        for _ in range(12):
            runtime.advance()
            cached = json.loads(runtime.world.audit_json())
            restored = Habitat.from_checkpoint_json(runtime.world.checkpoint_json())
            self.assertEqual(cached, json.loads(restored.audit_json()))
            changed |= cached["energy"] != previous
            previous = cached["energy"]
        self.assertTrue(changed)

    def test_proposed_world_cannot_invalidate_its_parents_audit(self):
        runtime = self.runtime()
        before = runtime.world.checkpoint_json()
        audit = runtime.world.audit_json()
        candidate = runtime.world.propose_controlled_packed(*self.packet(runtime))
        self.assertEqual(runtime.world.audit_json(), audit)
        self.assertEqual(runtime.world.checkpoint_json(), before)
        restored = Habitat.from_checkpoint_json(candidate.checkpoint_json())
        self.assertEqual(candidate.audit_json(), restored.audit_json())


if __name__ == "__main__":
    unittest.main()
