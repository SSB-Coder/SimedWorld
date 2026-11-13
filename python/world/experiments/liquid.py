"""Online, fixed-wiring ncps controls in a physical contact-feeding task."""

from copy import deepcopy
import json
import math

import torch

from .._native import Habitat
from ..brains.candidates import LiquidCandidate
from .learning import feeding_task


def compare_online_liquids(rules, seeds=(73, 89, 107), decisions=200, device="cpu"):
    if type(decisions) is not int or not 1 <= decisions <= 100000:
        raise ValueError("invalid liquid trial duration")
    rows = []
    for family in ["ltc", "cfc"]:
        for seed in seeds:
            for learning in [False, True]:
                trial_rules, config = feeding_task(deepcopy(rules), seed)
                world = Habitat(json.dumps(trial_rules), json.dumps(config))
                frame = json.loads(world.agent_frame_json())
                agent = frame["agents"][0]
                model = LiquidCandidate(
                    family,
                    len(agent["inputs"]),
                    len(agent["outputs"]),
                    seed=seed,
                    device=device,
                )
                parameters = [p for p in model.model.parameters() if p.requires_grad]
                initial = [p.detach().clone() for p in parameters]
                traces = [torch.zeros_like(p) for p in parameters]
                config["evolution"]["connection_energy_per_time"] *= sum(
                    p.numel() for p in parameters
                ) / max(1, len(agent["brain"]["connections"]))
                config["evolution"]["plasticity_energy_per_connection_time"] *= sum(
                    p.numel() for p in parameters
                ) / max(1, len(agent["brain"]["connections"]))
                config["evolution"]["neuron_energy_per_time"] *= 16 / len(
                    agent["brain"]["nodes"]
                )
                world = Habitat(json.dumps(trial_rules), json.dumps(config))
                baseline = 0.0
                generator = torch.Generator(device=device).manual_seed(seed)
                feed = next(
                    i for i, p in enumerate(agent["outputs"]) if p["channel"] == 120
                )
                for _ in range(decisions):
                    frame = json.loads(world.agent_frame_json())
                    agent = frame["agents"][0]
                    if not agent["authorized"]:
                        break
                    before = json.loads(world.snapshot_json())["organisms"][1][
                        "organism"
                    ]["reserves"]["energy"]
                    x = torch.tensor(
                        [[[v["value"] for v in agent["inputs"]]]],
                        dtype=torch.float64,
                        device=device,
                    )
                    dt = torch.tensor(
                        [[frame["duration"]]], dtype=torch.float64, device=device
                    )
                    output, state = model.model(x, hx=model.memory, timespans=dt)
                    mean = output.reshape(-1)[feed]
                    sigma = 0.15
                    sample = mean.detach() + sigma * torch.randn(
                        (), generator=generator, device=device, dtype=torch.float64
                    )
                    probability = torch.distributions.Normal(mean, sigma).log_prob(
                        sample
                    )
                    gradients = torch.autograd.grad(
                        probability, parameters, allow_unused=True
                    )
                    values = [
                        {
                            "port": p["port"],
                            "value": float(sample.tanh().clamp(0, 1))
                            if i == feed and p["active"]
                            else 0.0,
                        }
                        for i, p in enumerate(agent["outputs"])
                    ]
                    world.advance_controlled(
                        json.dumps(
                            {
                                "schema_version": 1,
                                "tick": frame["tick"],
                                "agents": [
                                    {
                                        "id": agent["id"],
                                        "version": agent["version"],
                                        "values": values,
                                    }
                                ],
                            }
                        )
                    )
                    model.memory = state.detach()
                    after = json.loads(world.snapshot_json())["organisms"][1][
                        "organism"
                    ]["reserves"]["energy"]
                    reward = max(-1.0, min(1.0, after - before))
                    advantage = reward - baseline
                    baseline += 0.02 * advantage
                    with torch.no_grad():
                        for p, trace, g in zip(
                            parameters, traces, gradients, strict=True
                        ):
                            trace.mul_(math.exp(-frame["duration"] / 6.0))
                            if g is not None:
                                trace.add_(
                                    g, alpha=1 - math.exp(-frame["duration"] / 6.0)
                                ).clamp_(-10, 10)
                            if learning:
                                p.add_(
                                    (0.01 * advantage * trace).clamp(-0.03, 0.03)
                                ).clamp_(-8, 8)
                    if not json.loads(world.snapshot_json())["organisms"][1][
                        "organism"
                    ]["alive"]:
                        break
                saved = model.snapshot()
                inherited = LiquidCandidate.restore(saved, device, newborn=True)
                restored = LiquidCandidate.restore(saved, device)
                current = json.loads(world.agent_frame_json())
                equal = True
                if current["agents"]:
                    a, _ = model.propose(current["agents"][0], current["duration"])
                    b, _ = restored.propose(current["agents"][0], current["duration"])
                    equal = a == b
                rows.append(
                    {
                        "family": family,
                        "seed": seed,
                        "learning": learning,
                        "tick": world.tick,
                        "parameter_change": sum(
                            float((p.detach() - old).abs().sum())
                            for p, old in zip(parameters, initial, strict=True)
                        ),
                        "food_ingested": json.loads(world.summary_json())["ecology"][
                            "ingested_nutrient"
                        ],
                        "newborn_memory_reset": inherited.memory is None,
                        "inherited_parameters_equal": all(
                            torch.equal(v, inherited.model.state_dict()[k])
                            for k, v in model.model.state_dict().items()
                        ),
                        "restored_next_actions_equal": equal,
                        "audit": json.loads(world.audit_json()),
                    }
                )
    return {
        "scope": "Actual ncps LTC/CfC, independent parameters, online perturbation learning, fixed topology, contact feeding only; movement held at zero as a declared task constraint. Parameter-count energy charge. Not a full ecological topology-mutable backend or proof of intelligence.",
        "trials": rows,
    }
