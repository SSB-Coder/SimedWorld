import math
from copy import copy

import numpy as np
import torch

from .packing import compiled_graph, graph_signature


class NeuralPopulation:
    kind = "sparse_leaky_coo_v1"

    def __init__(self, frame: dict, device="cpu", previous=None, memory=None):
        self.device = torch.device(device)
        if self.device.type not in {"cpu", "cuda"}:
            raise ValueError("supported neural devices are cpu and cuda")
        if self.device.type == "cuda" and not torch.cuda.is_available():
            raise RuntimeError("CUDA was requested but is unavailable")
        self.tick = frame["tick"]
        self.signature = self._signature(frame)
        self.keys = [
            (a["id"], n["id"]) for a in frame["agents"] for n in a["brain"]["nodes"]
        ]
        if len(set(self.keys)) != len(self.keys):
            raise ValueError("duplicate neural identity")
        self.index = {key: i for i, key in enumerate(self.keys)}
        self.rows = []
        sources, targets, weights, biases, leaks, input_indices = [], [], [], [], [], []
        offset = 0
        for agent in frame["agents"]:
            nodes = agent["brain"]["nodes"]
            packed = compiled_graph(agent["brain"])
            inputs = {
                n["port"]: self.index[(agent["id"], n["id"])]
                for n in nodes
                if n["kind"] == "input"
            }
            outputs = {
                n["port"]: self.index[(agent["id"], n["id"])]
                for n in nodes
                if n["kind"] == "output"
            }
            if set(inputs) != {p["port"] for p in agent["inputs"]} or set(outputs) != {
                p["port"] for p in agent["outputs"]
            }:
                raise ValueError("body and neural ports disagree")
            self.rows.append(
                (agent["id"], offset, offset + len(nodes), inputs, outputs)
            )
            biases.append(packed["biases"])
            leaks.append(packed["leaks"])
            input_indices.append(packed["inputs"] + offset)
            sources.append(packed["sources"] + offset)
            targets.append(packed["targets"] + offset)
            weights.append(packed["weights"])
            offset += len(nodes)

        def tensor(blocks, dtype):
            values = np.concatenate(blocks) if blocks else np.empty(0, dtype=dtype)
            return torch.from_numpy(values).to(self.device)

        self.sources = tensor(sources, np.int64)
        self.targets = tensor(targets, np.int64)
        self.weights = tensor(weights, np.float64)
        self.biases = tensor(biases, np.float64)
        self.leaks = tensor(leaks, np.float64)
        self.input_indices = tensor(input_indices, np.int64)
        self.memory = torch.zeros(
            len(self.keys), dtype=torch.float64, device=self.device
        )
        if previous is not None:
            source_memory = previous.memory if memory is None else memory
            compatible = [
                (i, previous.index[key])
                for i, key in enumerate(self.keys)
                if key in previous.index
            ]
            if compatible:
                to, source = zip(*compatible)
                self.memory[torch.tensor(to, device=self.device)] = source_memory[
                    torch.tensor(source, device=previous.device)
                ].to(self.device)

    @staticmethod
    def _signature(frame):
        return tuple(
            (a["id"], a["version"], *graph_signature(a["brain"]))
            for a in frame["agents"]
        )

    def validate(self, frame):
        if (
            frame["schema_version"] != 1
            or frame["tick"] != self.tick
            or self._signature(frame) != self.signature
        ):
            raise ValueError("neural population does not match world tick/genomes")
        if (
            self.memory.shape != (len(self.keys),)
            or not torch.isfinite(self.memory).all()
            or torch.any(self.memory.abs() > 1)
        ):
            raise ValueError("invalid recurrent state")

    @torch.inference_mode()
    def propose(self, frame):
        self.validate(frame)
        previous = self.memory.clone()
        indices, observations = [], []
        authorized = torch.zeros(len(self.keys), dtype=torch.bool, device=self.device)
        for agent, (_, begin, end, inputs, _) in zip(
            frame["agents"], self.rows, strict=True
        ):
            authorized[begin:end] = agent["authorized"]
            for value in agent["inputs"]:
                if not math.isfinite(value["value"]) or abs(value["value"]) > 1:
                    raise ValueError("invalid sensory value")
                indices.append(inputs[value["port"]])
                observations.append(value["value"])
        if indices:
            previous[torch.tensor(indices, device=self.device)] = torch.tensor(
                observations, dtype=torch.float64, device=self.device
            )
        incoming = self.biases.clone()
        if self.sources.numel():
            incoming.index_add_(
                0, self.targets, previous.index_select(0, self.sources) * self.weights
            )
        proposed = (1 - self.leaks) * self.memory + self.leaks * incoming.tanh()
        if self.input_indices.numel():
            proposed[self.input_indices] = previous[self.input_indices]
        proposed = torch.where(authorized, proposed, self.memory)
        if not torch.isfinite(proposed).all():
            raise ValueError("non-finite neural proposal")
        output_indices, descriptions = [], []
        for agent, (_, _, _, _, outputs) in zip(
            frame["agents"], self.rows, strict=True
        ):
            for port in agent["outputs"]:
                output_indices.append(outputs[port["port"]])
                descriptions.append((agent["id"], port))
        raw = (
            proposed[torch.tensor(output_indices, dtype=torch.long, device=self.device)]
            .cpu()
            .tolist()
        )
        values = {a["id"]: [] for a in frame["agents"]}
        for value, (identity, port) in zip(raw, descriptions, strict=True):
            value = value if port["channel"] in {102, 111} else max(0.0, value)
            values[identity].append(
                {"port": port["port"], "value": value if port["active"] else 0.0}
            )
        batch = {
            "schema_version": 1,
            "tick": self.tick,
            "agents": [
                {"id": a["id"], "version": a["version"], "values": values[a["id"]]}
                for a in frame["agents"]
            ],
        }
        return batch, proposed

    def reconcile(self, frame, memory):
        if self._signature(frame) == self.signature:
            result = copy(self)
            result.tick = frame["tick"]
            result.memory = memory
            result.validate(frame)
            return result
        result = type(self)(frame, self.device, self, memory)
        result.validate(frame)
        return result

    def snapshot(self):
        return {
            "schema_version": 1,
            "kind": self.kind,
            "tick": self.tick,
            "keys": [list(key) for key in self.keys],
            "memory": self.memory.cpu().tolist(),
        }

    @classmethod
    def restore(cls, frame, payload, device="cpu"):
        if not isinstance(payload, dict) or set(payload) != {
            "schema_version",
            "kind",
            "tick",
            "keys",
            "memory",
        }:
            raise ValueError("unexpected neural checkpoint fields")
        if (
            type(payload["schema_version"]) is not int
            or payload["schema_version"] != 1
            or payload["kind"] != cls.kind
            or type(payload["tick"]) is not int
            or payload["tick"] != frame["tick"]
        ):
            raise ValueError("unsupported or mismatched neural checkpoint")
        result = cls(frame, device)
        for key in payload["keys"]:
            if (
                not isinstance(key, list)
                or len(key) != 2
                or any(type(v) is not int or not 0 < v <= 2**64 - 1 for v in key)
            ):
                raise ValueError("invalid neural identity")
        keys = [tuple(key) for key in payload["keys"]]
        if keys != result.keys or len(payload["memory"]) != len(keys):
            raise ValueError("neural checkpoint identities do not match world")
        if any(
            type(v) not in {int, float} or not math.isfinite(v) or abs(v) > 1
            for v in payload["memory"]
        ):
            raise ValueError("invalid neural checkpoint state")
        result.memory = torch.tensor(
            payload["memory"], dtype=torch.float64, device=result.device
        )
        result.validate(frame)
        return result


def reference_update(agent, memory):
    previous = {n["id"]: memory.get(n["id"], 0.0) for n in agent["brain"]["nodes"]}
    if not agent["authorized"]:
        return previous
    inputs = {v["port"]: v["value"] for v in agent["inputs"]}
    for node in agent["brain"]["nodes"]:
        if node["kind"] == "input":
            previous[node["id"]] = inputs[node["port"]]
    result = {}
    for node in agent["brain"]["nodes"]:
        if node["kind"] == "input":
            result[node["id"]] = previous[node["id"]]
        else:
            value = node["bias"] + sum(
                previous[e["source"]] * e["weight"]
                for e in agent["brain"]["connections"]
                if e["target"] == node["id"]
            )
            result[node["id"]] = (1 - node["leak"]) * memory.get(
                node["id"], 0.0
            ) + node["leak"] * math.tanh(value)
    return result
