"""Data-only subjects containing the parameters actually expressed in life."""

import hashlib
import json
from pathlib import Path
import uuid

from .ecosystem import read_json, write_json


def digest(payload):
    return hashlib.sha256(
        json.dumps(
            payload, sort_keys=True, separators=(",", ":"), allow_nan=False
        ).encode()
    ).hexdigest()


def subject_package(runtime, identity, reason="inspection", memory=None, world=None):
    metadata = json.loads((world or runtime.world).subject_json(identity))
    agent = next((a for a in runtime._frame["agents"] if a["id"] == identity), None)
    if agent is None:
        raise ValueError("subject has no live controller at this boundary")
    nodes = agent["brain"]["nodes"]
    edges = agent["brain"]["connections"]
    brains = runtime.brains
    node_indices = [brains.index[(identity, n["id"])] for n in nodes]
    edge_indices = [
        i
        for i, key in enumerate(getattr(brains, "edge_keys", []))
        if key[0] == identity
    ]
    weights = (
        brains.weights[edge_indices].cpu().tolist()
        if edge_indices
        else [e["weight"] for e in edges]
    )
    biases = brains.biases[node_indices].cpu().tolist()
    graph = json.loads(json.dumps(agent["brain"]))
    for edge, value in zip(graph["connections"], weights, strict=True):
        edge["weight"] = value
    for node, value in zip(graph["nodes"], biases, strict=True):
        node["bias"] = value
    state = {
        "memory": (brains.memory if memory is None else memory)[node_indices]
        .cpu()
        .tolist(),
        "updates": getattr(brains, "updates", {}).get(identity, 0),
        "parameter_change": getattr(brains, "changes", {}).get(identity, 0.0),
    }
    if hasattr(brains, "edge_trace"):
        state.update(
            edge_trace=brains.edge_trace[edge_indices].cpu().tolist(),
            bias_trace=brains.bias_trace[node_indices].cpu().tolist(),
            baseline=brains.baseline[identity],
        )
    payload = {
        "schema_version": 1,
        "kind": "world.subject",
        "reason": reason,
        "origin": metadata,
        "expressed_graph": graph,
        "controller_kind": brains.kind,
        "state": state,
        "boundary": "parameters before transition learning; memory after selected action"
        if memory is not None
        else "committed decision boundary",
    }
    return {**payload, "sha256": digest(payload)}


def read_subject(path):
    payload = read_json(path)
    content = {k: v for k, v in payload.items() if k != "sha256"}
    if (
        payload.get("kind") != "world.subject"
        or payload.get("schema_version") != 1
        or payload.get("sha256") != digest(content)
    ):
        raise ValueError("invalid subject package or checksum")
    return payload


class SubjectArchive:
    def __init__(self, directory, limit=256):
        if type(limit) is not int or limit < 1:
            raise ValueError("archive limit must be positive")
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True)
        self.run_id = uuid.uuid4().hex
        self.limit = limit
        self.paths = []

    def capture(self, runtime, identity, reason="inspection", memory=None, world=None):
        payload = subject_package(runtime, identity, reason, memory, world)
        filename = f"{self.run_id}-{identity}-{payload['origin']['tick']}-{reason}.json"
        path = self.directory / filename
        write_json(path, payload)
        with (self.directory / f"{self.run_id}-index.jsonl").open(
            "a", encoding="utf-8"
        ) as stream:
            stream.write(
                json.dumps(
                    {
                        "file": filename,
                        "sha256": payload["sha256"],
                        "id": identity,
                        "tick": payload["origin"]["tick"],
                        "reason": reason,
                    }
                )
                + "\n"
            )
        self.paths.append(path)
        if len(self.paths) > self.limit:
            self.paths.pop(0).unlink(missing_ok=True)
        return path
