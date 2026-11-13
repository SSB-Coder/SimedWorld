from copy import deepcopy
import hashlib
import json

from ..ecosystem import EcosystemRuntime
from ..presets import evolution_preset, founder_genome


def compact_preset(rules, seed=73):
    rules, config = evolution_preset(rules, seed)
    rules.update(world_width=20.0, world_height=20.0, grid_width=20, grid_height=20)
    config["producers"]["count"] = 12
    config["populations"] = config["populations"][:2]
    config["populations"][0]["count"] = 4
    config["populations"][1]["count"] = 2
    config["ecology"]["max_population"] = 64
    fund(rules, config)
    return rules, config


def fund(rules, config):
    groups = [config["producers"]] + config.get("populations", [])
    config["initial_nutrient"] = config["mineral_nutrient_per_cell"] * rules[
        "grid_width"
    ] * rules["grid_height"] + sum(
        g["count"] * (g["tissue_nutrient"] + g["reserve_nutrient"]) for g in groups
    )
    config["initial_energy"] = sum(
        g["count"]
        * (
            g["tissue_nutrient"] * rules["tissue_energy_per_nutrient"]
            + g["reserve_energy"]
        )
        for g in groups
    )
    for cell in config.get("cell_overrides", []):
        if cell.get("mineral_nutrient") is not None:
            config["initial_nutrient"] += (
                cell["mineral_nutrient"] - config["mineral_nutrient_per_cell"]
            )
        organic = cell.get("detritus") or {}
        config["initial_nutrient"] += organic.get("nutrient", 0.0)
        config["initial_energy"] += organic.get("energy", 0.0)


def metrics(runtime):
    state = json.loads(runtime.world.snapshot_json())
    living = [r["organism"] for r in state["organisms"] if r["organism"]["alive"]]
    active = [o for o in living if o["genome"].get("brain")]
    summary = runtime.summary()
    return {
        "tick": runtime.tick,
        "living": len(living),
        "active_living": len(active),
        "births": summary["ecology"]["births"],
        "deaths": summary["ecology"]["deaths"],
        "feeding_events": summary["ecology"]["feeding_events"],
        "accepted_mutations": summary["diagnostics"]["accepted_mutations"],
        "rejected_mutations": summary["diagnostics"]["rejected_mutations"],
        "birth_blocks": summary["diagnostics"].get("birth_blocks", {}),
        "population_limit": state["config"]["ecology"]["max_population"],
        "nodes": sum(len(o["genome"]["brain"]["nodes"]) for o in active),
        "connections": sum(len(o["genome"]["brain"]["connections"]) for o in active),
        "module_counts": [len(o["genome"]["body"]["modules"]) for o in living],
        "audit": runtime.audit(),
    }


def archive_pair(runtime):
    organisms = {o["id"]: o for o in runtime.snapshot()["world"]["ledger"]["organisms"]}
    descendants = {identity: 0 for identity in organisms}
    for organism in organisms.values():
        parent = organism["parent_id"]
        while parent is not None:
            descendants[parent] += 1
            parent = organisms[parent]["parent_id"]
    eligible = [
        o
        for o in organisms.values()
        if o["genome"].get("brain") and o["parent_id"] is not None
    ]
    if not eligible:
        return None
    selected = min(eligible, key=lambda o: (-descendants[o["id"]], o["id"]))
    ancestor = selected
    while ancestor["parent_id"] is not None:
        ancestor = organisms[ancestor["parent_id"]]

    def item(organism):
        encoded = json.dumps(organism["genome"], sort_keys=True, separators=(",", ":"))
        return {
            "organism_id": organism["id"],
            "genome": deepcopy(organism["genome"]),
            "sha256": hashlib.sha256(encoded.encode()).hexdigest(),
        }

    return {
        "criterion": "born neural organism with most recorded descendants; lowest ID breaks ties",
        "ancestor": item(ancestor),
        "descendant": item(selected),
    }


def common_environment(rules, genome, seed, decisions=100):
    rules, config = compact_preset(rules, seed)
    config["ecology"]["reproduction"] = False
    config["evolution"]["mutation"].update(
        parameter_edits=0.0,
        body_edits=0.0,
        body_structure_rate=0.0,
        neural_structure_rate=0.0,
    )
    food = deepcopy(config["producers"])
    food["count"] = 8
    food["positions"] = [
        {"x": x, "y": y}
        for x, y in [
            (3, 3),
            (10, 3),
            (17, 3),
            (3, 10),
            (17, 10),
            (3, 17),
            (10, 17),
            (17, 17),
        ]
    ]
    subject = {
        "count": 1,
        "tissue_nutrient": 7.0,
        "reserve_nutrient": 3.0,
        "reserve_energy": 30.0,
        "positions": [{"x": 10.0, "y": 10.0}],
        "genome": deepcopy(genome),
    }
    config["producers"] = subject
    config["populations"] = [food]
    if genome["body"]["diet"] == "predator":
        config["populations"].append(
            {
                "count": 6,
                "tissue_nutrient": 6.0,
                "reserve_nutrient": 1.0,
                "reserve_energy": 25.0,
                "genome": founder_genome("grazer", neural=False),
                "positions": [
                    {"x": x, "y": y}
                    for x, y in [(6, 6), (14, 6), (6, 14), (14, 14), (6, 10), (14, 10)]
                ],
            }
        )
    fund(rules, config)
    runtime = EcosystemRuntime.create(rules, config)
    runtime.advance(decisions)
    subject_id = 1
    organism = next(
        o
        for o in runtime.snapshot()["world"]["ledger"]["organisms"]
        if o["id"] == subject_id
    )
    return {
        "seed": seed,
        "alive": organism["alive"],
        "age": organism["age"],
        "tissue": organism["tissue"],
        "reserves": organism["reserves"],
        "audit": runtime.audit(),
    }


def run_controls(rules, seeds=(73, 89, 107), decisions=120, evaluation_decisions=100):
    rows = []
    for seed in seeds:
        for condition in ["clone", "brain_only", "body_only", "combined"]:
            world_rules, config = compact_preset(rules, seed)
            config["evolution"]["mutation"].update(
                parameter_edits=0.0 if condition in {"clone", "body_only"} else 1.0,
                body_edits=0.0 if condition in {"clone", "brain_only"} else 1.0,
                body_structure_rate=0.0,
                neural_structure_rate=0.0,
            )
            runtime = EcosystemRuntime.create(world_rules, config)
            runtime.advance(decisions)
            pair = archive_pair(runtime)
            row = {
                "seed": seed,
                "condition": condition,
                "rules": world_rules,
                "configuration": runtime.snapshot()["world"]["config"],
                "engine_version": runtime.snapshot()["engine_version"],
                "metrics": metrics(runtime),
                "candidate_pair": pair,
            }
            if pair and condition == "combined":
                evaluation_seeds = [seed + 100003, seed + 200003]
                row["common_environment"] = {
                    kind: [
                        common_environment(
                            rules, pair[kind]["genome"], s, evaluation_decisions
                        )
                        for s in evaluation_seeds
                    ]
                    for kind in ["ancestor", "descendant"]
                }
            rows.append(row)
    return {
        "schema_version": 1,
        "seeds": list(seeds),
        "decisions": decisions,
        "evaluation_decisions": evaluation_decisions,
        "lifetime_plasticity": False,
        "structural_mutation": False,
        "interpretation": "Pilot controls and held-out common-environment comparisons. All seeds, extinctions, and absent gains are retained; this does not establish increasing intelligence or population-scale adaptation.",
        "runs": rows,
    }
