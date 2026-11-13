from copy import deepcopy
import json
import math
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import torch

from world._native import Habitat as NativeHabitat
from world.brains.evolving import NeuralPopulation, reference_update
from world.brains.candidates import LiquidCandidate
from world.control import RunControl
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import compact_preset, fund
from world.presets import founder_genome, neural_blueprint


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text())


def setup(seed=73, mutate=False):
    rules, config = compact_preset(RULES, seed)
    config["producers"]["count"] = 4
    config["populations"][0]["count"] = 2
    config["populations"][1]["count"] = 1
    config["ecology"]["max_population"] = 32
    if not mutate:
        config["evolution"]["mutation"].update(parameter_edits=0, body_edits=0, body_structure_rate=0, neural_structure_rate=0)
    fund(rules, config)
    return rules, config


def actions(frame, requests=None):
    requests = requests or {}
    return {"schema_version": 1, "tick": frame["tick"], "agents": [
        {"id": a["id"], "version": a["version"], "values": [
            {"port": p["port"], "value": requests.get((a["id"], p["channel"]), 0.0) if p["active"] else 0.0}
            for p in a["outputs"]]} for a in frame["agents"]]}


def contact_world():
    rules, config = setup()
    rules.update(world_width=14, world_height=12, grid_width=14, grid_height=12)
    producer = founder_genome("producer")
    producer["brain"] = neural_blueprint(producer["body"], hidden=0, kind="producer")
    consumer = founder_genome("grazer")
    consumer["body"]["modules"] = consumer["body"]["modules"][:2]
    consumer["body"]["next_id"] = 3
    consumer["brain"] = neural_blueprint(consumer["body"], hidden=0)
    config["producers"].update(count=1, reserve_nutrient=2.0, positions=[{"x": 7.0, "y": 6.0}], genome=producer)
    config["populations"] = [{"count": 1, "tissue_nutrient": 6, "reserve_nutrient": 0.5,
                               "reserve_energy": 20, "positions": [{"x": 3.0, "y": 6.0}], "genome": consumer}]
    config.update(mineral_nutrient_per_cell=0, light_energy_per_cell_per_time=0)
    config["ecology"].update(reproduction=False, decomposition=False)
    fund(rules, config)
    runtime = EcosystemRuntime.create(rules, config)
    payload = json.loads(runtime.world.checkpoint_json())
    for motion in payload["motion"]:
        motion["heading"] = 0.0
    def scale(genome, tissue):
        return math.sqrt(tissue / sum(math.pi * m["radius"]**2 * 12 for m in genome["body"]["modules"]))
    producer_r = 0.30 * scale(producer, 8)
    consumer_r = (0.27 + 2 * 0.16) * scale(consumer, 6)
    payload["positions"][1]["position"]["x"] = 7 - producer_r - consumer_r - 0.01
    world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
    return world


class EvolutionTests(unittest.TestCase):
    def test_existing_and_new_resource_budgets_are_funded(self):
        rules, config = setup()
        runtime = EcosystemRuntime.create(rules, config)
        audit = runtime.audit()
        self.assertAlmostEqual(audit["nutrient"], config["initial_nutrient"])
        self.assertAlmostEqual(audit["energy"], config["initial_energy"])
        self.assertTrue(runtime.frame()["agents"])
        with self.assertRaises(ValueError):
            runtime.world.advance()

    def test_sparse_batch_matches_an_independent_scalar_reference(self):
        runtime = EcosystemRuntime.create(*setup())
        for _ in range(3):
            frame = runtime.frame()
            _, proposed = runtime.brains.propose(frame)
            for agent in frame["agents"]:
                previous = {node: runtime.brains.memory[index].item() for (identity, node), index in runtime.brains.index.items() if identity == agent["id"]}
                expected = reference_update(agent, previous)
                for node, value in expected.items():
                    self.assertAlmostEqual(proposed[runtime.brains.index[(agent["id"], node)]].item(), value, places=13)
            runtime.advance()

    def test_neural_parameters_are_inherited_and_newborn_memory_resets(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(22)
        state = runtime.snapshot()
        population = {o["id"]: o for o in state["world"]["ledger"]["organisms"]}
        children = [o for o in population.values() if o["parent_id"] is not None]
        self.assertTrue(children)
        for child in children:
            self.assertEqual(child["genome"], population[child["parent_id"]]["genome"])
            if child["age"] == 0 and child["genome"].get("brain"):
                for (identity, _), index in runtime.brains.index.items():
                    if identity == child["id"]:
                        self.assertEqual(runtime.brains.memory[index].item(), 0)
        runtime.audit()

    def test_birth_mutation_records_variation_and_preserves_ancestors(self):
        runtime = EcosystemRuntime.create(*setup(mutate=True))
        initial = {o["id"]: deepcopy(o["genome"]) for o in runtime.snapshot()["world"]["ledger"]["organisms"]}
        runtime.advance(25)
        state = runtime.snapshot()
        for o in state["world"]["ledger"]["organisms"]:
            if o["id"] in initial:
                self.assertEqual(o["genome"], initial[o["id"]])
        self.assertGreater(state["world"]["diagnostics"]["accepted_mutations"], 0)
        self.assertTrue(any(e["kind"] == "birth" and e["edits"] for e in state["world"]["diagnostics"]["events"]))

    def test_full_checkpoint_preserves_neural_memory_and_future_births(self):
        runtime = EcosystemRuntime.create(*setup(mutate=True))
        runtime.advance(17)
        resumed = EcosystemRuntime.from_snapshot(runtime.snapshot())
        self.assertEqual(runtime.snapshot(), resumed.snapshot())
        for count in [1, 5, 17]:
            runtime.advance(count)
            resumed.advance(count)
            self.assertEqual(runtime.snapshot(), resumed.snapshot())

    def test_native_and_cross_language_failures_preserve_world_and_memory(self):
        runtime = EcosystemRuntime.create(*setup())
        before = runtime.snapshot()
        frame = runtime.frame()
        invalid = actions(frame)
        invalid["agents"][0]["version"] += 1
        with self.assertRaises(ValueError):
            runtime.world.advance_controlled(json.dumps(invalid))
        self.assertEqual(runtime.snapshot(), before)
        with patch.object(NeuralPopulation, "reconcile", side_effect=ValueError("injected controller migration failure")):
            with self.assertRaises(ValueError):
                runtime.advance()
        self.assertEqual(runtime.snapshot(), before)

    def test_duplicate_unknown_nonfinite_and_inactive_actions_are_rejected(self):
        runtime = EcosystemRuntime.create(*setup())
        baseline = runtime.snapshot()
        frame = runtime.frame()
        variants = []
        value = actions(frame); value["agents"].append(deepcopy(value["agents"][0])); variants.append(value)
        value = actions(frame); value["agents"][0]["id"] = 999; variants.append(value)
        value = actions(frame); value["agents"][0]["values"][0]["value"] = 2; variants.append(value)
        value = actions(frame); value["agents"][0]["values"].append(deepcopy(value["agents"][0]["values"][0])); variants.append(value)
        value = actions(frame); next(v for v in value["agents"][0]["values"] if v["port"] % 256 == 102)["value"] = 1; variants.append(value)
        for value in variants:
            with self.subTest(value=value), self.assertRaises(ValueError):
                runtime.world.advance_controlled(json.dumps(value))
            self.assertEqual(runtime.snapshot(), baseline)

    def test_malformed_coordinated_checkpoints_are_rejected(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance()
        baseline = runtime.snapshot()
        changes = [lambda p: p.update(extra=1), lambda p: p.update(kind="wrong"),
                   lambda p: p["neural"].update(tick=0), lambda p: p["neural"]["memory"].append(0),
                   lambda p: p["neural"]["memory"].__setitem__(0, True),
                   lambda p: p["neural"]["keys"][0].__setitem__(0, 999),
                   lambda p: p["world"]["ledger"]["organisms"][0]["body"]["modules"][0].update(nutrient=999)]
        for change in changes:
            value = deepcopy(baseline); change(value)
            with self.subTest(change=change), self.assertRaises((ValueError, RuntimeError)):
                EcosystemRuntime.from_snapshot(value)

    def test_topology_migration_preserves_survivors_and_initializes_only_new_nodes(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        runtime.brains.memory.fill_(0.25)
        changed = deepcopy(frame)
        agent = changed["agents"][0]
        new_id = agent["brain"]["next_node"]
        agent["brain"]["nodes"].append({"id": new_id, "kind": "hidden", "port": None, "bias": 0, "leak": 0.5})
        agent["brain"]["next_node"] += 1
        agent["brain"]["version"] += 1
        agent["version"] += 1
        changed["tick"] += 6
        child = deepcopy(changed["agents"][1]); child["id"] = 1000; changed["agents"].append(child)
        migrated = runtime.brains.reconcile(changed, runtime.brains.memory)
        for key, index in migrated.index.items():
            expected = 0.25 if key in runtime.brains.index else 0.0
            self.assertEqual(migrated.memory[index].item(), expected)
        self.assertEqual(migrated.memory[migrated.index[(agent["id"], new_id)]].item(), 0)

    def test_heterogeneous_and_thousand_neuron_graphs_execute_without_dense_padding(self):
        rules, config = setup()
        config["evolution"]["mutation"]["max_nodes"] = 4096
        config["populations"][0].update(count=1, genome=founder_genome("grazer", hidden=1000))
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        population = runtime.brains
        total_nodes = sum(len(a["brain"]["nodes"]) for a in runtime.frame()["agents"])
        self.assertEqual(population.memory.numel(), total_nodes)
        self.assertGreater(total_nodes, 1000)
        self.assertEqual(population.sources.numel(), sum(len(a["brain"]["connections"]) for a in runtime.frame()["agents"]))
        runtime.advance()
        runtime.audit()

    def test_zero_neuron_population_runs_without_scripted_foraging(self):
        rules, config = setup()
        for group in config["populations"]:
            group["genome"].pop("brain", None)
        runtime = EcosystemRuntime.create(rules, config)
        before = runtime.snapshot()["world"]["positions"]
        self.assertEqual(runtime.frame()["agents"], [])
        runtime.advance(2)
        self.assertEqual(before, runtime.snapshot()["world"]["positions"])
        self.assertEqual(runtime.summary()["ecology"]["distance_moved"], 0)

    def test_contact_damage_is_local_and_repair_is_resource_funded(self):
        world = contact_world()
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(json.dumps(actions(frame, {(2, 121): 1.0})))
        first = json.loads(world.checkpoint_json())
        target = first["ledger"]["organisms"][0]
        self.assertAlmostEqual(target["tissue"]["nutrient"], 7.9)
        self.assertLess(target["body"]["modules"][0]["nutrient"], target["body"]["modules"][0]["developed"])
        self.assertAlmostEqual(first["diagnostics"]["damage_nutrient"], 0.1)
        self.assertAlmostEqual(first["ledger"]["pools"]["detritus"]["nutrient"], 0.1)
        nutrient_before = target["reserves"]["nutrient"]
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(json.dumps(actions(frame, {(1, 101): 1.0})))
        second = json.loads(world.checkpoint_json())
        self.assertAlmostEqual(second["ledger"]["organisms"][0]["tissue"]["nutrient"], 7.95)
        self.assertAlmostEqual(second["ledger"]["organisms"][0]["reserves"]["nutrient"], nutrient_before - 0.05)
        self.assertAlmostEqual(second["diagnostics"]["costs"]["repair"], 0.15)
        world.audit_json()

    def test_biting_routes_local_waste_and_preserves_resource_accounting(self):
        world = contact_world()
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(json.dumps(actions(frame, {(2, 120): 1.0})))
        payload = json.loads(world.checkpoint_json())
        self.assertAlmostEqual(payload["ledger"]["organisms"][0]["tissue"]["nutrient"], 7.8)
        self.assertAlmostEqual(payload["ledger"]["organisms"][1]["reserves"]["nutrient"], 0.65)
        self.assertAlmostEqual(payload["ledger"]["pools"]["detritus"]["nutrient"], 0.05)
        self.assertEqual(payload["totals"]["feeding_events"], 1)
        self.assertAlmostEqual(payload["diagnostics"]["costs"]["digestion"], 0.05)
        world.audit_json()

    def test_growth_preserves_absolute_injury_instead_of_repairing_for_free(self):
        world = contact_world()
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(2, 121): 1.0})))
        before = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
        missing = sum(m["developed"] - m["nutrient"] for m in before["body"]["modules"])
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(1, 100): 1.0})))
        after = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
        self.assertGreater(after["tissue"]["nutrient"], before["tissue"]["nutrient"])
        self.assertAlmostEqual(sum(m["developed"] - m["nutrient"] for m in after["body"]["modules"]), missing)

    def test_storage_capacity_is_coupled_to_funded_functional_tissue(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        target = payload["ledger"]["organisms"][0]
        module = {"id": 3, "parent": 1, "kind": "storage", "radius": 0.13, "angle": -math.pi / 2,
                  "orientation": 0.0, "investment": 1.0}
        target["genome"]["body"]["modules"].append(module)
        target["genome"]["body"]["next_id"] = 4
        genes = target["genome"]["body"]["modules"]
        total = sum(m["radius"]**2 for m in genes)
        target["body"]["modules"] = [{"id": m["id"], "nutrient": 8 * m["radius"]**2 / total,
                                        "developed": 8 * m["radius"]**2 / total} for m in genes]
        target["reserves"].update(nutrient=8.5, energy=42)
        payload["config"]["producers"].update(genome=deepcopy(target["genome"]), reserve_nutrient=8.5, reserve_energy=42)
        for owner in [payload["ledger"], payload["config"]]:
            owner["initial_nutrient"] += 6.5
            owner["initial_energy"] += 17
        world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()))))
        target = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
        self.assertAlmostEqual(target["reserves"]["nutrient"], 8.5)
        self.assertGreater(target["reserves"]["energy"], 40)
        world.audit_json()

    def test_unreachable_attacks_cannot_damage_remote_organisms(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        payload["positions"][1]["position"] = {"x": 3.0, "y": 6.0}
        world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(2, 121): 1.0})))
        payload = json.loads(world.checkpoint_json())
        self.assertEqual(payload["diagnostics"]["attack_hits"], 0)
        self.assertEqual(payload["ledger"]["organisms"][0]["tissue"]["nutrient"], 8)

    def test_diagnostic_counter_overflow_rejects_the_whole_window(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        payload["diagnostics"]["encounters"] = 2**64 - 1
        world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
        before = world.checkpoint_json()
        with self.assertRaises(ValueError):
            world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(2, 120): 1.0})))
        self.assertEqual(world.checkpoint_json(), before)

    def test_liquid_candidates_use_the_same_ports_and_preserve_inherited_parameters(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        agent = frame["agents"][0]
        for family in ["cfc", "ltc"]:
            with self.subTest(family=family):
                model = LiquidCandidate(family, len(agent["inputs"]), len(agent["outputs"]))
                values, state = model.propose(agent, frame["duration"])
                model.memory = state
                payload = model.snapshot()
                restored = LiquidCandidate.restore(payload)
                newborn = LiquidCandidate.restore(payload, newborn=True)
                self.assertIsNone(newborn.memory)
                for key, value in model.model.state_dict().items():
                    self.assertTrue(torch.equal(value, newborn.model.state_dict()[key]))
                self.assertEqual(model.propose(agent, frame["duration"])[0], restored.propose(agent, frame["duration"])[0])
                batch = actions(frame)
                batch["agents"][0]["values"] = values
                candidate = runtime.world.fork()
                candidate.advance_controlled(json.dumps(batch))
                candidate.audit_json()

    def test_observer_controls_save_load_pause_and_failed_load_are_transactional(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory) / "checkpoint.json"
            control = RunControl(runtime, checkpoint, Path(directory) / "stream.json", paused=True)
            command_number = 0
            def command(kind, path=""):
                nonlocal command_number
                command_number += 1
                return {"schema_version": 1, "session": control.session, "id": f"{kind}-{command_number}", "kind": kind, "path": path}
            self.assertFalse(control.update())
            control.apply(command("advance")); self.assertTrue(control.update()); self.assertTrue(control.paused)
            control.apply(command("save")); before = control.runtime.snapshot()
            control.apply(command("advance")); control.update()
            self.assertGreater(control.runtime.tick, before["world"]["clock"]["tick"])
            control.apply(command("load")); self.assertEqual(control.runtime.snapshot(), before)
            checkpoint.write_text("invalid")
            with self.assertRaises(ValueError):
                control.apply(command("load"))
            self.assertEqual(control.runtime.snapshot(), before)
            control.publish()
            self.assertEqual(json.loads((Path(directory) / "stream.json").read_text())["state"], before)

    def test_realtime_pacing_respects_speed_pause_and_single_decisions(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            control = RunControl(runtime, Path(directory) / "checkpoint.json")
            def command(kind, identity):
                return {"schema_version": 1, "session": control.session, "id": identity, "kind": kind, "path": ""}
            self.assertTrue(control.update(now=0.0))
            self.assertFalse(control.update(now=0.05))
            self.assertTrue(control.update(now=0.1))
            for index in range(4):
                control.apply(command("faster", f"speed-{index}"))
            self.assertEqual(control.speed, 16)
            self.assertTrue(control.update(now=0.101))
            self.assertFalse(control.update(now=0.105))
            self.assertTrue(control.update(now=0.108))
            control.apply(command("pause", "pause"))
            tick = runtime.tick
            self.assertFalse(control.update(now=5.0))
            control.apply(command("advance", "single"))
            self.assertTrue(control.update(now=5.0))
            self.assertFalse(control.update(now=5.0))
            self.assertEqual(runtime.tick, tick + 6)
            control.apply(command("pause", "resume"))
            self.assertTrue(control.update(now=5.001))
            self.assertFalse(control.update(now=5.002))

    def test_published_speed_reports_elapsed_simulation_time(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            stream = Path(directory) / "stream.json"
            control = RunControl(runtime, Path(directory) / "checkpoint.json", stream)
            control.speed = 16
            control._rate_wall, control._rate_time = 0.0, 0.0
            runtime.advance(2)
            with patch("world.control.time.perf_counter", return_value=0.5):
                control.publish()
            observed = json.loads(stream.read_text())
            self.assertEqual(observed["speed"], 16)
            self.assertAlmostEqual(observed["actual_speed"], 0.4)
            self.assertEqual(observed["state"], runtime.snapshot())

    @unittest.skipUnless(torch.cuda.is_available(), "CUDA is unavailable")
    def test_sparse_cuda_proposal_agrees_with_cpu(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        cpu, _ = runtime.brains.propose(frame)
        gpu, _ = NeuralPopulation(frame, "cuda").propose(frame)
        for a, b in zip(cpu["agents"], gpu["agents"], strict=True):
            for x, y in zip(a["values"], b["values"], strict=True):
                self.assertAlmostEqual(x["value"], y["value"], places=10)


if __name__ == "__main__":
    unittest.main()
