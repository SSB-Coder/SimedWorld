from copy import deepcopy
from dataclasses import asdict
import json
import math
from pathlib import Path
import random
import unittest

from world import ResourceLedger
from world._native import ResourceLedger as RawLedger
from reference.ledger import Genome, Ledger, Organism, Resources
from reference.rules import WorldRules
from reference.worked_example import run_example as run_reference
from examples.native_lifecycle import initial_from_fixture, run_example


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text())
FIXTURE = json.loads((ROOT / "examples/lifecycle_expected.json").read_text())
INITIAL = initial_from_fixture(FIXTURE)
ID_FIELDS = {"organism_id", "consumer_id", "target_id", "parent_id", "child_id"}


def single(*, tissue=8.0, energy=20.0, nutrient=5.0, age=3.0, photosynthetic=False,
           can_ingest=True, weights=None, mineral=10.0, light=0.0, detritus=None):
    detritus = detritus or {"nutrient": 0.0, "energy": 0.0}
    return {"schema_version": 1, "initial_nutrient": mineral + tissue + nutrient + detritus["nutrient"],
            "initial_energy": 2 * tissue + energy + detritus["energy"], "mineral_nutrient": mineral,
            "light_available": light, "detritus": detritus,
            "organisms": [{"id": 7, "genome": {"photosynthetic": photosynthetic, "can_ingest": can_ingest,
                                               "initial_weights": weights or []},
                           "tissue": {"nutrient": tissue, "energy": 2 * tissue},
                           "reserves": {"nutrient": nutrient, "energy": energy}, "age": age}]}


class NativeResourceTests(unittest.TestCase):
    def setUp(self):
        self.pair()

    def pair(self, initial=None, rules=None):
        self.initial = deepcopy(INITIAL if initial is None else initial)
        self.rules = deepcopy(RULES if rules is None else rules)
        self.native = ResourceLedger(self.rules, self.initial)
        organisms = []
        for item in self.initial["organisms"]:
            genome = item["genome"]
            organisms.append(Organism(str(item["id"]), Genome(genome.get("photosynthetic", False),
                genome.get("can_ingest", False), tuple(genome.get("initial_weights", []))),
                Resources(**item["tissue"]), Resources(**item["reserves"]), age=item.get("age", 0),
                alive=item.get("alive", True), parent_id=(str(item["parent_id"]) if item.get("parent_id") is not None else None),
                next_reproduction_age=item.get("next_reproduction_age", 0), death_cause=item.get("death_cause")))
        self.reference = Ledger(WorldRules(**self.rules), mineral_nutrient=self.initial["mineral_nutrient"],
            initial_nutrient=self.initial["initial_nutrient"], initial_energy=self.initial["initial_energy"],
            organisms=organisms, light_available=self.initial.get("light_available", 0),
            detritus=Resources(**self.initial.get("detritus", {"nutrient": 0, "energy": 0})))

    def equal_values(self, actual, expected):
        if isinstance(expected, dict):
            self.assertEqual(actual.keys(), expected.keys())
            for key in expected:
                self.equal_values(actual[key], expected[key])
        elif isinstance(expected, (list, tuple)):
            self.assertEqual(len(actual), len(expected))
            for a, b in zip(actual, expected, strict=True):
                self.equal_values(a, b)
        elif isinstance(expected, (bool, int)) or expected is None or isinstance(expected, str):
            self.assertEqual(actual, expected)
        else:
            self.assertTrue(math.isclose(actual, expected, rel_tol=1e-8, abs_tol=1e-8), (actual, expected))

    def equal_state(self):
        self.equal_values(self.native.summary(), self.reference.summary("state"))
        expected = {}
        for identity, organism in self.reference.organisms.items():
            item = asdict(organism)
            item["id"] = int(item.pop("organism_id"))
            item["parent_id"] = int(item["parent_id"]) if item["parent_id"] is not None else None
            expected[int(identity)] = item
        actual = {item["id"]: item for item in self.native.snapshot()["organisms"]}
        self.equal_values(actual, expected)
        events = self.native.snapshot()["events"]
        self.assertEqual(len(events), len(self.reference.events))
        self.assertEqual([event["sequence"] for event in events], list(range(1, len(events) + 1)))
        self.assertEqual([event["operation"]["kind"] for event in events],
                         [event["operation"] for event in self.reference.events])
        self.equal_values(self.native.audit(), self.reference.audit())

    def both(self, kind, **arguments):
        before_native = self.native.snapshot()
        before_reference = (self.reference.summary("state"), dict(self.reference.organisms), self.reference.events)
        errors = []
        converted = {key: str(value) if key in ID_FIELDS else value for key, value in arguments.items()}
        for operation in (lambda: self.native.apply(kind, **arguments),
                          lambda: getattr(self.reference, kind)(**converted)):
            try:
                operation()
                errors.append(None)
            except (ValueError, RuntimeError) as error:
                errors.append(error)
        self.assertEqual(errors[0] is None, errors[1] is None, (kind, arguments, errors))
        if errors[0] is not None:
            self.assertEqual(self.native.snapshot(), before_native)
            self.assertEqual((self.reference.summary("state"), dict(self.reference.organisms), self.reference.events), before_reference)
        self.equal_state()
        return errors[0] is None

    def test_lifecycle_matches_independent_fixture_and_python_reference(self):
        ledger, actual = run_example()
        _, expected = run_reference()
        self.equal_values(actual, expected)
        self.equal_values(actual, FIXTURE["expected_snapshots"])
        self.assertEqual(len(ledger.snapshot()["events"]), 7)

    def test_photosynthesis_light_budget_and_energy_saturation(self):
        self.assertFalse(self.both("photosynthesize", organism_id=2, light_energy=1))
        self.assertFalse(self.both("photosynthesize", organism_id=1, light_energy=11))
        self.assertTrue(self.both("photosynthesize", organism_id=1, light_energy=10))
        self.assertFalse(self.both("photosynthesize", organism_id=1, light_energy=1))
        self.pair(single(energy=39, nutrient=5, photosynthetic=True, light=10))
        self.both("photosynthesize", organism_id=7, light_energy=10)
        self.assertEqual(self.native.summary()["heat"], 9)

    def test_mineral_uptake_capacity_capability_and_rate(self):
        self.assertFalse(self.both("absorb", organism_id=2, nutrient=1))
        self.assertFalse(self.both("absorb", organism_id=1, nutrient=2))
        self.pair(single(nutrient=8, photosynthetic=True))
        self.assertTrue(self.both("absorb", organism_id=7, nutrient=1))
        self.assertEqual(self.native.summary()["mineral_nutrient"], 10)
        self.pair(single(photosynthetic=True, mineral=0))
        self.assertFalse(self.both("absorb", organism_id=7, nutrient=1))

    def test_feeding_assimilation_prepayment_and_repeat_claims(self):
        self.assertTrue(self.both("feed", consumer_id=2, target_id=1, tissue_nutrient=2, contact=True))
        self.assertEqual(self.native.summary()["detritus_nutrient"], 0.5)
        for arguments in [dict(consumer_id=2, target_id=1, tissue_nutrient=1, contact=False),
                          dict(consumer_id=2, target_id=2, tissue_nutrient=1, contact=True),
                          dict(consumer_id=1, target_id=2, tissue_nutrient=1, contact=True),
                          dict(consumer_id=2, target_id=1, tissue_nutrient=3, contact=True)]:
            self.assertFalse(self.both("feed", **arguments))
        self.pair()
        self.assertTrue(self.both("feed", consumer_id=2, target_id=1, tissue_nutrient=10, duration=5, contact=True))
        self.assertFalse(self.both("feed", consumer_id=2, target_id=1, tissue_nutrient=1, contact=True))
        initial = deepcopy(INITIAL)
        initial["organisms"][1]["reserves"]["energy"] = 0.1
        initial["initial_energy"] = 66.1
        self.pair(initial)
        self.assertFalse(self.both("feed", consumer_id=2, target_id=1, tissue_nutrient=2, contact=True))

    def test_growth_requires_reserves_and_retains_embodied_energy(self):
        self.assertTrue(self.both("grow", organism_id=2, tissue_nutrient=1))
        organism = self.native.snapshot()["organisms"][1]
        self.assertEqual(organism["tissue"], {"nutrient": 9, "energy": 18})
        self.assertEqual(organism["reserves"], {"nutrient": 1, "energy": 31})
        self.assertFalse(self.both("grow", organism_id=2, tissue_nutrient=2))
        self.pair(single(nutrient=0, energy=30, photosynthetic=True, light=10))
        self.both("photosynthesize", organism_id=7, light_energy=10)
        self.assertFalse(self.both("grow", organism_id=7, tissue_nutrient=1))

    def test_exact_maintenance_then_starvation_commits_death(self):
        self.pair(single(energy=0.8))
        self.assertTrue(self.both("maintain", organism_id=7))
        self.assertTrue(self.native.snapshot()["organisms"][0]["alive"])
        self.assertTrue(self.both("maintain", organism_id=7))
        organism = self.native.snapshot()["organisms"][0]
        self.assertFalse(organism["alive"])
        self.assertEqual(organism["death_cause"], "starvation")
        self.assertEqual(self.native.summary()["detritus_energy"], 16)
        self.assertFalse(self.both("maintain", organism_id=7))

    def test_birth_debits_parent_preserves_genome_and_records_ancestry(self):
        self.pair(single(weights=[0.25, -0.5]))
        self.assertTrue(self.both("reproduce", parent_id=7, child_id=2**53 + 1, placement_available=True))
        parent, child = self.native.snapshot()["organisms"]
        self.assertEqual(parent["reserves"], {"nutrient": 2.5, "energy": 9})
        self.assertEqual(child["genome"], parent["genome"])
        self.assertEqual(child["parent_id"], 7)
        self.assertEqual(child["age"], 0)
        self.assertEqual(child["tissue"], {"nutrient": 2, "energy": 4})
        self.assertEqual(self.native.summary()["heat"], 2)
        self.assertFalse(self.both("reproduce", parent_id=7, child_id=9, placement_available=True))

    def test_birth_rejects_unfunded_immature_duplicate_and_unplaced_requests(self):
        for initial in [single(age=1), single(tissue=5), single(energy=10), single(nutrient=2)]:
            self.pair(initial)
            self.assertFalse(self.both("reproduce", parent_id=7, child_id=8, placement_available=True))
        self.pair(single())
        self.assertFalse(self.both("reproduce", parent_id=7, child_id=7, placement_available=True))
        self.assertFalse(self.both("reproduce", parent_id=7, child_id=8, placement_available=False))

    def test_damage_viability_capacity_reduction_and_single_death_transfer(self):
        self.assertTrue(self.both("damage", organism_id=1, tissue_nutrient=9))
        self.assertTrue(self.native.snapshot()["organisms"][0]["alive"])
        self.assertEqual(self.native.snapshot()["organisms"][0]["reserves"], {"nutrient": 1, "energy": 5})
        self.assertTrue(self.both("damage", organism_id=1, tissue_nutrient=0.5))
        self.assertFalse(self.both("damage", organism_id=1, tissue_nutrient=1))
        for kind, arguments in [("sense", {}), ("actuate", {"energy": 1}),
                                ("grow", {"tissue_nutrient": 1})]:
            self.assertFalse(self.both(kind, organism_id=1, **arguments))

    def test_decomposition_preserves_nutrients_and_handles_energy_only_waste(self):
        self.both("damage", organism_id=1, tissue_nutrient=10)
        self.both("decompose", fraction=0.5)
        self.assertEqual(self.native.summary()["detritus_nutrient"], 7.5)
        self.assertEqual(self.native.summary()["mineral_nutrient"], 82.5)
        self.both("decompose", fraction=1)
        self.assertFalse(self.both("decompose", fraction=1))
        self.pair(single(detritus={"nutrient": 0, "energy": 4}))
        self.both("decompose", fraction=0.5)
        self.assertEqual(self.native.summary()["heat"], 2)
        self.assertFalse(self.both("decompose", fraction=1, duration=0.001))
        self.assertFalse(self.both("decompose", fraction=2))

    def test_sensing_and_mechanical_energy_survive_actor_death(self):
        self.both("sense", organism_id=2)
        self.both("actuate", organism_id=2, energy=4)
        self.assertEqual(self.native.summary()["mechanical_energy"], 2)
        self.both("damage", organism_id=2, tissue_nutrient=8)
        self.assertEqual(self.native.summary()["mechanical_energy"], 2)
        self.both("dissipate_motion", energy=2)
        self.assertEqual(self.native.summary()["mechanical_energy"], 0)
        self.assertFalse(self.both("dissipate_motion", energy=1))

    def test_invalid_values_and_small_process_windows_leave_no_partial_state(self):
        for bad in [-1, 0, math.nan, math.inf, -math.inf, True]:
            for kind, arguments in [("actuate", {"energy": bad}), ("grow", {"tissue_nutrient": bad}),
                                    ("maintain", {"duration": bad})]:
                self.assertFalse(self.both(kind, organism_id=2, **arguments))
        for kind, arguments in [("photosynthesize", {"organism_id": 1, "light_energy": 2}),
                                ("feed", {"consumer_id": 2, "target_id": 1, "tissue_nutrient": 1, "contact": True}),
                                ("grow", {"organism_id": 2, "tissue_nutrient": 0.2}),
                                ("absorb", {"organism_id": 1, "nutrient": 0.2})]:
            self.assertFalse(self.both(kind, duration=0.1, **arguments))
        self.assertTrue(self.both("grow", organism_id=2, tissue_nutrient=0.1, duration=0.1))

    def test_initial_inventory_and_organism_invariants_are_enforced(self):
        for change in [lambda item: item.update(initial_nutrient=101),
                       lambda item: item["organisms"][0]["tissue"].update(energy=21),
                       lambda item: item["organisms"][0].update(alive=False),
                       lambda item: item["organisms"][0].update(death_cause="damage"),
                       lambda item: item["organisms"][0]["reserves"].update(energy=100)]:
            initial = deepcopy(INITIAL)
            change(initial)
            with self.assertRaises(RuntimeError):
                ResourceLedger(RULES, initial)
        initial = deepcopy(INITIAL)
        initial["organisms"].append(deepcopy(initial["organisms"][0]))
        with self.assertRaises(ValueError):
            ResourceLedger(RULES, initial)

    def test_invalid_rule_definitions_are_rejected(self):
        for update in [{"schema_version": 2}, {"physics_dt": 0}, {"grid_width": 2.5},
                       {"energy_assimilation_fraction": 1.1}, {"movement_efficiency": math.nan},
                       {"offspring_tissue_nutrient": 0.5}, {"offspring_reserve_energy": 100},
                       {"offspring_reserve_nutrient": 3}, {"model_id": ""}, {"unknown": 1}]:
            with self.subTest(update=update), self.assertRaises((ValueError, TypeError)):
                ResourceLedger({**RULES, **update}, INITIAL)

    def test_raw_protocol_rejects_unknown_fields_bad_ids_and_nonfinite_json(self):
        native = RawLedger(json.dumps(RULES), json.dumps(INITIAL))
        before = native.snapshot_json()
        for text in ['{"schema_version":1,"operation":{"kind":"sense","organism_id":true}}',
                     '{"schema_version":1,"operation":{"kind":"sense","organism_id":1.0}}',
                     '{"schema_version":1,"operation":{"kind":"sense","organism_id":1,"extra":0}}',
                     '{"schema_version":1,"operation":{"kind":"actuate","organism_id":2,"energy":NaN}}',
                     '{"schema_version":2,"operation":{"kind":"sense","organism_id":1}}',
                     '{"schema_version":1,"operation":{"kind":"reproduce","parent_id":2,"child_id":-1,"placement_available":true}}']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                native.apply_json(text)
            self.assertEqual(native.snapshot_json(), before)

    def test_receipts_and_snapshot_copies_are_independent(self):
        self.both("grow", organism_id=2, tissue_nutrient=1)
        snapshot = self.native.snapshot()
        event = snapshot["events"][0]
        self.assertEqual(event["organisms"][0]["before"]["tissue"]["nutrient"], 8)
        self.assertEqual(event["organisms"][0]["after"]["tissue"]["nutrient"], 9)
        self.assertEqual(event["pools_after"]["heat"] - event["pools_before"]["heat"], 1)
        snapshot["events"][0]["operation"]["organism_id"] = 999
        snapshot["organisms"][0]["reserves"]["energy"] = 999
        self.equal_state()

    def test_seeded_mixed_requests_match_reference_and_preserve_rejected_state(self):
        rng = random.Random(73)
        for _ in range(200):
            kind, arguments = rng.choice([
                ("photosynthesize", {"organism_id": 1, "light_energy": rng.uniform(0.01, 3)}),
                ("absorb", {"organism_id": 1, "nutrient": rng.uniform(0.01, 0.5)}),
                ("feed", {"consumer_id": 2, "target_id": 1, "tissue_nutrient": rng.uniform(0.01, 1), "contact": True}),
                ("grow", {"organism_id": 1, "tissue_nutrient": rng.uniform(0.01, 0.5)}),
                ("grow", {"organism_id": 2, "tissue_nutrient": rng.uniform(0.01, 0.5)}),
                ("maintain", {"organism_id": 2, "duration": 0.1}),
                ("actuate", {"organism_id": 2, "energy": rng.uniform(0.01, 0.5)}),
                ("dissipate_motion", {"energy": rng.uniform(0.01, 0.5)}),
                ("decompose", {"fraction": rng.uniform(0.01, 1)}),
            ])
            self.both(kind, **arguments)


if __name__ == "__main__":
    unittest.main()
