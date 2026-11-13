import json
import math
from pathlib import Path
import unittest

from world import Habitat
from world._native import Habitat as NativeHabitat


ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text(encoding="utf-8"))
PRESET = json.loads((ROOT / "configs/producer_habitat.json").read_text(encoding="utf-8"))


def small_rules():
    return {**RULES, "world_width": 2.0, "world_height": 1.0,
            "grid_width": 2, "grid_height": 1, "physics_dt": 0.5,
            "ecology_interval_ticks": 2}


def small_config():
    return {"schema_version": 1, "seed": 0, "initial_nutrient": 4.5, "initial_energy": 9.0,
            "mineral_nutrient_per_cell": 1.0, "light_energy_per_cell_per_time": 8.0,
            "light_gradient": 0.0, "producers": {"count": 1, "tissue_nutrient": 2.0,
                "reserve_nutrient": 0.5, "reserve_energy": 5.0,
                "positions": [{"x": 0.25, "y": 0.5}]}}


class HabitatTests(unittest.TestCase):
    def test_preset_is_funded_and_seed_reproduces_initial_positions(self):
        first = Habitat(RULES, PRESET)
        second = Habitat(RULES, PRESET)
        self.assertEqual(first.snapshot(), second.snapshot())
        self.assertEqual(first.summary()["living_count"], 64)
        self.assertEqual(first.summary()["total_nutrient"], 4256)
        self.assertEqual(first.audit()["energy"], 576)
        changed = Habitat(RULES, {**PRESET, "seed": 74})
        self.assertNotEqual(first.snapshot()["organisms"], changed.snapshot()["organisms"])
        for item in first.snapshot()["organisms"]:
            self.assertGreaterEqual(item["position"]["x"], 0)
            self.assertLessEqual(item["position"]["x"], 64)
            self.assertGreaterEqual(item["position"]["y"], 0)
            self.assertLessEqual(item["position"]["y"], 64)

    def test_seed_layout_has_a_stable_versioned_example(self):
        config = small_config()
        config["producers"].pop("positions")
        position = Habitat(small_rules(), config).snapshot()["organisms"][0]["position"]
        self.assertEqual(position, {"x": 1.7666216164272852, "y": 0.43152799704850997})
        config["seed"] = 2**53 + 1
        self.assertEqual(Habitat(small_rules(), config).summary()["seed"], 2**53 + 1)

    def test_ecology_runs_once_per_window_and_ages_once(self):
        habitat = Habitat(RULES, PRESET)
        habitat.advance(5)
        self.assertEqual(habitat.tick, 5)
        self.assertIsNone(habitat.summary()["last_update"])
        self.assertTrue(all(item["organism"]["age"] == 0 for item in habitat.snapshot()["organisms"]))
        habitat.advance()
        self.assertEqual(habitat.summary()["last_ecology_tick"], 6)
        self.assertEqual(habitat.summary()["last_update"]["producers_processed"], 64)
        ages = [item["organism"]["age"] for item in habitat.snapshot()["organisms"]]
        for age in ages:
            self.assertAlmostEqual(age, 0.1)
        habitat.advance()
        self.assertEqual([item["organism"]["age"] for item in habitat.snapshot()["organisms"]], ages)

    def test_one_window_matches_independent_hand_calculation(self):
        habitat = Habitat(small_rules(), small_config())
        habitat.advance(2)
        row = habitat.summary()
        for key, expected in {"total_nutrient": 4.5, "mineral_nutrient": 1,
                              "living_nutrient": 3.5, "living_energy": 12.8,
                              "heat": 2.45, "absorbed_light": 6.25, "incident_light": 16,
                              "escaped_light": 0, "available_light": 9.75,
                              "accounted_energy": 15.25}.items():
            self.assertAlmostEqual(row[key], expected, msg=key)
        organism = habitat.snapshot()["organisms"][0]["organism"]
        self.assertEqual(organism["tissue"], {"nutrient": 3, "energy": 6})
        self.assertAlmostEqual(organism["reserves"]["energy"], 6.8)
        self.assertEqual(organism["age"], 1)

    def test_remote_minerals_cannot_fund_local_growth(self):
        config = small_config()
        config["producers"]["reserve_nutrient"] = 0
        config["cell_overrides"] = [{"column": 0, "row": 0, "mineral_nutrient": 0},
                                     {"column": 1, "row": 0, "mineral_nutrient": 10}]
        config["initial_nutrient"] = 12
        habitat = Habitat(small_rules(), config)
        habitat.advance(8)
        state = habitat.snapshot()
        self.assertEqual(state["organisms"][0]["organism"]["tissue"]["nutrient"], 2)
        self.assertEqual(state["environment"]["cells"][1]["mineral_nutrient"], 10)
        self.assertEqual(habitat.summary()["last_update"]["nutrient_absorbed"], 0)

    def test_dark_cell_cannot_capture_remote_light_and_death_stays_local(self):
        config = small_config()
        config["producers"].update(reserve_nutrient=0, reserve_energy=0)
        config.update(initial_nutrient=3, initial_energy=4,
                      cell_overrides=[{"column": 0, "row": 0,
                                       "mineral_nutrient": 0, "light_energy_per_time": 0}])
        habitat = Habitat(small_rules(), config)
        habitat.advance(2)
        state = habitat.snapshot()
        self.assertEqual(habitat.summary()["absorbed_light"], 0)
        self.assertEqual(habitat.summary()["living_count"], 0)
        self.assertEqual(state["organisms"][0]["organism"]["death_cause"], "starvation")
        self.assertEqual(state["environment"]["cells"][0]["detritus"], {"nutrient": 2, "energy": 4})
        self.assertEqual(state["environment"]["cells"][1]["detritus"], {"nutrient": 0, "energy": 0})
        habitat.advance(6)
        self.assertEqual(habitat.summary()["living_count"], 0)
        self.assertEqual(habitat.summary()["detritus_nutrient"], 2)

    def test_cell_shares_do_not_duplicate_light_or_uptake(self):
        config = small_config()
        config.update(initial_nutrient=6, initial_energy=8, light_energy_per_cell_per_time=2)
        config["producers"].update(count=2, reserve_nutrient=0, reserve_energy=0,
            positions=[{"x": 0.25, "y": 0.5}, {"x": 0.75, "y": 0.5}])
        habitat = Habitat(small_rules(), config)
        habitat.advance(2)
        state = habitat.snapshot()
        first, second = [item["organism"] for item in state["organisms"]]
        self.assertEqual(first["tissue"], second["tissue"])
        self.assertEqual(first["reserves"], second["reserves"])
        self.assertAlmostEqual(first["tissue"]["nutrient"], 2.2)
        self.assertEqual(habitat.summary()["absorbed_light"], 2)
        self.assertEqual(habitat.summary()["last_update"]["nutrient_absorbed"], 1)
        self.assertEqual(state["environment"]["cells"][1]["light_available"], 2)

    def test_unabsorbed_light_expires_and_has_an_explicit_budget(self):
        habitat = Habitat(small_rules(), small_config())
        habitat.advance(4)
        row = habitat.summary()
        self.assertEqual(row["incident_light"], 32)
        self.assertAlmostEqual(row["escaped_light"], 9.75)
        self.assertAlmostEqual(row["incident_light"], row["escaped_light"] + row["available_light"] + row["absorbed_light"])
        self.assertAlmostEqual(row["accounted_energy"], row["initial_energy"] + row["absorbed_light"])

    def test_rate_caps_use_the_actual_window(self):
        rules = {**small_rules(), "physics_dt": 0.05}
        config = small_config()
        config.update(light_energy_per_cell_per_time=1000, mineral_nutrient_per_cell=10,
                      initial_nutrient=22, initial_energy=4)
        config["producers"].update(reserve_energy=0, reserve_nutrient=0)
        habitat = Habitat(rules, config)
        habitat.advance(2)
        report = habitat.summary()["last_update"]
        self.assertAlmostEqual(report["duration"], 0.1)
        self.assertAlmostEqual(report["light_captured"], 1)
        self.assertAlmostEqual(report["nutrient_absorbed"], 0.1)
        self.assertLessEqual(report["tissue_grown"], 0.1)

    def test_chunking_and_repeated_runs_produce_identical_states(self):
        first = Habitat(small_rules(), small_config())
        second = Habitat(small_rules(), small_config())
        first.advance(120)
        for _ in range(120):
            second.advance()
        self.assertEqual(first.snapshot(), second.snapshot())
        self.assertEqual(first.summary(), second.summary())

    def test_empty_world_advances_without_respawning(self):
        config = small_config()
        config.update(initial_nutrient=0, initial_energy=0, mineral_nutrient_per_cell=0,
                      light_energy_per_cell_per_time=2)
        config["producers"].update(count=0, positions=[])
        habitat = Habitat(small_rules(), config)
        habitat.advance(4)
        self.assertEqual(habitat.summary()["living_count"], 0)
        self.assertEqual(habitat.snapshot()["organisms"], [])
        self.assertEqual(habitat.summary()["incident_light"], 8)
        self.assertEqual(habitat.summary()["escaped_light"], 4)
        self.assertEqual(habitat.audit()["energy"], 0)

    def test_position_boundaries_and_invalid_positions(self):
        for position, cell in [({"x": 0, "y": 0}, 0), ({"x": 1, "y": 0}, 1),
                               ({"x": 2, "y": 1}, 1)]:
            config = small_config()
            config["producers"]["positions"] = [position]
            self.assertEqual(Habitat(small_rules(), config).snapshot()["organisms"][0]["cell_index"], cell)
        for position in [{"x": -0.01, "y": 0}, {"x": 2.01, "y": 0},
                         {"x": 1, "y": 1.01}, {"x": math.nan, "y": 0}, {"x": True, "y": 0}]:
            config = small_config()
            config["producers"]["positions"] = [position]
            with self.subTest(position=position), self.assertRaises(ValueError):
                Habitat(small_rules(), config)

    def test_malformed_configs_and_unfunded_founders_are_rejected(self):
        for update in [{"schema_version": 2}, {"seed": True}, {"seed": 2**64},
                       {"light_gradient": 1.1}, {"mineral_nutrient_per_cell": -1},
                       {"light_energy_per_cell_per_time": math.inf}, {"unknown": 1},
                       {"cell_overrides": [{"column": 2, "row": 0, "mineral_nutrient": 0}]},
                       {"cell_overrides": [{"column": 0, "row": 0}]},
                       {"cell_overrides": [{"column": 0, "row": 0, "mineral_nutrient": 1}]*2}]:
            with self.subTest(update=update), self.assertRaises(ValueError):
                Habitat(small_rules(), {**small_config(), **update})
        for update in [{"positions": []}, {"count": True}, {"tissue_nutrient": 0.5},
                       {"reserve_energy": 100}, {"reserve_nutrient": 3}, {"unknown": 1}]:
            config = small_config()
            config["producers"].update(update)
            with self.subTest(update=update), self.assertRaises(ValueError):
                Habitat(small_rules(), config)
        with self.assertRaises(RuntimeError):
            Habitat(small_rules(), {**small_config(), "initial_nutrient": 100})

    def test_invalid_advance_and_snapshot_edits_do_not_change_native_state(self):
        habitat = Habitat(small_rules(), small_config())
        before = habitat.snapshot()
        for count in [-1, True, 1.5, "1", 2**64]:
            with self.subTest(count=count), self.assertRaises(ValueError):
                habitat.advance(count)
            self.assertEqual(habitat.snapshot(), before)
        habitat.advance(0)
        copy = habitat.snapshot()
        copy["environment"]["cells"][0]["mineral_nutrient"] = 999
        copy["organisms"][0]["position"]["x"] = 999
        self.assertEqual(habitat.snapshot(), before)

    def test_light_overflow_rejects_the_whole_tick(self):
        rules = {**small_rules(), "grid_width": 1, "physics_dt": 1, "ecology_interval_ticks": 1}
        config = small_config()
        config.update(initial_nutrient=0, initial_energy=0, mineral_nutrient_per_cell=0,
                      light_energy_per_cell_per_time=1e308)
        config["producers"].update(count=0, positions=[])
        habitat = Habitat(rules, config)
        habitat.advance()
        before = habitat.snapshot()
        with self.assertRaises(ValueError):
            habitat.advance()
        self.assertEqual(habitat.tick, 1)
        self.assertEqual(habitat.snapshot(), before)

    def test_failed_processing_rolls_back_cells_age_and_clock(self):
        rules = {**small_rules(), "physics_dt": 1e308, "ecology_interval_ticks": 1,
                 "maintenance_per_tissue_nutrient_per_time": 1e-308,
                 "max_growth_nutrient_per_time": 1e308}
        config = {**small_config(), "light_energy_per_cell_per_time": 0}
        habitat = Habitat(rules, config)
        before = habitat.snapshot()
        with self.assertRaises(ValueError):
            habitat.advance()
        self.assertEqual(habitat.tick, 0)
        self.assertEqual(habitat.snapshot(), before)

    def test_raw_native_boundary_rejects_nonfinite_and_unknown_json(self):
        for text in [json.dumps({**small_config(), "unknown": 1}),
                     json.dumps({**small_config(), "seed": 1.5}),
                     json.dumps({**small_config(), "initial_nutrient": math.nan})]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                NativeHabitat(json.dumps(small_rules()), text)

    def test_default_run_keeps_global_and_cell_inventories_balanced(self):
        habitat = Habitat(RULES, PRESET)
        habitat.advance(600)
        state = habitat.snapshot()
        cells = state["environment"]["cells"]
        organisms = [item["organism"] for item in state["organisms"]]
        nutrient = math.fsum(c["mineral_nutrient"] + c["detritus"]["nutrient"] for c in cells)
        nutrient += math.fsum(o["tissue"]["nutrient"] + o["reserves"]["nutrient"] for o in organisms)
        self.assertAlmostEqual(nutrient, 4256)
        self.assertAlmostEqual(math.fsum(c["mineral_nutrient"] for c in cells), state["pools"]["mineral_nutrient"])
        for value in habitat.audit().values():
            self.assertTrue(math.isfinite(value))
        self.assertAlmostEqual(habitat.audit()["nutrient_error"], 0, places=7)
        self.assertAlmostEqual(habitat.audit()["energy_error"], 0, places=7)
        self.assertAlmostEqual(habitat.audit()["light_error"], 0, places=7)


if __name__ == "__main__":
    unittest.main()
