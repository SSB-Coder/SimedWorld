"""Declared ecological scenarios; founder instincts remain visible in the genes."""

from copy import deepcopy
import math

from ..presets import circle_genome, circle_preset, neural_blueprint
from .adaptation import fund


def ecosystem_preset(rules, seed=73, scale=1, terrain=True, revised=True):
    if type(scale) is not int or not 1 <= scale <= 16:
        raise ValueError("scale must be an integer within [1,16]")
    rules, config = circle_preset(rules, seed=seed)
    rules["model_id"] = "ecosystem-circle-v1"
    rules.update(
        world_width=48.0 * scale**0.5,
        world_height=27.0 * scale**0.5,
        grid_width=48 * scale,
        grid_height=27,
    )
    definitions = [
        ("producer", 96, 0.0, 0.0),
        ("grazer", 24, 0.05, 0.0),
        ("pursuit", 5, 0.1, 0.0),
        ("ambush", 4, 0.8, 0.0),
        ("thread", 4, 0.55, 0.8),
        ("scavenger", 5, 0.1, 0.0),
        ("recycler", 8, 0.0, 0.0),
    ]
    groups = []
    for ordinal, (kind, count, camo, filament) in enumerate(definitions):
        producer = kind == "producer"
        genome = circle_genome(producer, seed=seed + ordinal)
        body = genome["body"]
        body["diet"] = (
            "predator"
            if kind in {"pursuit", "ambush", "thread"}
            else kind
            if kind in {"scavenger", "recycler"}
            else "grazer"
        )
        body["circle"]["adaptations"] = {
            "agency": not producer,
            "anchored": producer,
            "camouflage": camo,
            "pigment": 0.5,
            "armor": 0.1,
            "capture": 0.8 if producer else 0.0,
            "digestion": 0.75 if not producer else 0.0,
            "filament": filament,
            "communication": 0.35 if not producer else 0.0,
            "burst": 0.6 if kind == "pursuit" else 0.15,
            "neural_growth": 0.3,
            "neural_pruning": 0.2,
            "inherit_structure": False,
        }
        body["circle"]["speed"] = (
            1.3 if kind == "pursuit" else 0.7 if kind in {"ambush", "thread"} else 1.0
        )
        if not producer:
            genome["brain"] = neural_blueprint(
                body, hidden=16, seed=seed + ordinal, kind=kind
            )
            for node in genome["brain"]["nodes"]:
                if node["port"] == 256 + 100:
                    node["bias"] = 1.5
                if node["port"] == 256 + 121:
                    node["bias"] = 1.0 if body["diet"] == "predator" else -2.0
            lookup = {
                n["port"]: n["id"]
                for n in genome["brain"]["nodes"]
                if n["port"] is not None
            }
            for source, target, weight in [
                (3, 101, 3.0),
                (23, 122, 3.0),
                (12, 109, 1.2),
                (24, 111, 0.4),
            ]:
                genome["brain"]["connections"].append(
                    {
                        "id": genome["brain"]["next_connection"],
                        "source": lookup[256 + source],
                        "target": lookup[256 + target],
                        "weight": weight,
                    }
                )
                genome["brain"]["next_connection"] += 1
        groups.append(
            {
                "count": count * scale,
                "tissue_nutrient": 7.0 if producer else 6.5,
                "reserve_nutrient": 4.0,
                "reserve_energy": 28.0,
                "genome": genome,
            }
        )
    config["producers"], config["populations"] = groups[0], groups[1:]
    config["ecology"].update(
        max_population=1024 * scale, decomposition_fraction_per_time=0.015
    )
    config["evolution"]["mutation"].update(
        body_structure_rate=0.015, brain_creation_rate=0.0
    )
    config["evolution"]["development_interval_ticks"] = 600
    config["evolution"]["terrain"] = []
    if terrain:
        size = scale**0.5
        for kind, x, y, radius, density in [
            ("rock", 14, 12, 1.8, 1.0),
            ("rock", 33, 18, 1.4, 1.0),
            ("vegetation", 11, 7, 5, 0.8),
            ("vegetation", 35, 9, 5, 0.7),
            ("water", 25, 20, 3, 0.7),
            ("shade", 37, 20, 4, 0.6),
        ]:
            config["evolution"]["terrain"].append(
                {
                    "kind": kind,
                    "center": {"x": x * size, "y": y * size},
                    "radius": radius * size,
                    "density": density,
                }
            )
    if revised:
        revise_ecology(rules, config, seed)
    fund(rules, config)
    return rules, config


def ablation(config, kind):
    result = deepcopy(config)
    if kind not in {
        "full",
        "fixed_weights",
        "fixed_topology",
        "no_signals",
        "ineffective_signals",
        "no_camouflage",
        "no_traps",
        "no_social",
        "genetic_only",
    }:
        raise ValueError("unknown control")
    for group in [result["producers"], *result.get("populations", [])]:
        circle = group["genome"]["body"]["circle"]
        a = circle.get("adaptations", {})
        if kind == "fixed_weights":
            circle["learning_rate"] = 0.0
        if kind == "fixed_topology":
            a.update(neural_growth=0.0, neural_pruning=0.0)
        if kind == "no_signals":
            a["communication"] = 0.0
        if kind == "no_camouflage":
            a["camouflage"] = 0.0
        if kind == "no_traps":
            a["filament"] = 0.0
    if kind == "ineffective_signals":
        result["evolution"]["receive_signals"] = False
    if kind == "fixed_weights":
        result["evolution"]["learned_weight_updates"] = False
    if kind == "fixed_topology":
        result["evolution"]["neural_development"] = False
        result["evolution"]["mutation"].update(
            neural_structure_rate=0.0, brain_creation_rate=0.0
        )
    if kind == "no_social":
        result["evolution"]["social_observations"] = False
    if kind == "genetic_only":
        result["evolution"]["inherit_learning"] = False
    for control, flag in [
        ("no_signals", "allow_communication"),
        ("no_traps", "allow_filaments"),
        ("no_camouflage", "allow_camouflage"),
    ]:
        if kind == control:
            result["evolution"][flag] = False
    return result


def hunting_scenario(rules, seed=73, hunters=4, signals=True):
    if type(hunters) is not int or not 1 <= hunters <= 16:
        raise ValueError("invalid hunter count")
    rules, config = ecosystem_preset(rules, seed, terrain=False)
    rules.update(world_width=16.0, world_height=16.0, grid_width=16, grid_height=16)
    config["cell_overrides"] = []
    config["evolution"]["founder_age_spread"] = 0.0
    config["producers"]["count"] = 0
    prey = deepcopy(config["populations"][0])
    prey.update(
        count=1,
        tissue_nutrient=24.2,
        reserve_energy=60.0,
        positions=[{"x": 8.0, "y": 8.0}],
    )
    prey["genome"]["body"]["modules"][0]["radius"] = 0.8
    hunt = deepcopy(config["populations"][1])
    hunt.update(
        count=hunters, tissue_nutrient=2.5, reserve_energy=12.0, reserve_nutrient=1.0
    )
    hunt["genome"]["body"]["modules"][0]["radius"] = 0.25
    hunt["genome"]["body"]["circle"]["adaptations"]["communication"] = (
        0.6 if signals else 0.0
    )
    hunt["positions"] = [
        {
            "x": 8 + 2.0 * math.cos(i * math.tau / hunters),
            "y": 8 + 2.0 * math.sin(i * math.tau / hunters),
        }
        for i in range(hunters)
    ]
    config["populations"] = [prey, hunt]
    config["ecology"].update(reproduction=False, max_population=64)
    fund(rules, config)
    return rules, config


def revise_ecology(rules, config, seed):
    rules.update(
        maturity_tissue_nutrient=2.0,
        minimum_reproduction_age=12.0,
        reproduction_cooldown=18.0,
        max_ingestion_nutrient_per_time=1.5,
    )
    config["light_energy_per_cell_per_time"] = 3.0
    config["evolution"].update(
        embodied_ecology=True,
        energy_budgeting=True,
        catabolism_fraction_per_time=0.04,
        catabolism_efficiency=0.8,
        foraging_reserve_time=6.0,
        juvenile_reserve_time=24.0,
        energy_history_samples=24,
        energy_history_interval_ticks=60,
        founder_age_spread=6.0,
        trace_time=6.0,
        exploration=0.10,
        injury_feedback=1.0,
        temporal_feedback=True,
        plasticity_decay_per_time=1.0 / 30.0,
        reproduction_feedback=0.25,
    )
    config["ecology"]["decomposition_fraction_per_time"] = 0.005
    config["ecology"].update(
        producer_minimum_reproduction_age=60.0,
        producer_reproduction_cooldown=120.0,
        producer_litter_fraction_per_time=0.008,
        producer_reproduction_reserve_fraction=0.3,
        grazing_residual_fraction=0.5,
        consumer_speed=2.55,
        turn_rate_per_time=4.5,
    )
    config["evolution"]["mutation"].update(sigma=0.1, body_sigma=0.12)
    config["cell_overrides"] = [
        {
            "column": x,
            "row": y,
            "mineral_nutrient": 0.4,
            "detritus": {"nutrient": 0.6, "energy": 1.2},
        }
        for y in range(rules["grid_height"])
        for x in range(rules["grid_width"])
        if (x + y) % 3 == 0
    ]
    for ordinal, group in enumerate([config["producers"], *config["populations"]]):
        scale = config["producers"]["count"] // 96
        if ordinal in {1, 2, 3, 4}:
            group["count"] = {1: 36, 2: 2, 3: 1, 4: 1}[ordinal] * scale
        genome = group["genome"]
        body = genome["body"]
        circle = body["circle"]
        a = circle["adaptations"]
        diet = body["diet"]
        group["reserve_energy"] = 16.0
        if ordinal == 0:
            continue
        a.update(
            social_senses=True,
            foraging_choices=True,
            energy_senses=True,
            offense=0.4 if diet == "grazer" else 0.7,
            sociality=0.4,
            pigment=0.25 if diet == "grazer" else 0.85 if diet == "predator" else 0.55,
        )
        if diet in {"scavenger", "recycler"}:
            tissue = 3.2 if diet == "scavenger" else 2.4
            group.update(
                tissue_nutrient=tissue,
                reserve_nutrient=1.0,
                reserve_energy=tissue * 4.0,
            )
            body["modules"][0]["radius"] = math.sqrt(
                tissue / (12.0 * math.pi * body["density"])
            )
            a.update(offense=0.15, communication=0.1, burst=0.05, digestion=1.0)
        circle["learning_rate"] = 0.002
        kind = [
            "producer",
            "grazer",
            "pursuit",
            "ambush",
            "thread",
            "scavenger",
            "recycler",
        ][ordinal]
        genome["brain"] = neural_blueprint(
            body, hidden=16, seed=seed + ordinal, kind=kind
        )
        brain = genome["brain"]
        lookup = {n["port"] % 256: n for n in brain["nodes"] if n["port"] is not None}
        turn = lookup[111]["id"]
        for e in brain["connections"]:
            if e["target"] == turn and e["source"] in {
                lookup[19]["id"],
                lookup[20]["id"],
            }:
                e["weight"] = 0.0
            elif (
                e["target"] == turn
                and next(n for n in brain["nodes"] if n["id"] == e["source"])["kind"]
                == "hidden"
            ):
                e["weight"] *= 0.1
        lookup[110]["bias"] = 0.25 if kind != "ambush" else -0.1
        lookup[108]["bias"] = -1.0
        lookup[121]["bias"] = 0.5 if diet == "predator" else -2.0
        for source, target, w in [
            (10, 111, 1.4),
            (19, 111, 1.0),
            (11, 110, 0.8),
            (40, 120, 2.0),
            (40, 110, -0.45),
            (12, 120, 0.6),
            (43, 111, 0.8),
            (44, 111, -0.8),
            (30, 111, 0.3),
            (33, 111, 0.2),
            (31, 110, 0.25),
            (35, 110, -0.7),
            (36, 111, -3.0),
            (37, 111, 0.8),
            (38, 110, 1.5),
            (42, 121, 7.0),
            (0, 121, -0.8),
            (42, 108, 2.0),
            (46, 111, -3.0),
            (47, 111, 0.8),
            (3, 101, 3.0),
            (23, 122, 3.0),
        ]:
            existing = next(
                (
                    e
                    for e in brain["connections"]
                    if e["source"] == lookup[source]["id"]
                    and e["target"] == lookup[target]["id"]
                ),
                None,
            )
            if existing:
                existing["weight"] = w
            else:
                brain["connections"].append(
                    {
                        "id": brain["next_connection"],
                        "source": lookup[source]["id"],
                        "target": lookup[target]["id"],
                        "weight": w,
                    }
                )
                brain["next_connection"] += 1


def social_scenario(rules, seed=73, defense=False):
    rules, config = ecosystem_preset(rules, seed, terrain=False)
    rules.update(world_width=24.0, world_height=16.0, grid_width=24, grid_height=16)
    config["cell_overrides"] = [
        c for c in config["cell_overrides"] if c["column"] < 24 and c["row"] < 16
    ]
    config["producers"].update(
        count=12,
        positions=[
            {"x": 3.0 + 3 * (i % 4), "y": [3.0, 5.0, 13.0][i // 4]} for i in range(12)
        ],
    )
    herb = deepcopy(config["populations"][0])
    predator = deepcopy(config["populations"][1])
    if defense:
        config["producers"].update(count=0, positions=[])
        herb.update(
            count=1,
            tissue_nutrient=24.2,
            reserve_energy=60.0,
            positions=[{"x": 10.0, "y": 8.0}],
        )
        herb["genome"]["body"]["modules"][0]["radius"] = 0.8
        herb["genome"]["body"]["circle"]["adaptations"]["offense"] = 1.0
        predator.update(
            count=1,
            tissue_nutrient=3.0,
            reserve_nutrient=1.0,
            reserve_energy=12.0,
            positions=[{"x": 11.5, "y": 8.0}],
        )
        predator["genome"]["body"]["modules"][0]["radius"] = 0.28
    else:
        herb.update(
            count=8,
            positions=[
                {"x": 8.0 + (i % 4) * 1.2, "y": 7.0 + (i // 4) * 1.2} for i in range(8)
            ],
        )
        predator.update(
            count=2, positions=[{"x": 16.0, "y": 7.0}, {"x": 16.0, "y": 10.0}]
        )
    config["populations"] = [herb, predator]
    config["ecology"].update(reproduction=False, max_population=64)
    fund(rules, config)
    return rules, config
