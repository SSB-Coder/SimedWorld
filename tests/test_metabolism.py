from copy import deepcopy
import json
import unittest

from world._native import Habitat
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.presets import neural_blueprint
from test_ecology_extensions import fixture, advance, state


def metabolic_fixture(kinds=("grazer",), positions=None):
    rules, config = fixture(kinds, positions)
    config["evolution"].update(
        energy_budgeting=True,
        catabolism_fraction_per_time=0.04,
        catabolism_efficiency=0.8,
        foraging_reserve_time=6.0,
        juvenile_reserve_time=24.0,
        energy_history_samples=4,
        energy_history_interval_ticks=6,
    )
    for group in config["populations"]:
        body = group["genome"]["body"]
        body["circle"]["adaptations"]["energy_senses"] = True
        group["genome"]["brain"] = neural_blueprint(body)
    fund(rules, config)
    return rules, config


class MetabolismTests(unittest.TestCase):
    def assert_conserved(self, world):
        audit = json.loads(world.audit_json())
        for key in ["energy_error", "nutrient_error", "light_error"]:
            self.assertAlmostEqual(audit[key], 0.0, places=8)

    def test_tissue_conversion_funds_upkeep_and_conserves_resources(self):
        rules, config = metabolic_fixture()
        config["populations"][0]["reserve_energy"] = 0.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        initial = state(world)["ledger"]["organisms"][0]
        advance(world, {})
        saved = state(world)
        organism = saved["ledger"]["organisms"][0]
        behavior = saved["diagnostics"]["behavior"]["1"]
        burned = initial["tissue"]["nutrient"] - organism["tissue"]["nutrient"]
        self.assertTrue(organism["alive"])
        self.assertGreater(burned, 0)
        self.assertAlmostEqual(behavior["catabolic_energy"], burned * 2.0 * 0.8)
        self.assertLess(behavior["reward"], 0)
        self.assert_conserved(world)

    def test_tissue_conversion_is_limited_and_cannot_create_a_grace_period(self):
        rules, config = metabolic_fixture()
        config["evolution"]["catabolism_fraction_per_time"] = 0.0
        config["populations"][0]["reserve_energy"] = 0.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        saved = state(world)
        self.assertEqual(saved["ledger"]["organisms"][0]["death_cause"], "starvation")
        self.assertEqual(saved["diagnostics"]["catabolic_energy"], 0)
        self.assert_conserved(world)

    def test_zero_reserve_animal_can_fund_contact_digestion_from_tissue(self):
        rules, config = metabolic_fixture(("scavenger",))
        config["populations"][0].update(reserve_energy=0.0, reserve_nutrient=0.0)
        config["cell_overrides"] = [{"column": 4, "row": 8,
            "detritus": {"nutrient": 2.0, "energy": 8.0}}]
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        saved = state(world)
        behavior = saved["diagnostics"]["behavior"]["1"]
        self.assertTrue(saved["ledger"]["organisms"][0]["alive"])
        self.assertGreater(behavior["retained_food_energy"], 0)
        self.assertGreater(behavior["catabolic_energy"], 0)
        self.assert_conserved(world)

    def test_unaffordable_juvenile_runway_blocks_birth_without_rollback(self):
        rules, config = metabolic_fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"].update(juvenile_reserve_time=1000.0, inherit_learning=False)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 103): 1.0})
        saved = state(world)
        self.assertEqual(len(saved["ledger"]["organisms"]), 1)
        self.assertGreater(saved["diagnostics"]["birth_blocks"]["juvenile_capacity"], 0)
        self.assert_conserved(world)

    def test_birth_reserves_cover_measured_child_upkeep(self):
        rules, config = metabolic_fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"]["inherit_learning"] = False
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 103): 1.0})
        children = [o for o in state(world)["ledger"]["organisms"] if o["parent_id"]]
        self.assertEqual(len(children), 1)
        child = children[0]
        advance(world, {})
        costs = state(world)["diagnostics"]["behavior"][str(child["id"])]["costs"]
        interval = rules["physics_dt"] * rules["ecology_interval_ticks"]
        self.assertGreaterEqual(child["reserves"]["energy"],
            sum(costs.values()) / interval * config["evolution"]["juvenile_reserve_time"])
        self.assert_conserved(world)

    def test_retained_intake_excludes_overflow_from_full_reserves(self):
        rules, config = metabolic_fixture(("scavenger",))
        config["cell_overrides"] = [{"column": 4, "row": 8,
            "detritus": {"nutrient": 2.0, "energy": 8.0}}]
        group = config["populations"][0]
        group["reserve_energy"] = group["tissue_nutrient"] * rules["reserve_energy_capacity_per_tissue_nutrient"]
        group["reserve_nutrient"] = 0.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        saved = state(world)
        behavior = saved["diagnostics"]["behavior"]["1"]
        self.assertGreater(behavior["assimilated_energy"], 0)
        self.assertLess(behavior["retained_food_energy"], behavior["assimilated_energy"])
        self.assert_conserved(world)

    def test_optional_actions_cannot_spend_the_upkeep_reserve(self):
        rules, config = metabolic_fixture()
        config["evolution"]["catabolism_fraction_per_time"] = 0.0
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        idle = state(world)["diagnostics"]["behavior"]["1"]["costs"]
        config["populations"][0]["reserve_energy"] = sum(idle.values()) + 0.0001
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 110): 1.0, (1, 111): 1.0, (1, 104): 1.0})
        saved = state(world)
        self.assertTrue(saved["ledger"]["organisms"][0]["alive"])
        self.assertEqual(saved["diagnostics"]["behavior"]["1"]["maintenance_shortfall"], 0)
        self.assert_conserved(world)

    def test_history_is_bounded_and_survives_exact_continuation(self):
        for controller in ["sparse", "ltc"]:
            rules, config = metabolic_fixture()
            config["populations"][0]["reserve_energy"] = 0.0
            fund(rules, config)
            runtime = EcosystemRuntime.create(rules, config, controller=controller)
            runtime.advance(10)
            saved = state(runtime.world)
            self.assertEqual(len(saved["diagnostics"]["energy_history"]["1"]), 4)
            restored = EcosystemRuntime.from_snapshot(runtime.snapshot())
            runtime.advance(3)
            restored.advance(3)
            self.assertEqual(runtime.snapshot(), restored.snapshot())
            self.assert_conserved(runtime.world)

    def test_starvation_history_retains_real_terminal_cause(self):
        rules, config = metabolic_fixture()
        config["populations"][0].update(tissue_nutrient=1.0, reserve_energy=0.0, reserve_nutrient=0.0)
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        saved = state(world)
        death = saved["diagnostics"]["death_history"][0]
        self.assertEqual(death["cause"], "starvation")
        self.assertIn("no_recent_retained_food_energy", death["contributing_signals"])
        self.assertNotIn("1", saved["diagnostics"].get("energy_history", {}))
        self.assertGreater(death["samples"][-1]["behavior"]["maintenance_shortfall"], 0)
        Habitat.from_checkpoint_json(world.checkpoint_json())
        self.assert_conserved(world)

    def test_history_interval_rounds_to_the_next_decision_boundary(self):
        rules, config = metabolic_fixture()
        config["evolution"]["energy_history_interval_ticks"] = 7
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(5):
            advance(world, {})
        history = state(world)["diagnostics"]["energy_history"]["1"]
        self.assertEqual([sample["tick"] for sample in history], [6, 18, 30])

    def test_larger_funded_newborn_can_cover_a_longer_horizon(self):
        rules, config = metabolic_fixture()
        config["ecology"]["reproduction"] = True
        config["evolution"].update(juvenile_reserve_time=95.0, inherit_learning=False)
        config["populations"][0].update(tissue_nutrient=20.0, reserve_nutrient=10.0, reserve_energy=100.0)
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 103): 1.0})
        children = [o for o in state(world)["ledger"]["organisms"] if o["parent_id"]]
        self.assertEqual(len(children), 1)
        child = children[0]
        self.assertGreater(child["tissue"]["nutrient"], rules["offspring_tissue_nutrient"])
        advance(world, {})
        costs = state(world)["diagnostics"]["behavior"][str(child["id"])]["costs"]
        self.assertGreaterEqual(child["reserves"]["energy"], sum(costs.values()) * 950.0)
        self.assert_conserved(world)

    def test_new_energy_inputs_are_present_bounded_and_food_dependent(self):
        rules, config = metabolic_fixture(("scavenger",))
        config["cell_overrides"] = [{"column": 4, "row": 8,
            "detritus": {"nutrient": 2.0, "energy": 8.0}}]
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        agent = json.loads(world.agent_frame_json())["agents"][0]
        inputs = {p["port"] % 256: p["value"] for p in agent["inputs"]}
        self.assertTrue(set(range(52, 56)) <= inputs.keys())
        for port in range(52, 56):
            self.assertLessEqual(abs(inputs[port]), 1)
        self.assertGreater(inputs[53], 0)
        self.assertGreater(inputs[54], 0)
        self.assertEqual(inputs[55], 1)

    def test_faster_motion_preserves_cost_per_distance(self):
        runs = []
        for speed in [1.7, 2.55]:
            rules, config = metabolic_fixture()
            config["ecology"]["consumer_speed"] = speed
            world = Habitat(json.dumps(rules), json.dumps(config))
            advance(world, {(1, 110): 1.0})
            behavior = state(world)["diagnostics"]["behavior"]["1"]
            runs.append(behavior)
            self.assert_conserved(world)
        self.assertAlmostEqual(runs[1]["distance"] / runs[0]["distance"], 1.5)
        for behavior in runs:
            self.assertAlmostEqual(behavior["costs"]["movement"] - 0.0002,
                behavior["distance"] * config["ecology"]["movement_energy_per_distance"])

    def test_high_speed_sweeps_stop_at_a_nearby_body(self):
        rules, config = metabolic_fixture(("grazer", "grazer"),
            [{"x": 6.0, "y": 8.0}, {"x": 8.0, "y": 8.0}])
        config["ecology"]["consumer_speed"] = 100.0
        world = Habitat(json.dumps(rules), json.dumps(config))
        saved = state(world)
        next(m for m in saved["motion"] if m["id"] == 1)["heading"] = 0.0
        world = Habitat.from_checkpoint_json(json.dumps(saved))
        advance(world, {(1, 110): 1.0})
        display = json.loads(world.display_json(None))
        first, second = display["organisms"]
        separation = ((first["position"]["x"] - second["position"]["x"]) ** 2
            + (first["position"]["y"] - second["position"]["y"]) ** 2) ** 0.5
        self.assertGreaterEqual(separation + 1e-9, first["radius"] + second["radius"])
        self.assertLess(first["position"]["x"], second["position"]["x"])
        self.assertTrue(state(world)["diagnostics"]["behavior"]["1"]["blocked"])
        self.assert_conserved(world)

    def test_loss_making_live_food_cannot_consume_mandatory_upkeep(self):
        rules, config = metabolic_fixture(("predator", "grazer"),
            [{"x": 8.0, "y": 8.0}, {"x": 8.85, "y": 8.0}])
        rules["energy_assimilation_fraction"] = 0.01
        config["evolution"]["catabolism_fraction_per_time"] = 0.0
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        idle = state(world)["diagnostics"]["behavior"]["1"]["costs"]
        config["populations"][0]["reserve_energy"] = sum(idle.values()) + 0.001
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        saved = state(world)
        self.assertTrue(saved["ledger"]["organisms"][0]["alive"])
        self.assertEqual(saved["diagnostics"]["behavior"]["1"]["maintenance_shortfall"], 0)
        self.assert_conserved(world)

    def test_death_histories_have_a_global_bound(self):
        rules, config = metabolic_fixture()
        group = config["populations"][0]
        group.update(count=70, tissue_nutrient=1.0, reserve_energy=0.0, reserve_nutrient=0.0,
            positions=[{"x": 2 + i % 10 * 1.3, "y": 2 + i // 10 * 1.3} for i in range(70)])
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        saved = state(world)
        self.assertEqual(len(saved["diagnostics"]["death_history"]), 64)
        self.assertFalse(saved["diagnostics"].get("energy_history"))
        self.assert_conserved(world)

    def test_new_settings_and_corrupt_history_are_rejected(self):
        rules, config = metabolic_fixture()
        for key, value in [("catabolism_fraction_per_time", 1.1),
                           ("catabolism_efficiency", 0),
                           ("energy_history_samples", 65),
                           ("energy_history_interval_ticks", 0),
                           ("turn_rate_per_time", 0)]:
            invalid = deepcopy(config)
            invalid["ecology" if key == "turn_rate_per_time" else "evolution"][key] = value
            with self.assertRaises(ValueError):
                Habitat(json.dumps(rules), json.dumps(invalid))
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        invalid = state(world)
        invalid["diagnostics"]["energy_history"]["1"][0]["behavior"]["catabolic_energy"] = -1
        with self.assertRaises(ValueError):
            Habitat.from_checkpoint_json(json.dumps(invalid))


if __name__ == "__main__":
    unittest.main()
