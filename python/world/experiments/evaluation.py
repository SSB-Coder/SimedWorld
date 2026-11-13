"""Held-out evaluation of copied subjects; scores do not enter the ecosystem."""

from copy import deepcopy
import math

from ..archive import digest, read_subject
from ..ecosystem import EcosystemRuntime
from .ecosystem import ecosystem_preset
from .adaptation import fund


def memory_probe(subject, delays=(1, 4, 12)):
    from ..brains.plastic import PlasticPopulation
    from ..brains.liquid import LIQUID_FAMILIES
    import torch

    graph = subject["expressed_graph"]
    ports = {
        n["port"] % 256: n["port"] for n in graph["nodes"] if n["port"] is not None
    }
    if 10 not in ports or 111 not in ports:
        return {"available": False}
    family = subject.get("controller_kind", PlasticPopulation.kind)
    if family not in {
        PlasticPopulation.kind,
        *LIQUID_FAMILIES,
        "sparse_leaky_coo_v1",
    }:
        return {"available": False, "reason": "unsupported controller dynamics"}
    cls = LIQUID_FAMILIES.get(family, PlasticPopulation)
    rules = subject["origin"]["rules"]
    agent = {
        "id": 1,
        "version": graph["version"],
        "brain": graph,
        "authorized": True,
        "learning_rate": 0.0,
        "reward": 0.0,
        "inherit": False,
        "inputs": [
            {"port": n["port"], "value": 0.0}
            for n in graph["nodes"]
            if n["kind"] == "input"
        ],
        "outputs": [
            {"port": n["port"], "channel": n["port"] % 256, "active": True}
            for n in graph["nodes"]
            if n["kind"] == "output"
        ],
    }
    results = []
    for delay in delays:
        responses = []
        for cue in [-1, 1]:
            frame = {
                "schema_version": 1,
                "seed": 73,
                "tick": 0,
                "duration": rules["physics_dt"] * rules["ecology_interval_ticks"],
                "agents": [deepcopy(agent)],
            }
            brain = cls(frame)
            brain.noise = lambda tick: torch.zeros_like(brain.memory)
            for tick in range(delay + 1):
                for value in frame["agents"][0]["inputs"]:
                    value["value"] = (
                        float(cue) if tick == 0 and value["port"] == ports[10] else 0.0
                    )
                _, proposal = brain.propose(frame)
                brain.memory = proposal.memory
            responses.append(
                float(
                    brain.memory[
                        brain.index[
                            (
                                1,
                                next(
                                    n["id"]
                                    for n in graph["nodes"]
                                    if n["port"] == ports[111]
                                ),
                            )
                        ]
                    ]
                )
                * cue
            )
        results.append(
            {
                "delay": delay,
                "signed_response": sum(responses) / 2,
                "correct": sum(v > 0 for v in responses) / 2,
            }
        )
    return {
        "available": True,
        "controller": family,
        "task": "signed food-bearing pulse, then blank observations; frozen weights and zero initial memory",
        "trials": results,
    }


def evaluate_subject(path, suite, device="cpu", progress=None):
    subject = read_subject(path)
    if (
        suite.get("schema_version") != 1
        or not suite.get("seeds")
        or not suite.get("tasks")
    ):
        raise ValueError("invalid evaluation suite")
    tasks = set(suite["tasks"])
    if not tasks <= {"foraging", "risk", "transfer", "learning", "memory"}:
        raise ValueError("unknown evaluation task")
    decisions = suite.get("decisions", 300)
    if type(decisions) is not int or not 1 <= decisions <= 1_000_000:
        raise ValueError("invalid evaluation duration")
    origin = subject["origin"]
    results = []
    for task in sorted(tasks - {"memory"}):
        for seed in suite["seeds"]:
            for control in ["learned", "birth_weights", "fixed_learning"]:
                rules, config = ecosystem_preset(origin["rules"], seed=seed)
                config["ecology"]["reproduction"] = False
                config["evolution"]["mutation"].update(
                    parameter_edits=0,
                    body_edits=0,
                    body_structure_rate=0,
                    neural_structure_rate=0,
                    brain_creation_rate=0,
                )
                genome = deepcopy(origin["organism"]["genome"])
                if control != "birth_weights":
                    genome["brain"] = deepcopy(subject["expressed_graph"])
                if control == "fixed_learning":
                    genome["body"]["circle"]["learning_rate"] = 0.0
                group = config["populations"][0]
                group["count"] = 1
                group["genome"] = genome
                group["tissue_nutrient"] = max(
                    rules["maturity_tissue_nutrient"],
                    sum(
                        math.pi * m["radius"] ** 2 * genome["body"]["density"] * 12
                        for m in genome["body"]["modules"]
                    ),
                )
                group["reserve_nutrient"], group["reserve_energy"] = (
                    min(
                        3.0,
                        group["tissue_nutrient"]
                        * rules["reserve_nutrient_capacity_per_tissue_nutrient"],
                    ),
                    min(
                        24.0,
                        group["tissue_nutrient"]
                        * rules["reserve_energy_capacity_per_tissue_nutrient"],
                    ),
                )
                if task == "transfer":
                    config["light_gradient"] = 0.8
                    config["light_energy_per_cell_per_time"] = 3.0
                if task == "risk":
                    for predator in config["populations"][1:]:
                        if predator["genome"]["body"]["diet"] == "predator":
                            predator["count"] *= 2
                if task == "learning":
                    config["ecology"]["perception_radius"] = 3.0
                fund(rules, config)
                runtime = EcosystemRuntime.create(
                    rules,
                    config,
                    device,
                    {
                        "plastic_sparse_ltc_v1": "ltc_v1",
                        "plastic_sparse_ltc_v2": "ltc_v2",
                        "plastic_sparse_ltc_v3": "ltc",
                    }.get(subject.get("controller_kind"), "sparse"),
                )
                identity = config["producers"]["count"] + 1
                runtime.advance(decisions)
                final = next(
                    (
                        o
                        for o in runtime.display(identity)["organisms"]
                        if o["id"] == identity
                    ),
                    None,
                )
                results.append(
                    {
                        "task": task,
                        "seed": seed,
                        "control": control,
                        "subject_id": identity,
                        "alive": bool(final and final["alive"]),
                        "energy": final["energy"] if final else 0.0,
                        "integrity": final["integrity"] if final else 0.0,
                        "model_time": runtime.elapsed_time,
                        "food_ingested": runtime.telemetry()["food_web"]["intake"].get(
                            str(identity), 0.0
                        ),
                        "world_feeding_events": runtime.summary()["ecology"][
                            "feeding_events"
                        ],
                        "audit": runtime.audit(),
                    }
                )
                if progress is not None:
                    progress(results[-1])
    return {
        "schema_version": 1,
        "kind": "world.evaluation",
        "suite_sha256": digest(suite),
        "subject_sha256": subject["sha256"],
        "results": results,
        "memory": memory_probe(subject) if "memory" in tasks else None,
        "limitations": [
            "finite declared tasks; no general intelligence score",
            "risk trial changes the ecological opponent, not the subject body",
            "growth topology and plasticity remain enabled unless explicitly controlled",
        ],
    }
