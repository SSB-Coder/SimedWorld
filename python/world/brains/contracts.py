from dataclasses import dataclass
from typing import ClassVar

import numpy as np


SCHEMA_VERSION = 1
UINT64_MAX = 2**64 - 1


def uint64(value: int, name: str) -> int:
    if type(value) is not int or not 0 <= value <= UINT64_MAX:
        raise ValueError(f"{name} must be an unsigned 64-bit integer")
    return value


@dataclass(frozen=True, slots=True)
class Batch:
    tick: int
    ids: np.ndarray
    topology_versions: np.ndarray
    values: np.ndarray
    schema_version: int = SCHEMA_VERSION
    width: ClassVar[int] = 0

    def __post_init__(self):
        uint64(self.tick, "tick")
        if type(self.schema_version) is not int or self.schema_version != SCHEMA_VERSION:
            raise ValueError("unsupported batch schema")
        for name, dtype, rank in [("ids", np.uint64, 1),
                                  ("topology_versions", np.uint64, 1),
                                  ("values", np.float32, 2)]:
            array = getattr(self, name)
            if not isinstance(array, np.ndarray) or array.dtype != np.dtype(dtype):
                raise TypeError(f"{name} must be a {np.dtype(dtype)} NumPy array")
            if array.ndim != rank or not array.flags.c_contiguous:
                raise ValueError(f"{name} has invalid rank or is not C-contiguous")
            owned = array.copy()
            owned.setflags(write=False)
            object.__setattr__(self, name, owned)
        count = len(self.ids)
        if self.topology_versions.shape != (count,) or self.values.shape != (count, self.width):
            raise ValueError("batch shapes do not agree")
        if len(np.unique(self.ids)) != count:
            raise ValueError("duplicate organism ID")
        if not np.isfinite(self.values).all():
            raise ValueError("batch values must be finite")
        self._validate_values()

    def _validate_values(self):
        raise NotImplementedError

    @classmethod
    def from_native(cls, payload: dict):
        if set(payload) != {"schema_version", "tick", "ids", "topology_versions", "values"}:
            raise ValueError("unexpected batch fields")
        return cls(**payload)

    def to_native(self) -> dict:
        return {"schema_version": self.schema_version, "tick": self.tick, "ids": self.ids,
                "topology_versions": self.topology_versions, "values": self.values}

    def reordered(self, order):
        order = np.asarray(order)
        if len(self.ids) == 0 and order.shape == (0,):
            order = np.empty(0, dtype=np.intp)
        if order.ndim != 1 or order.dtype.kind not in "iu" or not np.array_equal(
                np.sort(order), np.arange(len(self.ids))):
            raise ValueError("row order must be a complete permutation")
        order = order.astype(np.intp)
        return type(self)(self.tick, self.ids[order], self.topology_versions[order],
                          self.values[order], self.schema_version)


class ObservationBatch(Batch):
    __slots__ = ()
    width = 3

    def _validate_values(self):
        if np.any(self.values < 0) or np.any(self.values > 1):
            raise ValueError("fixture observations must be in [0, 1]")


class ActionBatch(Batch):
    __slots__ = ()
    width = 2

    def _validate_values(self):
        if np.any(np.abs(self.values[:, 0]) > 1) or np.any(
                (self.values[:, 1] < 0) | (self.values[:, 1] > 1)):
            raise ValueError("actions require turn in [-1, 1] and thrust in [0, 1]")
