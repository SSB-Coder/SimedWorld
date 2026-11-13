import json
from pathlib import Path

from ._native import Habitat as NativeHabitat


class Habitat:
    def __init__(self, rules: dict, config: dict):
        self._native = NativeHabitat(json.dumps(rules, allow_nan=False),
                                     json.dumps(config, allow_nan=False))

    @property
    def tick(self) -> int:
        return self._native.tick

    def advance(self, ticks: int = 1):
        if type(ticks) is not int or not 0 <= ticks <= 2**64 - 1:
            raise ValueError("ticks must be an unsigned 64-bit integer")
        for _ in range(ticks):
            self._native.advance()

    def audit(self) -> dict:
        return json.loads(self._native.audit_json())

    def summary(self) -> dict:
        return json.loads(self._native.summary_json())

    def snapshot(self) -> dict:
        return json.loads(self._native.snapshot_json())

    def save(self, path: str | Path):
        self._native.save(str(Path(path)))

    @classmethod
    def load(cls, path: str | Path):
        result = cls.__new__(cls)
        result._native = NativeHabitat.load(str(Path(path)))
        return result
