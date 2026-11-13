import json
import os
from pathlib import Path
import tempfile

import numpy as np

from ._native import FixtureWorld
from .brains.contracts import ObservationBatch
from .brains.registry import ControllerRegistry
from .config import load_fixture


class BridgeRuntime:
    def __init__(self, world: FixtureWorld, registry: ControllerRegistry):
        self.world = world
        self.registry = registry
        self.snapshot()

    @classmethod
    def from_config(cls, path: str | Path, device: str = "cpu"):
        world = load_fixture(path)
        observations = ObservationBatch.from_native(world.observe())
        return cls(world, ControllerRegistry.initialize(observations, device))

    def advance(self):
        observations = ObservationBatch.from_native(self.world.observe())
        actions, candidate = self.registry.propose(observations)
        self.world.apply_actions(**actions.to_native())
        self.registry = candidate
        return actions

    def snapshot(self) -> dict:
        observations = ObservationBatch.from_native(self.world.observe())
        self.registry.validate_alignment(observations)
        world = json.loads(self.world.snapshot_json())
        controller = self.registry.snapshot()
        previous = {item["organism"]["id"]: item["last_action"] for item in world["organisms"]}
        expected = np.asarray([previous[identity] for identity in controller["ids"]],
                              dtype=np.float32).reshape(-1, 2)
        actual = np.asarray(controller["memory"], dtype=np.float32).reshape(-1, 2)
        if not np.array_equal(expected, actual):
            raise ValueError("world actions and controller memory are inconsistent")
        return {"schema_version": 1, "world": world, "controller": controller}

    def save(self, path: str | Path):
        text = json.dumps(self.snapshot(), indent=2, allow_nan=False) + "\n"
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = None
        try:
            with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                             prefix=f".{path.name}.", delete=False) as stream:
                temporary = Path(stream.name)
                stream.write(text)
            os.replace(temporary, path)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)

    @classmethod
    def load(cls, path: str | Path, device: str = "cpu"):
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
        if not isinstance(payload, dict) or set(payload) != {"schema_version", "world", "controller"}:
            raise ValueError("unexpected fixture checkpoint fields")
        if type(payload["schema_version"]) is not int or payload["schema_version"] != 1:
            raise ValueError("unsupported fixture checkpoint schema")
        world = FixtureWorld.from_snapshot_json(json.dumps(payload["world"], allow_nan=False))
        registry = ControllerRegistry.from_snapshot(payload["controller"], device)
        return cls(world, registry)
