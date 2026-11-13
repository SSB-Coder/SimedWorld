import json
import os
from pathlib import Path
import tempfile
import time

from ._native import Habitat as NativeHabitat, core_version
from .brains.evolving import NeuralPopulation
from .brains.plastic import PlasticPopulation
from .brains.liquid import (
    LiquidPopulation,
    TonicLiquidPopulation,
    LegacyLiquidPopulation,
    LIQUID_FAMILIES,
)
from .brains.packing import freeze_frame


def write_json(path, payload):
    text = json.dumps(payload, allow_nan=False, separators=(",", ":"))
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            delete=False,
        ) as stream:
            temporary = Path(stream.name)
            stream.write(text)
            stream.flush()
            os.fsync(stream.fileno())
        for attempt in range(40):
            try:
                os.replace(temporary, path)
                break
            except PermissionError:
                if os.name != "nt" or attempt == 39:
                    raise
                time.sleep(0.01)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def read_json(path):
    for attempt in range(40):
        try:
            return json.loads(Path(path).read_text(encoding="utf-8"))
        except PermissionError:
            if os.name != "nt" or attempt == 39:
                raise
            time.sleep(0.01)


class EcosystemRuntime:
    def __init__(self, world, brains, frame=None):
        self.world = world
        self.brains = brains
        self.archive = None
        self._frame = freeze_frame(self.frame() if frame is None else frame)
        self.brains.validate(self._frame)
        self.brains.signature = self.brains._signature(self._frame)

    @classmethod
    def create(cls, rules, config, device="cpu", controller="sparse"):
        if controller not in {"sparse", "ltc", "ltc_v1", "ltc_v2"}:
            raise ValueError("unsupported ecological controller")
        world = NativeHabitat(
            json.dumps(rules, allow_nan=False), json.dumps(config, allow_nan=False)
        )
        frame = freeze_frame(json.loads(world.agent_frame_json()))
        if controller != "sparse" and not frame.get("online_learning"):
            raise ValueError(
                "the liquid ecological backend requires online_learning enabled"
            )
        family = {
            "sparse": PlasticPopulation,
            "ltc": LiquidPopulation,
            "ltc_v1": LegacyLiquidPopulation,
            "ltc_v2": TonicLiquidPopulation,
        }[controller] if frame.get("online_learning") else NeuralPopulation
        return cls(world, family(frame, device), frame)

    @property
    def tick(self):
        return self.world.tick

    @property
    def decision_duration(self):
        return self.world.decision_duration

    @property
    def elapsed_time(self):
        return self.world.elapsed_time

    @property
    def next_event(self):
        return self.world.next_event

    def frame(self):
        return json.loads(self.world.agent_frame_json())

    def advance(self, decisions=1):
        if type(decisions) is not int or decisions < 0:
            raise ValueError("decisions must be a nonnegative integer")
        for _ in range(decisions):
            frame = self._frame
            if frame["tick"] != self.tick:
                raise ValueError("world was advanced outside the coordinator")
            packed = isinstance(self.brains, PlasticPopulation) and hasattr(
                self.world, "propose_controlled_packed"
            )
            if packed:
                actions, proposed = self.brains.propose(frame, packed=True)
                candidate = self.world.propose_controlled_packed(
                    json.dumps(actions, allow_nan=False),
                    self.brains.noise_ids, self.brains.biases.detach().cpu().numpy(),
                    self.brains.parameter_edge_ids, self.brains.weights.detach().cpu().numpy(),
                )
            else:
                actions, proposed = self.brains.propose(frame)
                candidate = self.world.propose_controlled(json.dumps(actions, allow_nan=False))
            known = {a["id"]: a for a in frame["agents"]}
            next_frame = json.loads(
                candidate.agent_frame_delta_json(
                    json.dumps({str(k): v["version"] for k, v in known.items()})
                )
            )
            for agent in next_frame["agents"]:
                if "brain" not in agent:
                    if (
                        agent["id"] not in known
                        or known[agent["id"]]["version"] != agent["version"]
                    ):
                        raise ValueError("native graph delta lacks a matching version")
                    agent["brain"] = known[agent["id"]]["brain"]
            freeze_frame(next_frame)
            brains = self.brains.reconcile(next_frame, proposed)
            if self.archive is not None:
                survivors = {a["id"] for a in next_frame["agents"]}
                memory = getattr(proposed, "memory", proposed)
                for identity in known.keys() - survivors:
                    self.archive.capture(
                        self, identity, "death", memory=memory, world=candidate
                    )
            self.world = candidate
            self.brains = brains
            self._frame = next_frame

    def summary(self, *, compact=False):
        getter = (
            self.world.summary_compact_json
            if compact and hasattr(self.world, "summary_compact_json")
            else self.world.summary_json
        )
        result = json.loads(getter())
        if compact and result.get("diagnostics") is not None:
            for key in ["events", "behavior", "energy_history", "death_history"]:
                result["diagnostics"].pop(key, None)
        return result

    def telemetry(self):
        return json.loads(self.world.telemetry_json())

    def events_since(self, sequence):
        return json.loads(self.world.events_since_json(sequence))

    def audit(self):
        return json.loads(self.world.audit_json())

    def display(self, selected=None):
        payload = json.loads(self.world.display_json(selected))
        if selected is not None and isinstance(self.brains, PlasticPopulation):
            agent = next(
                (a for a in self._frame["agents"] if a["id"] == selected), None
            )
            if agent:
                nodes = []
                for kind in ["input", "hidden", "output"]:
                    nodes.extend(
                        [n for n in agent["brain"]["nodes"] if n["kind"] == kind][:48]
                    )
                ids = {n["id"] for n in nodes}
                node_values = (
                    self.brains.memory[
                        [self.brains.index[(selected, n["id"])] for n in nodes]
                    ]
                    .cpu()
                    .tolist()
                )
                edges = [
                    e
                    for e in agent["brain"]["connections"]
                    if e["source"] in ids and e["target"] in ids
                ][:512]
                weights = (
                    self.brains.weights[
                        [self.brains.edge_index[(selected, e["id"])] for e in edges]
                    ]
                    .cpu()
                    .tolist()
                )
                payload["brain"] = {
                    "id": selected,
                    "nodes": [
                        {
                            "id": n["id"],
                            "kind": n["kind"],
                            "value": v,
                            "channel": n["port"] % 256
                            if n["port"] is not None
                            else None,
                        }
                        for n, v in zip(nodes, node_values, strict=True)
                    ],
                    "edges": [
                        {"source": e["source"], "target": e["target"], "weight": v}
                        for e, v in zip(edges, weights, strict=True)
                    ],
                    "updates": self.brains.updates[selected],
                    "parameter_change": self.brains.changes[selected],
                }
        return payload

    def snapshot(self):
        world = json.loads(self.world.checkpoint_json())
        agents = [
            {
                "id": o["id"],
                "version": (o.get("expressed_brain") or o["genome"]["brain"])[
                    "version"
                ],
                "brain": o.get("expressed_brain") or o["genome"]["brain"],
            }
            for o in world["ledger"]["organisms"]
            if o["alive"] and o["genome"].get("brain")
        ]
        self.brains.validate({"schema_version": 1, "tick": self.tick, "agents": agents})
        return {
            "schema_version": 2 if isinstance(self.brains, PlasticPopulation) else 1,
            "kind": "world.ecosystem",
            "engine_version": core_version(),
            "world": world,
            "neural": self.brains.snapshot(),
        }

    def save(self, path):
        write_json(path, self.snapshot())

    @classmethod
    def from_snapshot(cls, payload, device="cpu"):
        if not isinstance(payload, dict) or set(payload) != {
            "schema_version",
            "kind",
            "engine_version",
            "world",
            "neural",
        }:
            raise ValueError("unexpected ecosystem checkpoint fields")
        if (
            type(payload["schema_version"]) is not int
            or payload["schema_version"] not in {1, 2}
            or payload["kind"] != "world.ecosystem"
            or payload["engine_version"] != core_version()
        ):
            raise ValueError("unsupported ecosystem checkpoint")
        world = NativeHabitat.from_checkpoint_json(
            json.dumps(payload["world"], allow_nan=False)
        )
        frame = freeze_frame(json.loads(world.agent_frame_json()))
        if (payload["schema_version"] == 2) != frame.get("online_learning", False):
            raise ValueError("checkpoint controller family does not match world")
        brains = (
            LIQUID_FAMILIES.get(payload["neural"].get("kind"), PlasticPopulation)
            if payload["schema_version"] == 2
            else NeuralPopulation
        ).restore(frame, payload["neural"], device)
        return cls(world, brains, frame)

    @classmethod
    def load(cls, path, device="cpu"):
        return cls.from_snapshot(read_json(path), device)
