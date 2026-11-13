"""Immutable native graph records shared between coordinated decisions."""

import numpy as np


def compiled_graph(brain):
    cached = getattr(brain, "_compiled", None)
    if cached is not None:
        return cached
    indices = {node["id"]: i for i, node in enumerate(brain["nodes"])}
    data = {
        "sources": np.array([indices[e["source"]] for e in brain["connections"]], dtype=np.int64),
        "targets": np.array([indices[e["target"]] for e in brain["connections"]], dtype=np.int64),
        "weights": np.array([e["weight"] for e in brain["connections"]], dtype=np.float64),
        "biases": np.array([n["bias"] for n in brain["nodes"]], dtype=np.float64),
        "leaks": np.array([n["leak"] for n in brain["nodes"]], dtype=np.float64),
        "inputs": np.array([i for i, n in enumerate(brain["nodes"]) if n["kind"] == "input"], dtype=np.int64),
    }
    for values in data.values():
        values.setflags(write=False)
    if isinstance(brain, FrozenGraph):
        brain._compiled = data
    return data


def named_edges(brain, identity):
    if not isinstance(brain, FrozenGraph):
        return tuple((identity, e["id"]) for e in brain["connections"])
    if not hasattr(brain, "_named_edges"):
        brain._named_edges = {}
    if identity not in brain._named_edges:
        brain._named_edges[identity] = tuple((identity, e["id"]) for e in brain["connections"])
    return brain._named_edges[identity]


class FrozenRecord(dict):
    def _immutable(self, *args, **kwargs):
        raise TypeError("native graph records are immutable")

    __setitem__ = __delitem__ = clear = pop = popitem = setdefault = update = (
        __ior__
    ) = _immutable


def graph_signature(brain):
    if isinstance(brain, FrozenGraph):
        return brain.signature
    return (
        tuple(
            (n["id"], n["kind"], n["port"], n["bias"], n["leak"])
            for n in brain["nodes"]
        ),
        tuple(
            (e["id"], e["source"], e["target"], e["weight"])
            for e in brain["connections"]
        ),
    )


class FrozenGraph(FrozenRecord):
    def __init__(self, brain):
        dict.__init__(self, brain)
        dict.__setitem__(self, "nodes", tuple(FrozenRecord(n) for n in brain["nodes"]))
        dict.__setitem__(
            self, "connections", tuple(FrozenRecord(e) for e in brain["connections"])
        )
        self.signature = graph_signature(brain)


def freeze_frame(frame):
    for agent in frame["agents"]:
        if not isinstance(agent["brain"], FrozenGraph):
            agent["brain"] = FrozenGraph(agent["brain"])
    return frame
