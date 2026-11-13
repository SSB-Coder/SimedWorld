from copy import copy
from dataclasses import dataclass
import math

import numpy as np
import torch

from .evolving import NeuralPopulation
from .packing import named_edges


@dataclass
class Proposal:
    memory: torch.Tensor
    edges: torch.Tensor
    biases: torch.Tensor
    authorized: set
    activity: torch.Tensor | None = None
    features: dict | None = None


class PlasticPopulation(NeuralPopulation):
    kind = "plastic_sparse_rnn_v1"
    exploration = 0.18
    needs_affine_input = True

    def __init__(self, frame, device="cpu", previous=None, memory=None):
        super().__init__(frame, device, previous, memory)
        self.seed = frame["seed"]
        self.exploration = frame.get("exploration", 0.18)
        self.trace_time = frame.get("trace_time", 1.5)
        self.temporal_feedback = frame.get("temporal_feedback", False)
        self.plasticity_decay = frame.get("plasticity_decay_per_time", 0.0)
        self.inherited_weights = self.weights.clone()
        self.inherited_biases = self.biases.clone()
        self.activity = (
            torch.zeros_like(self.memory)
            if frame.get("embodied_ecology", False)
            else None
        )
        self.critic = (
            {a["id"]: [0.0] * 9 for a in frame["agents"]}
            if self.temporal_feedback
            else {}
        )
        self.edge_keys = [
            key
            for a in frame["agents"]
            for key in named_edges(a["brain"], a["id"])
        ]
        self.edge_index = {key: i for i, key in enumerate(self.edge_keys)}
        self.edge_trace = torch.zeros_like(self.weights)
        self.bias_trace = torch.zeros_like(self.biases)
        self.baseline = {a["id"]: 0.0 for a in frame["agents"]}
        self.changes = {a["id"]: 0.0 for a in frame["agents"]}
        self.updates = {a["id"]: 0 for a in frame["agents"]}
        owners = {a["id"]: i for i, a in enumerate(frame["agents"])}
        self.node_owners = torch.tensor(
            [owners[k[0]] for k in self.keys], dtype=torch.long, device=self.device
        )
        self.edge_owners = torch.tensor(
            [owners[k[0]] for k in self.edge_keys], dtype=torch.long, device=self.device
        )
        self.noise_ids = np.asarray(self.keys, dtype=np.uint64).reshape(-1, 2)
        self.parameter_edge_ids = np.asarray(self.edge_keys, dtype=np.uint64).reshape(-1, 2)
        self.noise_ids.setflags(write=False)
        self.parameter_edge_ids.setflags(write=False)

    def noise(self, tick):
        with np.errstate(over="ignore"):
            value = (
                self.noise_ids[:, 0] * np.uint64(0x9E3779B97F4A7C15)
                ^ self.noise_ids[:, 1] * np.uint64(0xD1B54A32D192ED03)
                ^ np.uint64((self.seed + tick * 0x94D049BB133111EB) % (2**64))
            )

            def mix(data):
                data = (data ^ (data >> np.uint64(30))) * np.uint64(0xBF58476D1CE4E5B9)
                data = (data ^ (data >> np.uint64(27))) * np.uint64(0x94D049BB133111EB)
                return data ^ (data >> np.uint64(31))

            a = mix(value)
            b = mix(value + np.uint64(0x9E3779B97F4A7C15))
        u = ((a >> np.uint64(11)).astype(np.float64) + 0.5) / 9007199254740992
        v = ((b >> np.uint64(11)).astype(np.float64) + 0.5) / 9007199254740992
        return (
            torch.from_numpy(np.sqrt(-2 * np.log(u)) * np.cos(2 * math.pi * v)).to(
                self.device
            )
            * self.exploration
        )

    def validate(self, frame):
        super().validate(frame)
        if (
            self.weights.shape != (len(self.edge_keys),)
            or self.edge_trace.shape != self.weights.shape
            or self.biases.shape != (len(self.keys),)
            or self.bias_trace.shape != self.biases.shape
        ):
            raise ValueError("plastic state shape mismatch")
        if self.activity is not None and (
            self.activity.shape != self.memory.shape
            or not torch.isfinite(self.activity).all()
            or torch.any((self.activity < 0) | (self.activity > 1))
        ):
            raise ValueError("invalid activity history")
        if self.temporal_feedback and (
            set(self.critic) != set(self.baseline)
            or any(
                len(v) != 9 or any(not math.isfinite(x) or abs(x) > 8 for x in v)
                for v in self.critic.values()
            )
        ):
            raise ValueError("invalid temporal value state")
        for values, bound in [
            (self.weights, 8),
            (self.biases, 8),
            (self.edge_trace, 10),
            (self.bias_trace, 10),
        ]:
            if not torch.isfinite(values).all() or torch.any(values.abs() > bound):
                raise ValueError("invalid plastic controller state")

    @staticmethod
    def features(agent):
        inputs = {v["port"] % 256: v["value"] for v in agent["inputs"]}
        return [1.0] + [inputs.get(c, 0.0) for c in [0, 1, 2, 3, 4, 12, 22, 27]]

    def transition(self, previous, incoming, noise, duration):
        return (1 - self.leaks) * self.memory + self.leaks * (incoming + noise).tanh()

    def eligibility_features(self, previous):
        return previous[self.sources]

    @torch.inference_mode()
    def propose(self, frame, *, packed=False):
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
        incoming = None
        if self.needs_affine_input:
            incoming = self.biases.clone()
            if self.sources.numel():
                incoming.index_add_(0, self.targets, previous[self.sources] * self.weights)
        noise = self.noise(frame["tick"])
        noise[self.input_indices] = 0
        noise = torch.where(authorized, noise, 0)
        memory = self.transition(previous, incoming, noise, frame["duration"])
        memory[self.input_indices] = previous[self.input_indices]
        memory = torch.where(authorized, memory, self.memory)
        decay = math.exp(-frame["duration"] / self.trace_time)
        score = noise / self.exploration**2
        edge_trace = (
            decay * self.edge_trace
            + (1 - decay) * score[self.targets] * self.eligibility_features(previous)
        ).clamp(-10, 10)
        bias_trace = (decay * self.bias_trace + (1 - decay) * score).clamp(-10, 10)
        bias_trace[self.input_indices] = 0
        values = memory.cpu().tolist()
        inherited_weights = (
            self.weights.cpu().tolist()
            if not packed and any(a["inherit"] for a in frame["agents"])
            else None
        )
        inherited_biases = (
            self.biases.cpu().tolist() if inherited_weights is not None else None
        )
        activity = (
            None
            if self.activity is None
            else torch.where(
                authorized, 0.98 * self.activity + 0.02 * memory.abs(), self.activity
            )
        )
        development = frame.get("development_due", False)
        activity_values = (
            activity.cpu().tolist() if activity is not None and development else None
        )
        rows = []
        for agent, (_, begin, end, _, outputs) in zip(
            frame["agents"], self.rows, strict=True
        ):
            output = []
            for port in agent["outputs"]:
                value = values[outputs[port["port"]]]
                if port["channel"] != 111:
                    value = max(0.0, value)
                output.append(
                    {"port": port["port"], "value": value if port["active"] else 0.0}
                )
            row = {"id": agent["id"], "version": agent["version"], "values": output}
            if activity_values is not None:
                row["activity"] = [
                    [n["id"], activity_values[self.index[(agent["id"], n["id"])]]]
                    for n in agent["brain"]["nodes"]
                ]
            if agent["inherit"] and not packed:
                row["learned"] = {
                    "weights": [
                        [
                            e["id"],
                            inherited_weights[self.edge_index[(agent["id"], e["id"])]],
                        ]
                        for e in agent["brain"]["connections"]
                    ],
                    "biases": [
                        [n["id"], inherited_biases[self.index[(agent["id"], n["id"])]]]
                        for n in agent["brain"]["nodes"]
                    ],
                }
            rows.append(row)
        return {"schema_version": 1, "tick": self.tick, "agents": rows}, Proposal(
            memory,
            edge_trace,
            bias_trace,
            {a["id"] for a in frame["agents"] if a["authorized"]},
            activity,
            {a["id"]: self.features(a) for a in frame["agents"]}
            if self.temporal_feedback
            else None,
        )

    @torch.inference_mode()
    def reconcile(self, frame, proposal):
        if self._signature(frame) == self.signature:
            result = copy(self)
            result.tick = frame["tick"]
            result.memory = proposal.memory
            result.activity = proposal.activity
            result.weights, result.biases = self.weights.clone(), self.biases.clone()
            result.edge_trace, result.bias_trace = (
                proposal.edges.clone(),
                proposal.biases.clone(),
            )
            result.baseline, result.changes, result.updates = (
                self.baseline.copy(),
                self.changes.copy(),
                self.updates.copy(),
            )
        else:
            result = type(self)(frame, self.device, self, proposal.memory)
            for destination, origin in [
                (result.edge_keys, self.edge_index),
                (result.keys, self.index),
            ]:
                matching = [
                    (i, origin[key])
                    for i, key in enumerate(destination)
                    if key in origin
                ]
                if matching:
                    to, source = zip(*matching)
                    to = torch.tensor(to, device=self.device)
                    source = torch.tensor(source, device=self.device)
                    if destination is result.edge_keys:
                        result.weights[to], result.edge_trace[to] = (
                            self.weights[source],
                            proposal.edges[source],
                        )
                    else:
                        if (
                            result.activity is not None
                            and proposal.activity is not None
                        ):
                            result.activity[to] = proposal.activity[source]
                        result.biases[to], result.bias_trace[to] = (
                            self.biases[source],
                            proposal.biases[source],
                        )
            for identity in result.baseline.keys() & self.baseline.keys():
                (
                    result.baseline[identity],
                    result.changes[identity],
                    result.updates[identity],
                ) = (
                    self.baseline[identity],
                    self.changes[identity],
                    self.updates[identity],
                )
        result.critic = (
            {
                a["id"]: self.critic.get(a["id"], [0.0] * 9).copy()
                for a in frame["agents"]
            }
            if self.temporal_feedback
            else {}
        )
        rates, advantages = [], []
        for agent in frame["agents"]:
            identity = agent["id"]
            if identity not in proposal.authorized:
                rates.append(0.0)
                advantages.append(0.0)
                continue
            reward = max(-1.0, min(1.0, agent["reward"]))
            rate = agent["learning_rate"]
            advantage = reward - result.baseline[identity]
            if (
                self.temporal_feedback
                and proposal.features
                and identity in proposal.features
            ):
                before = proposal.features[identity]
                after = self.features(agent)
                weights = result.critic[identity]
                advantage = (
                    reward
                    + math.exp(-frame["duration"] / 30.0)
                    * sum(w * x for w, x in zip(weights, after, strict=True))
                    - sum(w * x for w, x in zip(weights, before, strict=True))
                )
                if rate:
                    result.critic[identity] = [
                        max(
                            -8.0,
                            min(8.0, w + 0.01 * max(-1.0, min(1.0, advantage)) * x),
                        )
                        for w, x in zip(weights, before, strict=True)
                    ]
            result.baseline[identity] += 0.02 * (reward - result.baseline[identity])
            advantage = max(-1.0, min(1.0, advantage))
            rates.append(rate)
            advantages.append(advantage)
            if rate:
                result.updates[identity] += 1
        scale = torch.tensor(
            [r * a for r, a in zip(rates, advantages, strict=True)],
            dtype=torch.float64,
            device=self.device,
        )
        weight_delta = (scale[result.edge_owners] * result.edge_trace).clamp(
            -0.03, 0.03
        )
        bias_delta = (scale[result.node_owners] * result.bias_trace).clamp(-0.03, 0.03)
        if self.plasticity_decay:
            decay = 1 - math.exp(-self.plasticity_decay * frame["duration"])
            enabled = torch.tensor(
                [rate > 0 for rate in rates], dtype=torch.bool, device=self.device
            )
            weight_delta -= torch.where(
                enabled[result.edge_owners],
                decay * (result.weights - result.inherited_weights),
                0,
            )
            bias_delta -= torch.where(
                enabled[result.node_owners],
                decay * (result.biases - result.inherited_biases),
                0,
            )
            weight_delta.clamp_(-0.03, 0.03)
            bias_delta.clamp_(-0.03, 0.03)
        next_weights = (result.weights + weight_delta).clamp(-8, 8)
        next_biases = (result.biases + bias_delta).clamp(-8, 8)
        weight_delta = next_weights - result.weights
        bias_delta = next_biases - result.biases
        result.weights, result.biases = next_weights, next_biases
        changes = torch.zeros(len(rates), dtype=torch.float64, device=self.device)
        changes.index_add_(0, result.edge_owners, weight_delta.abs())
        changes.index_add_(0, result.node_owners, bias_delta.abs())
        for agent, change in zip(frame["agents"], changes.cpu().tolist(), strict=True):
            result.changes[agent["id"]] += change
        result.validate(frame)
        return result

    def snapshot(self):
        payload = super().snapshot()
        payload.update(
            schema_version=2,
            edge_keys=[list(k) for k in self.edge_keys],
            weights=self.weights.cpu().tolist(),
            biases=self.biases.cpu().tolist(),
            edge_trace=self.edge_trace.cpu().tolist(),
            bias_trace=self.bias_trace.cpu().tolist(),
            baseline=[[k, v] for k, v in sorted(self.baseline.items())],
            changes=[[k, v] for k, v in sorted(self.changes.items())],
            updates=[[k, v] for k, v in sorted(self.updates.items())],
            seed=self.seed,
            noise_kind="splitmix_box_muller_v1",
        )
        if self.activity is not None:
            payload["activity"] = self.activity.cpu().tolist()
        if self.temporal_feedback:
            payload["critic"] = [[k, v] for k, v in sorted(self.critic.items())]
        return payload

    @classmethod
    def restore(cls, frame, payload, device="cpu"):
        result = cls(frame, device)
        input_biases = result.biases[result.input_indices].clone()
        expected = set(result.snapshot())
        if (
            not isinstance(payload, dict)
            or set(payload) != expected
            or type(payload["schema_version"]) is not int
            or payload["schema_version"] != 2
            or payload["kind"] != cls.kind
            or type(payload["tick"]) is not int
            or payload["tick"] != frame["tick"]
            or type(payload["seed"]) is not int
            or payload["seed"] != frame["seed"]
            or payload["noise_kind"] != "splitmix_box_muller_v1"
        ):
            raise ValueError("invalid plastic checkpoint header")
        if payload["keys"] != [list(k) for k in result.keys] or payload[
            "edge_keys"
        ] != [list(k) for k in result.edge_keys]:
            raise ValueError("plastic checkpoint identity mismatch")
        if any(
            not isinstance(key, list)
            or len(key) != 2
            or any(type(v) is not int or not 0 < v < 2**64 for v in key)
            for name in ["keys", "edge_keys"]
            for key in payload[name]
        ):
            raise ValueError("invalid plastic gene identity")
        for name in ["memory", "weights", "biases", "edge_trace", "bias_trace"]:
            data = payload[name]
            if (
                not isinstance(data, list)
                or len(data) != getattr(result, name).numel()
                or any(
                    type(v) not in {int, float} or not math.isfinite(v) for v in data
                )
            ):
                raise ValueError("invalid plastic parameter array")
            setattr(
                result,
                name,
                torch.tensor(data, dtype=torch.float64, device=result.device),
            )
        for name in ["baseline", "changes", "updates"]:
            items = payload[name]
            if not isinstance(items, list) or any(
                not isinstance(v, list) or len(v) != 2 or type(v[0]) is not int
                for v in items
            ):
                raise ValueError("invalid plastic population metadata")
            values = dict(items)
            if (
                len(values) != len(items)
                or set(values) != set(result.baseline)
                or any(
                    type(v) not in {float, int}
                    or not math.isfinite(v)
                    or (name != "baseline" and v < 0)
                    or (name == "updates" and type(v) is not int)
                    for v in values.values()
                )
            ):
                raise ValueError("invalid plastic population metadata values")
            setattr(result, name, values)
        if result.activity is not None:
            values = payload["activity"]
            if (
                not isinstance(values, list)
                or len(values) != len(result.keys)
                or any(
                    type(v) not in {int, float}
                    or not math.isfinite(v)
                    or not 0 <= v <= 1
                    for v in values
                )
            ):
                raise ValueError("invalid activity checkpoint")
            result.activity = torch.tensor(
                values, dtype=torch.float64, device=result.device
            )
        if result.temporal_feedback:
            values = payload["critic"]
            if not isinstance(values, list) or any(
                not isinstance(v, list)
                or len(v) != 2
                or type(v[0]) is not int
                or not isinstance(v[1], list)
                or len(v[1]) != 9
                or any(
                    type(x) not in {int, float} or not math.isfinite(x) or abs(x) > 8
                    for x in v[1]
                )
                for v in values
            ):
                raise ValueError("invalid critic checkpoint")
            result.critic = dict(values)
            if len(result.critic) != len(values):
                raise ValueError("duplicate critic identity")
        if not torch.equal(
            result.biases[result.input_indices], input_biases
        ) or torch.any(result.bias_trace[result.input_indices] != 0):
            raise ValueError("input bias/trace was modified")
        result.validate(frame)
        return result
