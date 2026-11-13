from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest

from world import Habitat
from world._native import Habitat as NativeHabitat


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text(encoding="utf-8"))


def rules():
    return {**RULES, "world_width": 4.0, "world_height": 2.0, "grid_width": 4, "grid_height": 2,
            "physics_dt": 1.0, "ecology_interval_ticks": 1, "minimum_reproduction_age": 1.0}


def population(tissue, nutrient, energy, positions):
    return {"count": len(positions), "tissue_nutrient": tissue, "reserve_nutrient": nutrient,
            "reserve_energy": energy, "positions": [{"x": x, "y": y} for x, y in positions]}


def config(producers=None, consumers=None, minerals=0.0, light=0.0, **ecology):
    producers = producers or population(6, 2.5, 12, [(1, 1)])
    groups = [producers] + ([consumers] if consumers else [])
    return {"schema_version": 1, "seed": 73,
            "initial_nutrient": minerals * 8 + sum(p["count"]*(p["tissue_nutrient"]+p["reserve_nutrient"]) for p in groups),
            "initial_energy": sum(p["count"]*(2*p["tissue_nutrient"]+p["reserve_energy"]) for p in groups),
            "mineral_nutrient_per_cell": minerals, "light_energy_per_cell_per_time": light,
            "light_gradient": 0.0, "producers": producers, "consumers": consumers,
            "ecology": {"reproduction": True, "decomposition": True, "consumer_speed": 0,
                        "minimum_spacing": 0.1, "feeding_radius": 0.25, "birth_radius": 0.5,
                        "max_population": 32, **ecology}}


def payload(habitat):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "checkpoint.json"
        habitat.save(path)
        return json.loads(path.read_text(encoding="utf-8"))


def restore(value):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "checkpoint.json"
        path.write_text(json.dumps(value, allow_nan=False), encoding="utf-8")
        return Habitat.load(path)


class EcosystemTests(unittest.TestCase):
    def test_birth_is_parent_funded_and_newborn_waits_until_next_window(self):
        habitat = Habitat(rules(), config())
        habitat.advance()
        organisms = [item["organism"] for item in habitat.snapshot()["organisms"]]
        parent, child = organisms
        self.assertEqual(habitat.summary()["ecology"]["births"], 1)
        self.assertEqual(parent["reserves"]["nutrient"], 0)
        self.assertAlmostEqual(parent["reserves"]["energy"], 0.4)
        self.assertEqual(child["tissue"], {"nutrient": 2, "energy": 4})
        self.assertEqual(child["reserves"], {"nutrient": 0.5, "energy": 5})
        self.assertEqual(child["parent_id"], parent["id"])
        self.assertEqual(child["age"], 0)
        self.assertEqual(parent["age"], 1)
        self.assertAlmostEqual(habitat.summary()["heat"], 2.6)
        self.assertEqual(habitat.audit()["nutrient"], 8.5)
        self.assertAlmostEqual(habitat.audit()["energy"], 24)

    def test_blocked_population_cap_spends_no_birth_investment(self):
        habitat = Habitat(rules(), config(max_population=1))
        habitat.advance()
        parent = habitat.snapshot()["organisms"][0]["organism"]
        self.assertEqual(habitat.summary()["ecology"]["births"], 0)
        self.assertEqual(parent["reserves"]["nutrient"], 2.5)
        self.assertAlmostEqual(parent["reserves"]["energy"], 11.4)

    def test_failed_placement_is_unfunded_and_does_not_create_an_id(self):
        habitat = Habitat(rules(), config(birth_radius=100))
        habitat.advance()
        self.assertEqual(habitat.summary()["living_count"], 1)
        self.assertEqual(habitat.snapshot()["ecology"]["next_id"], 2)
        self.assertAlmostEqual(habitat.snapshot()["organisms"][0]["organism"]["reserves"]["energy"], 11.4)

    def test_cooldown_prevents_repeated_births(self):
        habitat = Habitat({**rules(), "reproduction_cooldown": 10},
                          config(population(10, 8, 40, [(1, 1)])))
        habitat.advance(2)
        self.assertEqual(habitat.summary()["ecology"]["births"], 1)
        self.assertEqual(habitat.snapshot()["organisms"][0]["organism"]["next_reproduction_age"], 11)

    def test_clone_weights_and_large_offspring_ids_survive_checkpoint(self):
        value = payload(Habitat(rules(), config()))
        value["ledger"]["organisms"][0]["genome"]["initial_weights"] = [0.25, -0.5]
        value["next_id"] = 2**53 + 1
        habitat = restore(value)
        habitat.advance()
        parent, child = [item["organism"] for item in habitat.snapshot()["organisms"]]
        self.assertEqual(child["id"], 2**53 + 1)
        self.assertEqual(child["genome"], parent["genome"])
        self.assertEqual(child["genome"]["initial_weights"], [0.25, -0.5])
        self.assertEqual(restore(payload(habitat)).snapshot(), habitat.snapshot())

    def feeding_case(self, tissue=2, reserve_nutrient=1, reserve_energy=2, speed=0):
        cfg = config(population(tissue, reserve_nutrient, reserve_energy, [(0.99, 0.5)]),
                     population(6, 0, 10, [(1.01, 0.5)]), consumer_speed=speed,
                     decomposition=False)
        world_rules = {**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}
        return Habitat(world_rules, cfg)

    def test_cross_cell_feeding_keeps_carcass_and_waste_in_correct_cells(self):
        habitat = self.feeding_case()
        habitat.advance()
        state = habitat.snapshot()
        target, consumer = [item["organism"] for item in state["organisms"]]
        self.assertFalse(target["alive"])
        self.assertEqual(target["death_cause"], "consumption")
        self.assertAlmostEqual(consumer["reserves"]["energy"], 11.88)
        cells = state["environment"]["cells"]
        self.assertAlmostEqual(cells[0]["detritus"]["nutrient"], 1)
        self.assertAlmostEqual(cells[0]["detritus"]["energy"], 1.8)
        self.assertEqual(cells[1]["detritus"], {"nutrient": 0.5, "energy": 1})
        self.assertAlmostEqual(habitat.summary()["heat"], 1.32)
        self.assertEqual(habitat.summary()["ecology"]["deaths"], 1)
        self.assertAlmostEqual(habitat.audit()["energy"], 28)

    def test_tissue_loss_overflow_returns_minerals_to_target_cell(self):
        habitat = self.feeding_case(tissue=8, reserve_nutrient=8, reserve_energy=30)
        value = payload(habitat)
        value["rules"]["max_ingestion_nutrient_per_time"] = 1
        habitat = restore(value)
        habitat.advance()
        cells = habitat.snapshot()["environment"]["cells"]
        self.assertAlmostEqual(cells[0]["mineral_nutrient"], 1)
        self.assertAlmostEqual(cells[1]["mineral_nutrient"], 0)
        self.assertAlmostEqual(cells[1]["detritus"]["nutrient"], 0.25)
        habitat.audit()

    def test_consumers_cannot_sense_or_feed_on_remote_producers(self):
        cfg = config(population(2, 1, 2, [(0.2, 0.5)]), population(6, 0, 10, [(3.8, 0.5)]),
                     perception_radius=0.5, consumer_speed=0)
        habitat = Habitat({**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}, cfg)
        habitat.advance()
        self.assertEqual(habitat.summary()["ecology"]["feeding_events"], 0)
        self.assertEqual(habitat.summary()["ecology"]["distance_moved"], 0)
        self.assertEqual(habitat.snapshot()["organisms"][0]["organism"]["tissue"]["nutrient"], 2)

    def test_sensing_and_actual_movement_are_paid_and_work_dissipates(self):
        cfg = config(population(2, 1, 2, [(1.5, 0.5)]), population(6, 0, 10, [(0.2, 0.5)]),
                     consumer_speed=0.1)
        habitat = Habitat({**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}, cfg)
        habitat.advance()
        consumer = habitat.snapshot()["organisms"][1]
        self.assertAlmostEqual(consumer["position"]["x"], 0.3)
        self.assertAlmostEqual(consumer["organism"]["reserves"]["energy"], 9.37)
        self.assertAlmostEqual(habitat.summary()["ecology"]["distance_moved"], 0.1)
        self.assertEqual(habitat.summary()["mechanical_energy"], 0)
        self.assertAlmostEqual(habitat.summary()["heat"], 0.83)

    def test_decomposition_rate_is_shared_across_the_entire_grid(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5), (2.5, 0.5)]),
                     decomposition_fraction_per_time=10, reproduction=False)
        habitat = Habitat({**rules(), "max_decomposition_nutrient_per_time": 1}, cfg)
        habitat.advance(2)
        row = habitat.summary()
        self.assertEqual(row["living_count"], 0)
        self.assertAlmostEqual(row["ecology"]["decomposed_nutrient"], 1)
        self.assertAlmostEqual(row["ecology"]["decomposed_energy"], 2)
        self.assertAlmostEqual(row["detritus_nutrient"], 19)
        self.assertAlmostEqual(row["mineral_nutrient"], 1)
        habitat.audit()

    def test_decomposition_also_respects_the_world_energy_cap(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5), (2.5, 0.5)]),
                     decomposition_fraction_per_time=10, reproduction=False)
        habitat = Habitat({**rules(), "max_decomposition_energy_per_time": 1}, cfg)
        habitat.advance(2)
        self.assertAlmostEqual(habitat.summary()["ecology"]["decomposed_energy"], 1)
        self.assertAlmostEqual(habitat.summary()["ecology"]["decomposed_nutrient"], 0.5)

    def test_depleted_digestion_cannot_consume_food_for_free(self):
        cfg = config(population(2, 1, 2, [(0.99, 0.5)]), population(6, 0, 0.02, [(1.01, 0.5)]),
                     reproduction=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        self.assertEqual(habitat.summary()["ecology"]["feeding_events"], 0)
        self.assertEqual(habitat.snapshot()["organisms"][1]["organism"]["death_cause"], "starvation")

    def test_fast_movement_cannot_tunnel_through_another_consumer(self):
        cfg = config(population(6, 0, 20, [(3.5, 0.5)]),
                     population(6, 0, 20, [(0.5, 0.5), (1.5, 0.5)]),
                     reproduction=False, consumer_speed=10, perception_radius=4)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        blocked = habitat.snapshot()["organisms"][1]
        self.assertEqual(blocked["position"], {"x": 0.5, "y": 0.5})
        self.assertAlmostEqual(blocked["organism"]["reserves"]["energy"], 19.38)
        habitat.audit()

    def test_nearly_equal_clearance_and_feeding_radii_still_allow_contact(self):
        cfg = config(population(6, 0, 20, [(2.5, 0.5)]),
                     population(6, 0, 20, [(0.5, 0.5)]),
                     reproduction=False, consumer_speed=10, perception_radius=4,
                     feeding_radius=0.11)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        self.assertEqual(habitat.summary()["ecology"]["feeding_events"], 1)
        self.assertAlmostEqual(habitat.snapshot()["organisms"][1]["position"]["x"], 2.395)
        habitat.audit()

    def test_dark_extinction_is_not_reseeded_and_recycling_can_be_disabled(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5)]), reproduction=False,
                     decomposition=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance(50)
        row = habitat.summary()
        self.assertEqual(row["living_count"], 0)
        self.assertEqual(row["ecology"]["births"], 0)
        self.assertEqual(row["ecology"]["deaths"], 1)
        self.assertEqual(row["ecology"]["decomposed_nutrient"], 0)
        self.assertEqual(row["mineral_nutrient"], 0)
        self.assertEqual(row["detritus_nutrient"], 10)
        habitat.audit()

    def test_growth_metrics_include_producers_and_preserve_resources(self):
        cfg = config(population(2, 0.5, 5, [(0.5, 0.5)]), reproduction=False,
                     decomposition=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        row = habitat.summary()
        self.assertAlmostEqual(habitat.snapshot()["organisms"][0]["organism"]["tissue"]["nutrient"], 2.5)
        self.assertAlmostEqual(row["ecology"]["tissue_grown"], 0.5)
        self.assertAlmostEqual(row["last_update"]["tissue_grown"], 0.5)
        self.assertAlmostEqual(row["total_nutrient"], 2.5)
        habitat.audit()

    def test_checkpoint_preserves_partial_windows_and_future_trajectories(self):
        world_rules = json.loads((ROOT / "configs/ecosystem_rules.json").read_text())
        cfg = json.loads((ROOT / "configs/ecosystem_habitat.json").read_text())
        uninterrupted = Habitat(world_rules, cfg)
        uninterrupted.advance(137)
        resumed = restore(payload(uninterrupted))
        self.assertEqual(uninterrupted.snapshot(), resumed.snapshot())
        for count in [1, 5, 30, 77, 350]:
            uninterrupted.advance(count)
            resumed.advance(count)
            self.assertEqual(uninterrupted.snapshot(), resumed.snapshot())
            self.assertEqual(uninterrupted.summary(), resumed.summary())
        self.assertGreater(resumed.summary()["ecology"]["births"], 0)
        self.assertGreater(resumed.summary()["ecology"]["feeding_events"], 0)
        resumed.audit()

    def test_atomic_save_overwrites_and_failed_destinations_preserve_previous_file(self):
        habitat = Habitat(rules(), config())
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory) / "saved"
            path = folder / "world.json"
            habitat.save(path)
            habitat.advance()
            habitat.save(path)
            before = path.read_bytes()
            self.assertEqual(Habitat.load(path).snapshot(), habitat.snapshot())
            with self.assertRaises(ValueError):
                habitat.save(folder)
            self.assertEqual(path.read_bytes(), before)
            self.assertEqual(list(Path(directory).glob("*.tmp")), [])

    def test_corrupted_checkpoints_are_rejected(self):
        habitat = self.feeding_case()
        habitat.advance()
        baseline = payload(habitat)
        changes = [
            lambda p: p.update(schema_version=2), lambda p: p.update(kind="inspection"),
            lambda p: p.update(engine_version="wrong"), lambda p: p.update(extra=1),
            lambda p: p["clock"].update(last_ecology_tick=0),
            lambda p: p["positions"].append(deepcopy(p["positions"][0])),
            lambda p: p["positions"][0].update(id=999),
            lambda p: p["positions"][0]["position"].update(x=-1),
            lambda p: p["motion"].append(deepcopy(p["motion"][0])),
            lambda p: p["motion"][0].update(heading=10),
            lambda p: p["ledger"]["pools"].update(heat=1000),
            lambda p: p["ledger"]["organisms"].append(deepcopy(p["ledger"]["organisms"][0])),
            lambda p: p["ledger"]["organisms"][0]["reserves"].update(energy=1),
            lambda p: p["grid"].update(columns=0),
            lambda p: p["grid"]["cells"][0].update(light_energy_per_time=100),
            lambda p: p.update(next_id=1), lambda p: p["totals"].update(deaths=0),
            lambda p: p.update(last_update=None),
            lambda p: p["config"].update(initial_energy=999),
            lambda p: p["ledger"]["organisms"][0].update(age=1e6),
            lambda p: p["ledger"]["organisms"][0].update(next_reproduction_age=1e6),
        ]
        for change in changes:
            value = deepcopy(baseline)
            change(value)
            with self.subTest(change=change), self.assertRaises((ValueError, RuntimeError)):
                NativeHabitat.from_checkpoint_json(json.dumps(value))
        self.assertEqual(payload(habitat), baseline)

    def test_offspring_id_overflow_rolls_back_everything(self):
        value = payload(Habitat(rules(), config()))
        value["next_id"] = 2**64 - 1
        habitat = restore(value)
        before = habitat.snapshot()
        with self.assertRaises(ValueError):
            habitat.advance()
        self.assertEqual(habitat.snapshot(), before)

    def test_inspection_snapshot_is_not_accepted_as_a_checkpoint(self):
        habitat = Habitat(rules(), config())
        with self.assertRaises(ValueError):
            NativeHabitat.from_checkpoint_json(json.dumps(habitat.snapshot()))

    def test_ecology_config_rejects_invalid_costs_radii_and_limits(self):
        for bad in [{"consumer_speed": -1}, {"minimum_spacing": 1}, {"max_population": 0},
                    {"movement_energy_per_distance": 0}, {"perception_radius": 0.1},
                    {"decomposition_fraction_per_time": -1}, {"consumer_speed": True},
                    {"reproduction": 1}, {"unknown": 1}]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                Habitat(rules(), config(**bad))


if __name__ == "__main__":
    unittest.main()
