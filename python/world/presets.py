import math
from copy import deepcopy


def body_blueprint(kind="grazer"):
    def module(identity, parent, category, radius, angle=0.0, investment=1.0):
        return {
            "id": identity,
            "parent": parent,
            "kind": category,
            "radius": radius,
            "angle": angle,
            "orientation": 0.0,
            "investment": investment,
        }

    if kind == "producer":
        modules = [
            module(1, None, "core", 0.30),
            module(2, 1, "photosynthetic", 0.26, math.pi / 2),
        ]
    else:
        modules = [
            module(1, None, "core", 0.27),
            module(2, 1, "mouth", 0.16),
            module(3, 1, "sensor", 0.09, math.pi / 2),
            module(4, 1, "propulsion", 0.14, math.pi),
        ]
        if kind == "armored_grazer":
            modules.append(module(5, 1, "armor", 0.13, -math.pi / 2, 1.5))
        elif kind == "pursuit":
            modules[3]["investment"] = 1.7
            modules[2]["investment"] = 1.3
        elif kind == "ambush":
            modules[1]["investment"] = 2.0
            modules[2]["investment"] = 1.6
        elif kind != "grazer":
            raise ValueError("unknown founder design")
    return {
        "modules": modules,
        "next_id": max(m["id"] for m in modules) + 1,
        "diet": "predator" if kind in {"pursuit", "ambush"} else "grazer",
        "density": 1.0,
    }


def body_ports(body):
    if body.get("circle"):
        root = body["modules"][0]["id"]
        agency = body["circle"].get("adaptations", {}).get("agency", False)
        extra = (
            list(range(30, 48))
            if body["circle"].get("adaptations", {}).get("social_senses", False)
            else []
        )
        foraging = body["circle"].get("adaptations", {}).get("foraging_choices", False)
        if foraging:
            extra += list(range(48, 52))
        if body["circle"].get("adaptations", {}).get("energy_senses", False):
            extra += list(range(52, 56))
        return [
            {"id": root * 256 + c, "channel": c, "module": root, "output": out}
            for out, channels in [
                (
                    False,
                    [
                        0,
                        1,
                        2,
                        3,
                        4,
                        5,
                        6,
                        10,
                        11,
                        12,
                        13,
                        14,
                        15,
                        16,
                        17,
                        19,
                        20,
                        21,
                        22,
                        23,
                        24,
                        25,
                        26,
                        27,
                    ]
                    if agency
                    else [0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20],
                ),
                (
                    True,
                    [
                        100,
                        101,
                        103,
                        104,
                        105,
                        106,
                        107,
                        108,
                        109,
                        110,
                        111,
                        120,
                        121,
                        122,
                        123,
                    ]
                    if agency
                    else [110, 111, 120],
                ),
            ]
            for c in (
                channels + extra if not out else channels + ([124] if foraging else [])
            )
        ]
    channels = {
        "core": ([0, 1, 2, 3, 4, 5, 6], [100, 101, 102]),
        "sensor": ([10, 11, 12, 13, 14, 15, 16, 18], []),
        "mouth": ([], [120, 121]),
        "propulsion": ([], [110, 111]),
    }
    return sorted(
        [
            {
                "id": m["id"] * 256 + channel,
                "channel": channel,
                "module": m["id"],
                "output": output,
            }
            for m in body["modules"]
            for output, sequence in enumerate(channels.get(m["kind"], ([], [])))
            for channel in sequence
        ],
        key=lambda p: p["id"],
    )


def neural_blueprint(body, hidden=8, seed=73, kind="grazer"):
    if type(hidden) is not int or not 0 <= hidden <= 3000:
        raise ValueError("hidden neuron count is out of bounds")
    counter = seed & (2**64 - 1)

    def unit():
        nonlocal counter
        counter = (counter + 0x9E3779B97F4A7C15) & (2**64 - 1)
        value = counter
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & (2**64 - 1)
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & (2**64 - 1)
        return ((value ^ (value >> 31)) >> 11) / 9007199254740992

    nodes, connections = [], []
    edge_lookup = {}
    ports = body_ports(body)
    for port in ports:
        bias = 0.0
        if port["output"]:
            bias = {
                100: 0.4,
                101: 0.0,
                102: 0.0,
                103: 0.8,
                104: -1.8,
                105: -2.0,
                106: 1.0 if kind == "thread" else -2.0,
                107: -2.0,
                108: 0.25,
                109: -0.8,
                110: 0.9,
                111: 0.0,
                120: 1.5,
                121: 1.0 if kind in {"pursuit", "ambush"} else -1.0,
                122: -0.5,
                123: 0.2 if kind in {"ambush", "thread"} else -1.5,
                124: 0.8,
            }[port["channel"]]
            if kind == "ambush" and port["channel"] == 110:
                bias = 0.22
        nodes.append(
            {
                "id": len(nodes) + 1,
                "kind": "output" if port["output"] else "input",
                "port": port["id"],
                "bias": bias,
                "leak": 1.0,
            }
        )
    hidden_nodes = []
    for _ in range(hidden):
        node = {
            "id": len(nodes) + 1,
            "kind": "hidden",
            "port": None,
            "bias": (unit() - 0.5) * 0.06,
            "leak": 0.6,
        }
        nodes.append(node)
        hidden_nodes.append(node)
    lookup = {n["port"]: n for n in nodes if n["port"] is not None}

    def edge(source, target, weight):
        existing = edge_lookup.get((source, target))
        if existing:
            existing["weight"] = weight
        else:
            item = {
                "id": len(connections) + 1,
                "source": source,
                "target": target,
                "weight": weight,
            }
            connections.append(item)
            edge_lookup[(source, target)] = item

    inputs = [n for n in nodes if n["kind"] == "input"]
    outputs = [n for n in nodes if n["kind"] == "output"]
    for index, h in enumerate(hidden_nodes):
        selected_inputs = (
            inputs
            if hidden <= 64
            else [inputs[index % len(inputs)], inputs[(index + 1) % len(inputs)]]
        )
        selected_outputs = (
            outputs
            if hidden <= 64
            else [outputs[index % len(outputs)], outputs[(index + 1) % len(outputs)]]
        )
        for node in selected_inputs:
            edge(node["id"], h["id"], (unit() - 0.5) * 0.12)
        for node in selected_outputs:
            edge(h["id"], node["id"], (unit() - 0.5) * 0.10)
        edge(h["id"], h["id"], 0.1)
    sensors = [m for m in body["modules"] if m["kind"] == "sensor"]
    thrusters = [m for m in body["modules"] if m["kind"] == "propulsion"]
    root = next(m for m in body["modules"] if m["parent"] is None)
    if body.get("circle"):
        sensors = thrusters = [root]
    for prop in thrusters:
        for sensor in sensors:
            if hidden_nodes:
                edge(lookup[sensor["id"] * 256 + 10]["id"], hidden_nodes[0]["id"], 2.0)
                edge(hidden_nodes[0]["id"], lookup[prop["id"] * 256 + 111]["id"], 2.0)
            edge(
                lookup[sensor["id"] * 256 + 12]["id"],
                lookup[prop["id"] * 256 + 110]["id"],
                -0.8 if kind != "ambush" else 1.0,
            )
    if not body.get("circle"):
        edge(
            lookup[root["id"] * 256 + 4]["id"],
            lookup[root["id"] * 256 + 100]["id"],
            -3.0,
        )
        edge(
            lookup[root["id"] * 256 + 3]["id"],
            lookup[root["id"] * 256 + 101]["id"],
            2.0,
        )
    if body.get("circle"):
        turn = lookup[root["id"] * 256 + 111]["id"]
        edge(lookup[root["id"] * 256 + 19]["id"], turn, 1.8)
        edge(lookup[root["id"] * 256 + 20]["id"], turn, 1.0)
    return {
        "version": 1,
        "next_node": len(nodes) + 1,
        "next_connection": len(connections) + 1,
        "nodes": nodes,
        "connections": connections,
    }


def founder_genome(kind="grazer", hidden=8, seed=73, neural=True):
    body = body_blueprint(kind)
    result = {
        "photosynthetic": kind == "producer",
        "can_ingest": kind != "producer",
        "initial_weights": [],
        "body": body,
    }
    if neural and kind != "producer":
        result["brain"] = neural_blueprint(body, hidden, seed, kind)
    return result


def evolution_preset(rules, seed=73, mutation=True, structural=True):
    rules = deepcopy(rules)
    rules.update(world_width=32.0, world_height=32.0, grid_width=32, grid_height=32)
    groups = []
    for index, (kind, count, tissue, nutrient, energy) in enumerate(
        [
            ("producer", 48, 8.0, 3.0, 25.0),
            ("grazer", 10, 6.0, 2.9, 24.0),
            ("armored_grazer", 6, 7.0, 3.0, 26.0),
            ("pursuit", 3, 8.0, 4.0, 30.0),
            ("ambush", 3, 8.0, 4.0, 30.0),
        ]
    ):
        groups.append(
            {
                "count": count,
                "tissue_nutrient": tissue,
                "reserve_nutrient": nutrient,
                "reserve_energy": energy,
                "genome": founder_genome(kind, seed=seed + index),
            }
        )
    initial_n = 3.0 * rules["grid_width"] * rules["grid_height"] + sum(
        g["count"] * (g["tissue_nutrient"] + g["reserve_nutrient"]) for g in groups
    )
    initial_e = sum(
        g["count"]
        * (
            g["tissue_nutrient"] * rules["tissue_energy_per_nutrient"]
            + g["reserve_energy"]
        )
        for g in groups
    )
    config = {
        "schema_version": 1,
        "seed": seed,
        "initial_nutrient": initial_n,
        "initial_energy": initial_e,
        "mineral_nutrient_per_cell": 3.0,
        "light_energy_per_cell_per_time": 8.0,
        "light_gradient": 0.5,
        "producers": groups[0],
        "populations": groups[1:],
        "ecology": {
            "reproduction": True,
            "decomposition": True,
            "birth_radius": 1.8,
            "perception_radius": 3.5,
            "max_population": 192,
        },
        "evolution": {
            "mutation": {
                "parameter_edits": 1.0 if mutation else 0.0,
                "body_edits": 1.0 if mutation else 0.0,
                "body_structure_rate": 0.02 if mutation and structural else 0.0,
                "neural_structure_rate": 0.03 if mutation and structural else 0.0,
            }
        },
    }
    return rules, config


def circle_genome(producer=False, hidden=8, seed=73, neural=True, learning_rate=0.04):
    body = {
        "modules": [
            {
                "id": 1,
                "parent": None,
                "kind": "core",
                "radius": 0.40,
                "angle": 0.0,
                "orientation": 0.0,
                "investment": 1.0,
            }
        ],
        "next_id": 2,
        "diet": "grazer",
        "density": 1.0,
        "circle": {
            "photosynthetic": producer,
            "can_ingest": not producer,
            "speed": 1.0,
            "perception": 1.0,
            "learning_rate": learning_rate,
        },
    }
    result = {
        "photosynthetic": producer,
        "can_ingest": not producer,
        "initial_weights": [],
        "body": body,
    }
    if neural and not producer:
        result["brain"] = neural_blueprint(body, hidden, seed)
    return result


def circle_preset(rules, seed=73, consumers=20, producers=80):
    rules = deepcopy(rules)
    rules.update(
        model_id="circle-world-v0.3",
        world_width=40.0,
        world_height=22.5,
        grid_width=40,
        grid_height=23,
        maintenance_per_tissue_nutrient_per_time=0.025,
        minimum_reproduction_age=8.0,
        reproduction_cooldown=6.0,
        max_ingestion_nutrient_per_time=3.0,
        offspring_reserve_energy=8.0,
    )
    groups = [
        {
            "count": producers,
            "tissue_nutrient": 7.0,
            "reserve_nutrient": 4.0,
            "reserve_energy": 28.0,
            "genome": circle_genome(True),
        },
        {
            "count": consumers,
            "tissue_nutrient": 6.1,
            "reserve_nutrient": 3.0,
            "reserve_energy": 24.0,
            "genome": circle_genome(seed=seed),
        },
    ]
    config = {
        "schema_version": 1,
        "seed": seed,
        "initial_nutrient": 0.0,
        "initial_energy": 0.0,
        "mineral_nutrient_per_cell": 1.0,
        "light_energy_per_cell_per_time": 8.0,
        "light_gradient": 0.2,
        "producers": groups[0],
        "populations": groups[1:],
        "ecology": {
            "reproduction": True,
            "decomposition": True,
            "birth_radius": 1.4,
            "perception_radius": 6.0,
            "consumer_speed": 1.7,
            "movement_energy_per_distance": 0.04,
            "max_population": 512,
            "grazing_residual_fraction": 0.15,
            "mineral_diffusion_per_time": 0.25,
        },
        "evolution": {
            "online_learning": True,
            "maintenance_reserve_time": 24.0,
            "neuron_energy_per_time": 0.0001,
            "connection_energy_per_time": 0.000005,
            "plasticity_energy_per_connection_time": 0.000002,
            "module_upkeep_per_time": 0.003,
            "mutation": {
                "parameter_edits": 1.0,
                "body_edits": 0.5,
                "body_structure_rate": 0.0,
                "neural_structure_rate": 0.08,
                "brain_creation_rate": 0.02,
                "max_nodes": 4096,
            },
        },
    }
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
    return rules, config
