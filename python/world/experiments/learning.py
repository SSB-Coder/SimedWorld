from copy import deepcopy

from ..ecosystem import EcosystemRuntime
from ..presets import circle_genome, circle_preset
from .adaptation import fund


def feeding_task(rules, seed=73, learning_rate=0.15):
    rules, config = circle_preset(rules, seed, consumers=1, producers=1)
    rules.update(
        world_width=16.0,
        world_height=16.0,
        grid_width=16,
        grid_height=16,
        max_ingestion_nutrient_per_time=1.0,
        max_growth_nutrient_per_time=2.0,
    )
    config["producers"].update(positions=[{"x": 8.0, "y": 8.0}], reserve_nutrient=7.0)
    config["populations"][0].update(
        positions=[{"x": 7.13, "y": 8.0}],
        genome=circle_genome(hidden=0, seed=seed, learning_rate=learning_rate),
    )
    for node in config["populations"][0]["genome"]["brain"]["nodes"]:
        if node["kind"] == "output":
            node["bias"] = -8.0 if node["port"] % 256 == 110 else 0.0
    config["evolution"].update(repair_nutrient_per_time=2.0)
    config["ecology"]["reproduction"] = False
    fund(rules, config)
    return rules, config


def compare_learning(rules, seeds=(73, 89, 107), decisions=500):
    rows = []
    for seed in seeds:
        for condition, rate in [("plastic", 0.15), ("frozen", 0.0)]:
            task_rules, config = feeding_task(deepcopy(rules), seed, rate)
            runtime = EcosystemRuntime.create(task_rules, config)
            initial = runtime.brains.snapshot()
            runtime.advance(decisions)
            state = runtime.snapshot()
            organism = next(
                o for o in state["world"]["ledger"]["organisms"] if o["id"] == 2
            )
            rows.append(
                {
                    "seed": seed,
                    "condition": condition,
                    "decisions": decisions,
                    "alive": organism["alive"],
                    "food_ingested": state["world"]["totals"]["ingested_nutrient"],
                    "energy": organism["reserves"]["energy"],
                    "parameter_change": runtime.brains.changes.get(2),
                    "initial_parameters": initial,
                    "configuration": config,
                    "rules": task_rules,
                    "final_state": state,
                    "audit": runtime.audit(),
                }
            )
    return {
        "schema_version": 1,
        "scope": "Controlled lifetime feeding-skill task; identical exploration seeds, no births or population reseeding. Not evidence of general intelligence.",
        "runs": rows,
    }
