"""Behavioral checks for the rule contract, including failure atomicity."""

from dataclasses import FrozenInstanceError, replace
import math
import random
import unittest

from reference.ledger import Genome, InvariantViolation, Ledger, Organism, Resources
from reference.rules import load_rules
from reference.worked_example import build_reference_world, check_expected, load_fixture, run_example


class ResourceContractTests(unittest.TestCase):
    def setUp(self):
        self.rules = load_rules()
        self.world = build_reference_world()

    def assertUnchangedFailure(self, operation, exception=ValueError):
        before = self.world.summary("state"), dict(self.world.organisms), self.world.events
        with self.assertRaises(exception):
            operation()
        after = self.world.summary("state"), dict(self.world.organisms), self.world.events
        self.assertEqual(before, after)

    def make_single(self, *, tissue=8.0, energy=20.0, reserve_nutrient=5.0,
                    age=3.0, genome=None, mineral=10.0, light=0.0, detritus=Resources()):
        organism = Organism("solo", genome or Genome(can_ingest=True),
                            Resources(tissue, tissue * self.rules.tissue_energy_per_nutrient),
                            Resources(reserve_nutrient, energy), age=age)
        return Ledger(self.rules, mineral_nutrient=mineral, organisms=[organism],
                      initial_nutrient=mineral + tissue + reserve_nutrient + detritus.nutrient,
                      initial_energy=organism.tissue.energy + energy + detritus.energy,
                      light_available=light, detritus=detritus)

    def test_full_lifecycle_matches_independently_written_expected_snapshots(self):
        world, snapshots = run_example()
        self.assertEqual(len(snapshots), 8)
        for actual, expected in zip(snapshots, load_fixture()["expected_snapshots"], strict=True):
            check_expected(actual, expected, self.rules)
        self.assertAlmostEqual(world.summary("final")["mineral_nutrient"], 88.5)
        self.assertFalse(world.organisms["producer"].alive)
        self.assertTrue(world.organisms["child"].alive)

    def test_initial_inventories_are_checked_instead_of_inferred(self):
        with self.assertRaises(InvariantViolation):
            Ledger(self.rules, mineral_nutrient=75.0, organisms=list(self.world.organisms.values()),
                   initial_nutrient=101.0, initial_energy=100.0)

    def test_duplicate_initial_ids_rejected(self):
        organism = self.world.organisms["producer"]
        with self.assertRaises(ValueError):
            Ledger(self.rules, mineral_nutrient=0, organisms=[organism, organism],
                   initial_nutrient=30, initial_energy=100)

    def test_tissue_energy_density_is_part_of_the_contract(self):
        bad = replace(self.world.organisms["producer"], tissue=Resources(10.0, 21.0))
        with self.assertRaises(InvariantViolation):
            Ledger(self.rules, mineral_nutrient=0, organisms=[bad], initial_nutrient=15, initial_energy=51)

    def test_light_has_a_finite_external_budget_and_capability(self):
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("consumer", 1))
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 11))
        self.world.photosynthesize("producer", 10)
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 1))
        self.assertAlmostEqual(self.world.organisms["producer"].reserves.energy, 38)

    def test_energy_storage_saturation_is_explicit_heat(self):
        self.world = self.make_single(tissue=8, energy=39, genome=Genome(photosynthetic=True), light=10)
        self.world.photosynthesize("solo", 10)
        self.assertAlmostEqual(self.world.organisms["solo"].reserves.energy, 40)
        self.assertAlmostEqual(self.world.summary("after")["heat"], 9)

    def test_absorption_returns_overflow_to_minerals(self):
        self.world = self.make_single(tissue=8, reserve_nutrient=8, genome=Genome(photosynthetic=True))
        self.world.absorb("solo", 1)
        self.assertEqual(self.world.organisms["solo"].reserves.nutrient, 8)
        self.assertEqual(self.world.summary("after")["mineral_nutrient"], 10)

    def test_absorption_requires_available_material_and_rate_capacity(self):
        self.assertUnchangedFailure(lambda: self.world.absorb("producer", 2))
        self.assertUnchangedFailure(lambda: self.world.absorb("consumer", 1))
        self.world = self.make_single(mineral=0, genome=Genome(photosynthetic=True))
        self.assertUnchangedFailure(lambda: self.world.absorb("solo", 1))

    def test_feeding_removes_substrate_and_charges_digestion(self):
        self.world.feed("consumer", "producer", 2, contact=True)
        self.assertEqual(self.world.organisms["producer"].tissue, Resources(8, 16))
        self.assertEqual(self.world.organisms["consumer"].reserves, Resources(3.5, 36.5))
        row = self.world.summary("after")
        self.assertEqual((row["detritus_nutrient"], row["detritus_energy"], row["heat"]), (0.5, 1, 0.5))

    def test_contact_self_feeding_and_excess_bites_are_rejected(self):
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=False))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "consumer", 1, contact=True))
        self.assertUnchangedFailure(lambda: self.world.feed("producer", "consumer", 1, contact=True))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 3, contact=True))

    def test_already_eaten_tissue_cannot_be_claimed_twice(self):
        self.world.feed("consumer", "producer", 10, contact=True, duration=5)
        self.assertFalse(self.world.organisms["producer"].alive)
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=True))

    def test_unaffordable_digestion_leaves_food_untouched(self):
        hungry = replace(self.world.organisms["consumer"], reserves=Resources(2, 0.1))
        producer = self.world.organisms["producer"]
        self.world = Ledger(self.rules, mineral_nutrient=75, organisms=[producer, hungry],
                            initial_nutrient=100, initial_energy=66.1, light_available=10)
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 2, contact=True))

    def test_growth_pays_both_embodied_energy_and_construction_loss(self):
        self.world.grow("consumer", 1)
        self.assertEqual(self.world.organisms["consumer"].tissue, Resources(9, 18))
        self.assertEqual(self.world.organisms["consumer"].reserves, Resources(1, 31))
        self.assertEqual(self.world.summary("after")["heat"], 1)
        self.assertUnchangedFailure(lambda: self.world.grow("consumer", 2))

    def test_growth_cannot_use_light_as_material(self):
        self.world = self.make_single(reserve_nutrient=0, energy=30,
                                       genome=Genome(photosynthetic=True), light=10)
        self.world.photosynthesize("solo", 10)
        self.assertUnchangedFailure(lambda: self.world.grow("solo", 1))

    def test_maintenance_exact_funding_and_age(self):
        self.world = self.make_single(energy=0.8)
        self.world.maintain("solo")
        self.assertTrue(self.world.organisms["solo"].alive)
        self.assertEqual(self.world.organisms["solo"].reserves.energy, 0)
        self.assertEqual(self.world.organisms["solo"].age, 4)
        self.world.maintain("solo")
        self.assertFalse(self.world.organisms["solo"].alive)

    def test_starvation_does_not_use_tissue_energy_as_free_maintenance(self):
        self.world = self.make_single(energy=0.2)
        self.world.maintain("solo")
        organism = self.world.organisms["solo"]
        self.assertFalse(organism.alive)
        self.assertEqual(organism.death_cause, "starvation")
        self.assertEqual(organism.tissue, Resources())
        row = self.world.summary("after")
        self.assertEqual(row["detritus_energy"], 16)
        self.assertAlmostEqual(row["heat"], 0.2)
        self.assertUnchangedFailure(lambda: self.world.maintain("solo"))

    def test_birth_is_funded_and_preserves_inherited_weights(self):
        self.world = self.make_single(genome=Genome(can_ingest=True, initial_weights=(0.25, -0.5)))
        before = self.world.organisms["solo"]
        child = self.world.reproduce("solo", "baby", placement_available=True)
        parent = self.world.organisms["solo"]
        self.assertEqual(parent.reserves, Resources(2.5, 9))
        self.assertEqual(child.tissue, Resources(2, 4))
        self.assertEqual(child.reserves, Resources(0.5, 5))
        self.assertEqual(child.genome, before.genome)
        self.assertEqual(child.genome.initial_weights, (0.25, -0.5))
        self.assertEqual(child.parent_id, "solo")
        self.assertEqual(child.age, 0)
        self.assertFalse(child.mature(self.rules))
        self.assertEqual(self.world.summary("after")["heat"], 2)

    def test_birth_checks_maturity_placement_ids_and_cooldown(self):
        self.world = self.make_single(age=1)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.world = self.make_single(tissue=5)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.world = self.make_single()
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=False))
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "solo", placement_available=True))
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "", placement_available=True))
        self.world.reproduce("solo", "baby", placement_available=True)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "other", placement_available=True))

    def test_unfunded_birth_cannot_create_resources_or_partial_child(self):
        self.world = self.make_single(energy=10)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.assertNotIn("baby", self.world.organisms)
        self.world = self.make_single(reserve_nutrient=2)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))

    def test_damage_transfers_all_resources_once_when_vital_tissue_fails(self):
        self.world.damage("producer", 10)
        row = self.world.summary("after")
        self.assertEqual(row["detritus_nutrient"], 15)
        self.assertEqual(row["detritus_energy"], 50)
        self.assertUnchangedFailure(lambda: self.world.damage("producer", 1))

    def test_exact_vital_threshold_is_alive_and_lower_is_dead(self):
        self.world.damage("producer", 9)
        self.assertTrue(self.world.organisms["producer"].alive)
        self.assertEqual(self.world.organisms["producer"].tissue.nutrient, 1)
        self.assertEqual(self.world.organisms["producer"].reserves, Resources(1, 5))
        self.world.damage("producer", 0.5)
        self.assertFalse(self.world.organisms["producer"].alive)

    def test_decomposition_handles_partial_and_complete_substrate(self):
        self.world.damage("producer", 10)
        self.world.decompose(fraction=0.5)
        row = self.world.summary("partial")
        self.assertEqual(row["detritus_nutrient"], 7.5)
        self.assertEqual(row["detritus_energy"], 25)
        self.assertEqual(row["mineral_nutrient"], 82.5)
        self.assertEqual(row["heat"], 25)
        self.world.decompose(fraction=1)
        self.assertUnchangedFailure(lambda: self.world.decompose(fraction=1))

    def test_decomposition_supports_energy_only_waste_and_limits_processing(self):
        self.world = self.make_single(detritus=Resources(0, 4))
        self.world.decompose(fraction=0.5)
        self.assertEqual(self.world.summary("after")["detritus_energy"], 2)
        self.assertEqual(self.world.summary("after")["heat"], 2)
        self.assertUnchangedFailure(lambda: self.world.decompose(fraction=1, duration=0.001))
        self.assertUnchangedFailure(lambda: self.world.decompose(fraction=2))

    def test_movement_tracks_mechanical_storage_before_dissipation(self):
        self.world.actuate("consumer", 4)
        row = self.world.summary("moving")
        self.assertEqual(row["mechanical_energy"], 2)
        self.assertEqual(row["heat"], 2)
        self.assertEqual(self.world.organisms["consumer"].reserves.energy, 30)
        self.world.dissipate_motion(2)
        row = self.world.summary("stopped")
        self.assertEqual(row["mechanical_energy"], 0)
        self.assertEqual(row["heat"], 4)
        self.assertUnchangedFailure(lambda: self.world.dissipate_motion(1))

    def test_sensing_spends_energy_and_dead_organisms_cannot_act(self):
        self.world.sense("consumer")
        self.assertAlmostEqual(self.world.organisms["consumer"].reserves.energy, 33.98)
        self.world.damage("consumer", 8)
        self.assertUnchangedFailure(lambda: self.world.sense("consumer"))
        self.assertUnchangedFailure(lambda: self.world.actuate("consumer", 1))
        self.assertUnchangedFailure(lambda: self.world.grow("consumer", 1))
        self.assertUnchangedFailure(lambda: self.world.reproduce("consumer", "baby", placement_available=True))

    def test_invalid_numbers_and_durations_cannot_mutate_inventory(self):
        for bad in (-1.0, math.nan, math.inf, -math.inf, True):
            with self.subTest(value=bad):
                self.assertUnchangedFailure(lambda: self.world.actuate("consumer", bad))
                self.assertUnchangedFailure(lambda: self.world.grow("consumer", bad))
                self.assertUnchangedFailure(lambda: self.world.maintain("consumer", duration=bad))
        self.assertUnchangedFailure(lambda: self.world.feed("missing", "producer", 1, contact=True))

    def test_immutable_public_records_and_defensive_history(self):
        held_snapshot = self.world.organisms
        with self.assertRaises(TypeError):
            self.world.organisms["other"] = self.world.organisms["producer"]
        with self.assertRaises(FrozenInstanceError):
            self.world.organisms["consumer"].age = 100
        self.world.sense("consumer")
        self.assertEqual(held_snapshot["consumer"].reserves.energy, 34)
        self.assertAlmostEqual(self.world.organisms["consumer"].reserves.energy, 33.98)
        self.world.events[0]["arguments"]["organism_id"] = "other"
        self.assertEqual(self.world.events[0]["arguments"]["organism_id"], "consumer")

    def test_audit_failure_rolls_back_even_after_tentative_writes(self):
        original = self.world.audit
        before = self.world.summary("state"), dict(self.world.organisms), self.world.events
        calls = 0
        def fail_on_commit():
            nonlocal calls
            calls += 1
            if calls == 2:
                raise InvariantViolation("Injected commit audit failure")
            return original()
        self.world.audit = fail_on_commit
        with self.assertRaises(InvariantViolation):
            self.world.grow("consumer", 1)
        self.assertEqual(calls, 2)
        self.world.audit = original
        after = self.world.summary("state"), dict(self.world.organisms), self.world.events
        self.assertEqual(before, after)
        self.world.audit()

    def test_dead_body_does_not_destroy_existing_mechanical_energy(self):
        self.world.actuate("consumer", 4)
        self.world.damage("consumer", 8)
        self.assertEqual(self.world.summary("after death")["mechanical_energy"], 2)
        self.world.dissipate_motion(2)
        self.assertEqual(self.world.summary("after dissipation")["mechanical_energy"], 0)
        self.world.audit()

    def test_rate_limits_scale_with_the_authorized_integration_window(self):
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 2, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=True, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.grow("consumer", 0.2, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.absorb("producer", 0.2, duration=0.1))
        self.world.grow("consumer", 0.1, duration=0.1)
        self.world.audit()

    def test_seeded_mixed_transactions_preserve_inventory_or_fail_atomically(self):
        rng = random.Random(73)
        for _ in range(200):
            operation = rng.choice((
                lambda: self.world.photosynthesize("producer", rng.uniform(0.01, 3)),
                lambda: self.world.absorb("producer", rng.uniform(0.01, 0.5)),
                lambda: self.world.feed("consumer", "producer", rng.uniform(0.01, 1), contact=True),
                lambda: self.world.grow("producer", rng.uniform(0.01, 0.5)),
                lambda: self.world.grow("consumer", rng.uniform(0.01, 0.5)),
                lambda: self.world.maintain("consumer", duration=0.1),
                lambda: self.world.actuate("consumer", rng.uniform(0.01, 0.5)),
                lambda: self.world.dissipate_motion(rng.uniform(0.01, 0.5)),
                lambda: self.world.decompose(fraction=rng.uniform(0.01, 1)),
            ))
            before = self.world.summary("state"), dict(self.world.organisms), self.world.events
            try:
                operation()
            except ValueError:
                self.assertEqual(before, (self.world.summary("state"), dict(self.world.organisms), self.world.events))
            result = self.world.audit()
            self.assertAlmostEqual(result["nutrient_error"], 0)
            self.assertAlmostEqual(result["energy_error"], 0)


class RuleValidationTests(unittest.TestCase):
    def test_invalid_rules_rejected(self):
        rules = load_rules()
        for update in ({"schema_version": 2}, {"physics_dt": 0}, {"grid_width": 2.5},
                       {"energy_assimilation_fraction": 1.1}, {"movement_efficiency": math.nan},
                       {"offspring_tissue_nutrient": 0.5}, {"offspring_reserve_energy": 100},
                       {"offspring_reserve_nutrient": 3}, {"model_id": ""}):
            with self.subTest(update=update), self.assertRaises(ValueError):
                replace(rules, **update)

    def test_resources_and_genomes_reject_invalid_values(self):
        for operation in (lambda: Resources(-1, 0), lambda: Resources(0, math.nan),
                          lambda: Genome(initial_weights=(math.inf,)),
                          lambda: Genome(initial_weights=[0.1]), lambda: Genome(photosynthetic=1)):
            with self.assertRaises(ValueError):
                operation()


if __name__ == "__main__":
    unittest.main()
