from copy import deepcopy
import json
import unittest

from world._native import Habitat
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.experiments.ecosystem import ecosystem_preset
from world.presets import neural_blueprint
from test_ecology_extensions import RULES, advance, fixture, state
from test_metabolism import metabolic_fixture


def planted_fixture(energy=0.0, plant_tissue=7.0):
    rules, config = metabolic_fixture()
    config["evolution"].update(catabolism_fraction_per_time=0.0, embodied_ecology=True)
    config["producers"].update(count=1, tissue_nutrient=plant_tissue,
        reserve_energy=0.0, reserve_nutrient=0.0, positions=[{"x": 4.64, "y": 8.0}])
    config["populations"][0].update(tissue_nutrient=1.0, reserve_energy=energy,
        reserve_nutrient=0.0)
    fund(rules, config)
    return rules, config


def inputs(world, identity):
    actor = next(a for a in json.loads(world.agent_frame_json())["agents"] if a["id"] == identity)
    return {p["port"] % 256: p["value"] for p in actor["inputs"]}


class PrioritySurvivalTests(unittest.TestCase):
    def assert_conserved(self, world):
        for key in ["energy_error", "nutrient_error", "light_error"]:
            self.assertAlmostEqual(json.loads(world.audit_json())[key], 0.0, places=8)

    def test_starving_contact_meal_funds_digestion_and_controller_once(self):
        rules, config = planted_fixture()
        world = Habitat(json.dumps(rules), json.dumps(config))
        actor = json.loads(world.agent_frame_json())["agents"][0]
        self.assertTrue(actor["authorized"])
        advance(world, {(2, 120): 1.0})
        saved = state(world)
        grazer = saved["ledger"]["organisms"][1]
        behavior = saved["diagnostics"]["behavior"]["2"]
        self.assertTrue(grazer["alive"])
        self.assertGreater(grazer["reserves"]["energy"], 0.0)
        self.assertGreater(behavior["retained_food_energy"], 0.0)
        self.assertEqual(behavior["catabolic_energy"], 0.0)
        self.assertLessEqual(behavior["ingested"], rules["max_ingestion_nutrient_per_time"]
            * rules["physics_dt"] * rules["ecology_interval_ticks"] + 1e-12)
        self.assert_conserved(world)

    def test_contact_credit_does_not_force_feeding_or_grant_energy(self):
        rules, config = planted_fixture()
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {})
        saved = state(world)
        self.assertEqual(saved["ledger"]["organisms"][1]["death_cause"], "starvation")
        behavior = saved["diagnostics"]["death_history"][0]["samples"][-1]["behavior"]
        self.assertEqual(behavior["ingested"], 0.0)
        self.assertIn("controller_unfunded", behavior["failures"])
        self.assertIn("120", behavior["requested"])
        self.assert_conserved(world)

    def test_small_real_bite_is_both_sensed_and_eaten(self):
        rules, config = planted_fixture(energy=3.0, plant_tissue=1.0)
        config["ecology"]["grazing_residual_fraction"] = 0.9
        group = config["populations"][0]
        group["tissue_nutrient"] = 6.5
        body = group["genome"]["body"]
        body["circle"]["adaptations"]["social_senses"] = True
        group["genome"]["brain"] = neural_blueprint(body)
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        sensory = inputs(world, 2)
        self.assertEqual(sensory[40], 1.0)
        self.assertGreater(sensory[12], 0.0)
        advance(world, {(2, 120): 1.0})
        self.assertGreater(state(world)["diagnostics"]["behavior"]["2"]["ingested"], 0.0)
        self.assert_conserved(world)

    def test_competing_contact_meals_cannot_spend_the_same_food_twice(self):
        rules, config = planted_fixture(plant_tissue=1.0)
        config["ecology"]["grazing_residual_fraction"] = 0.9
        config["populations"][0]["positions"] = [{"x": 4.26, "y": 8.0}]
        other = deepcopy(config["populations"][0])
        other["positions"] = [{"x": 5.02, "y": 8.0}]
        config["populations"].append(other)
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(2, 120): 1.0, (3, 120): 1.0})
        saved = state(world)
        self.assertEqual(sum(o["alive"] for o in saved["ledger"]["organisms"]), 1)
        self.assertAlmostEqual(saved["last_update"]["ecology"]["ingested_nutrient"], 0.1)
        self.assert_conserved(world)

    def test_contact_recovery_from_detritus_preserves_one_window_intake_limit(self):
        rules, config = metabolic_fixture(("scavenger",))
        config["evolution"]["catabolism_fraction_per_time"] = 0.0
        config["populations"][0].update(reserve_energy=0.0, reserve_nutrient=0.0)
        config["cell_overrides"] = [{"column": 4, "row": 8,
            "detritus": {"nutrient": 2.0, "energy": 8.0}}]
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        behavior = state(world)["diagnostics"]["behavior"]["1"]
        self.assertTrue(state(world)["ledger"]["organisms"][0]["alive"])
        self.assertGreater(behavior["retained_food_energy"], 0.0)
        self.assertLessEqual(behavior["ingested"], rules["max_ingestion_nutrient_per_time"]
            * rules["physics_dt"] * rules["ecology_interval_ticks"] + 1e-12)
        self.assert_conserved(world)

    def test_executed_intake_includes_both_feeding_phases(self):
        rules, config = metabolic_fixture(("scavenger",))
        config["evolution"]["catabolism_fraction_per_time"] = 0.0
        config["populations"][0].update(reserve_energy=0.0, reserve_nutrient=0.0)
        config["cell_overrides"] = [
            {"column": 3, "row": 8, "detritus": {"nutrient": 0.06, "energy": 0.24}},
            {"column": 4, "row": 8, "detritus": {"nutrient": 0.5, "energy": 2.0}},
        ]
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        advance(world, {(1, 120): 1.0})
        behavior = state(world)["diagnostics"]["behavior"]["1"]
        self.assertGreater(behavior["ingested"], 0.06)
        self.assertAlmostEqual(behavior["executed"]["120"], behavior["ingested"])
        self.assert_conserved(world)

    def test_only_actual_danger_populates_threat_channels(self):
        for neighbor, expected in [("grazer", False), ("predator", True)]:
            rules, config = fixture(("grazer", neighbor),
                [{"x": 8.0, "y": 8.0}, {"x": 9.2, "y": 8.0}])
            for group in config["populations"]:
                body = group["genome"]["body"]
                body["circle"]["adaptations"].update(social_senses=True, sociality=1.0, camouflage=0.0)
                group["genome"]["brain"] = neural_blueprint(body)
            world = Habitat(json.dumps(rules), json.dumps(config))
            sensory = inputs(world, 1)
            self.assertEqual(sensory[16] > 0.0, expected)
            self.assertEqual(sensory[38] > 0.0, expected)
            self.assertGreater(sensory[32], 0.0)

    def test_seeded_neural_controllers_can_flee_a_nearby_predator(self):
        for controller in ["sparse", "ltc"]:
            with self.subTest(controller=controller):
                rules, config = ecosystem_preset(RULES, terrain=False)
                rules.update(world_width=32.0, world_height=24.0, grid_width=32, grid_height=24)
                config["producers"]["count"] = 0
                groups = {g["genome"]["body"]["diet"]: g for g in config["populations"]}
                config["populations"] = [deepcopy(groups[d]) for d in ["grazer", "predator"]]
                config["cell_overrides"] = []
                config["ecology"].update(reproduction=False, decomposition=False)
                for i, group in enumerate(config["populations"]):
                    group.update(count=1, positions=[{"x": 16.0 + i * 3, "y": 12.0}],
                        reserve_energy=32.0)
                    group["genome"]["body"]["circle"]["adaptations"]["camouflage"] = 0.0
                predator = config["populations"][1]
                predator["genome"]["body"]["circle"]["adaptations"]["anchored"] = True
                for node in predator["genome"]["brain"]["nodes"]:
                    node["bias"] = 0.0
                for edge in predator["genome"]["brain"]["connections"]:
                    edge["weight"] = 0.0
                fund(rules, config)
                runtime = EcosystemRuntime.create(rules, config, controller=controller)
                runtime.advance(100)
                organisms = {o["id"]: o for o in runtime.display(1)["organisms"]}
                dx = organisms[1]["position"]["x"] - organisms[2]["position"]["x"]
                dy = organisms[1]["position"]["y"] - organisms[2]["position"]["y"]
                self.assertTrue(organisms[1]["alive"])
                self.assertGreater(dx * dx + dy * dy, 3.0 ** 2)
                self.assert_conserved(runtime.world)

    def test_predator_senses_every_animal_guild_as_food(self):
        for diet in ["grazer", "predator", "scavenger", "recycler"]:
            with self.subTest(diet=diet):
                rules, config = fixture(("predator", diet),
                    [{"x": 8.0, "y": 8.0}, {"x": 9.2, "y": 8.0}])
                for group in config["populations"]:
                    group["genome"]["body"]["circle"]["adaptations"]["camouflage"] = 0.0
                world = Habitat(json.dumps(rules), json.dumps(config))
                self.assertGreater(inputs(world, 1)[12], 0.0)

    def test_late_attack_memory_uses_actual_hit_tick(self):
        rules, config = fixture(("predator", "grazer"),
            [{"x": 8.0, "y": 8.0}, {"x": 8.9, "y": 8.0}])
        config["evolution"]["embodied_ecology"] = True
        world = Habitat(json.dumps(rules), json.dumps(config))
        for _ in range(22):
            advance(world, {})
        for _ in range(40):
            advance(world, {(1, 121): 1.0, (1, 111): 1.0})
            behavior = state(world)["diagnostics"]["behavior"]["2"]
            if behavior["damage_received"] > 0:
                break
        else:
            self.fail("No physical hit in a full turn")
        hit_tick = json.loads(world.agent_frame_json())["tick"]
        self.assertEqual(behavior["attacker_tick"], hit_tick)
        advance(world, {})
        remembered = state(world)["diagnostics"]["behavior"]["2"]
        self.assertEqual(remembered["attacker"], 1)
        self.assertEqual(remembered["attacker_tick"], hit_tick)
        self.assert_conserved(world)


if __name__ == "__main__":
    unittest.main()
