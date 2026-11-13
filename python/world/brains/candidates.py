from copy import deepcopy
import math
import time

import torch
from ncps.torch import CfC, LTC
from ncps.wirings import FullyConnected


class LiquidCandidate:
    def __init__(self, family, inputs, outputs, units=16, seed=73, device="cpu"):
        if family not in {"cfc", "ltc"} or inputs < 1 or outputs < 1 or units < outputs:
            raise ValueError("invalid liquid candidate interface")
        self.spec = {"family": family, "inputs": inputs, "outputs": outputs, "units": units, "seed": seed}
        self.device = torch.device(device)
        if self.device.type not in {"cpu", "cuda"} or self.device.type == "cuda" and not torch.cuda.is_available():
            raise ValueError("unsupported or unavailable candidate device")
        with torch.random.fork_rng(devices=[]):
            torch.manual_seed(seed)
            wiring = FullyConnected(units, output_dim=outputs, erev_init_seed=seed)
            if family == "cfc":
                self.model = CfC(inputs, wiring, return_sequences=False)
            else:
                self.model = LTC(inputs, wiring, return_sequences=False)
        self.model = self.model.to(device=self.device, dtype=torch.float64).eval()
        self.memory = None

    @torch.inference_mode()
    def propose(self, agent, duration):
        values = [v["value"] for v in agent["inputs"]]
        if len(values) != self.spec["inputs"] or len(agent["outputs"]) != self.spec["outputs"] or not math.isfinite(duration) or duration <= 0:
            raise ValueError("candidate input/output interface mismatch")
        if not agent["authorized"]:
            return [{"port": p["port"], "value": 0.0} for p in agent["outputs"]], self.memory
        inputs = torch.tensor(values, dtype=torch.float64, device=self.device).reshape(1, 1, -1)
        times = torch.tensor([[duration]], dtype=torch.float64, device=self.device)
        output, memory = self.model(inputs, hx=self.memory, timespans=times)
        if output.shape[-1] < self.spec["outputs"]:
            raise ValueError("candidate returned fewer motor values than declared")
        output = output[..., :self.spec["outputs"]]
        raw = output.tanh().reshape(-1).cpu().tolist()
        if not all(math.isfinite(v) for v in raw) or not torch.isfinite(memory).all():
            raise ValueError("non-finite liquid candidate result")
        actions = []
        for value, port in zip(raw, agent["outputs"], strict=True):
            value = value if port["channel"] in {102, 111} else max(0.0, value)
            actions.append({"port": port["port"], "value": value if port["active"] else 0.0})
        return actions, memory

    def snapshot(self):
        return {"schema_version": 1, "kind": "fixed_liquid_candidate", "spec": deepcopy(self.spec),
                "parameters": {name: {"shape": list(value.shape), "values": value.cpu().tolist()}
                               for name, value in self.model.state_dict().items()},
                "memory": None if self.memory is None else self.memory.cpu().tolist()}

    @classmethod
    def restore(cls, payload, device="cpu", newborn=False):
        if not isinstance(payload, dict) or set(payload) != {"schema_version", "kind", "spec", "parameters", "memory"} or type(payload["schema_version"]) is not int or payload["schema_version"] != 1 or payload["kind"] != "fixed_liquid_candidate":
            raise ValueError("invalid liquid candidate checkpoint")
        if set(payload["spec"]) != {"family", "inputs", "outputs", "units", "seed"}:
            raise ValueError("invalid liquid candidate specification")
        result = cls(**payload["spec"], device=device)
        expected = result.model.state_dict()
        if set(payload["parameters"]) != set(expected):
            raise ValueError("liquid parameter identities mismatch")
        parameters = {}
        for name, tensor in expected.items():
            item = payload["parameters"][name]
            value = torch.tensor(item["values"], dtype=tensor.dtype, device=result.device)
            if item["shape"] != list(tensor.shape) or value.shape != tensor.shape or not torch.isfinite(value).all():
                raise ValueError("invalid liquid parameter shape/value")
            parameters[name] = value
        result.model.load_state_dict(parameters, strict=True)
        if payload["memory"] is not None and not newborn:
            value = torch.tensor(payload["memory"], dtype=torch.float64, device=result.device)
            if value.shape != (1, result.spec["units"]) or not torch.isfinite(value).all():
                raise ValueError("invalid liquid candidate memory")
            result.memory = value
        return result


def compare_candidates(frame, iterations=100, device="cpu"):
    if iterations < 1 or not frame["agents"]:
        raise ValueError("candidate comparison needs active observations and positive iterations")
    agent = frame["agents"][0]
    reports = []
    for family in ["cfc", "ltc"]:
        candidate = LiquidCandidate(family, len(agent["inputs"]), len(agent["outputs"]), units=16, device=device)
        start = time.perf_counter()
        for _ in range(iterations):
            values, state = candidate.propose(agent, frame["duration"])
            candidate.memory = state
        if candidate.device.type == "cuda":
            torch.cuda.synchronize(candidate.device)
        elapsed = time.perf_counter() - start
        snapshot = candidate.snapshot()
        inherited = LiquidCandidate.restore(snapshot, device, newborn=True)
        restored = LiquidCandidate.restore(snapshot, device)
        a, _ = candidate.propose(agent, frame["duration"])
        b, _ = restored.propose(agent, frame["duration"])
        reports.append({"family": family, "device": device, "iterations": iterations,
                        "inputs": len(agent["inputs"]), "outputs": len(agent["outputs"]),
                        "units": 16, "parameter_count": sum(p.numel() for p in candidate.model.parameters()),
                        "wall_seconds": elapsed, "bounded_actions": values,
                        "inherited_parameters_equal": all(torch.equal(v, inherited.model.state_dict()[k]) for k, v in candidate.model.state_dict().items()),
                        "newborn_memory_reset": inherited.memory is None, "restored_next_actions_equal": a == b})
    return {"schema_version": 1, "scope": "Fixed wired CfC/LTC interface, parameter inheritance, continuation, and compute comparison on the same local observations. These candidates are not yet topology-mutable ecological controllers; the evolving baseline is sparse leaky recurrence.", "reports": reports}
