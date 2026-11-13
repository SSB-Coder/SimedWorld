import numpy as np
import torch

from .contracts import ActionBatch, ObservationBatch, uint64


class ControllerRegistry:
    def __init__(self, observations: ObservationBatch, memory: torch.Tensor):
        self.tick = observations.tick
        self._ids = observations.ids
        self._versions = observations.topology_versions
        self._rows = {int(identity): row for row, identity in enumerate(self._ids)}
        self._memory = memory

    @classmethod
    def initialize(cls, observations: ObservationBatch, device: str = "cpu"):
        device = torch.device(device)
        if device.type not in {"cpu", "cuda"}:
            raise ValueError("supported controller devices are cpu and cuda")
        if device.type == "cuda" and not torch.cuda.is_available():
            raise RuntimeError("CUDA was requested but is unavailable")
        return cls(observations, torch.zeros((len(observations.ids), 2), dtype=torch.float32,
                                             device=device))

    def validate_alignment(self, observations: ObservationBatch):
        if observations.tick != self.tick or set(map(int, observations.ids)) != set(self._rows):
            raise ValueError("controller cohort or tick does not match the world")
        for identity, version in zip(observations.ids, observations.topology_versions, strict=True):
            if int(version) != int(self._versions[self._rows[int(identity)]]):
                raise ValueError("controller topology version does not match the world")

    @torch.inference_mode()
    def propose(self, observations: ObservationBatch):
        self.validate_alignment(observations)
        next_tick = uint64(self.tick + 1, "next tick")
        device = self._memory.device
        rows = torch.tensor([self._rows[int(identity)] for identity in observations.ids],
                            dtype=torch.long, device=device)
        previous = self._memory.index_select(0, rows)
        signals = torch.tensor(observations.values, dtype=torch.float32, device=device)
        turn = (signals[:, 1] - signals[:, 0] + 0.25 * previous[:, 0]).clamp(-1, 1)
        thrust = (signals[:, 2] + 0.25 * previous[:, 1]).clamp(0, 1)
        next_values = torch.stack((turn, thrust), dim=1)
        actions = ActionBatch(observations.tick, observations.ids, observations.topology_versions,
                              next_values.cpu().numpy().copy())
        memory = self._memory.clone()
        memory.index_copy_(0, rows, next_values)
        canonical = ObservationBatch(next_tick, self._ids, self._versions,
                                     np.zeros((len(self._ids), 3), dtype=np.float32))
        return actions, type(self)(canonical, memory)

    def snapshot(self) -> dict:
        return {"schema_version": 1, "kind": "diagnostic_recurrent", "tick": self.tick,
                "ids": self._ids.tolist(), "topology_versions": self._versions.tolist(),
                "memory": self._memory.cpu().numpy().tolist()}

    @classmethod
    def from_snapshot(cls, payload: dict, device: str = "cpu"):
        if set(payload) != {"schema_version", "kind", "tick", "ids", "topology_versions", "memory"}:
            raise ValueError("unexpected controller snapshot fields")
        if type(payload["schema_version"]) is not int or payload["schema_version"] != 1 or payload["kind"] != "diagnostic_recurrent":
            raise ValueError("unsupported controller snapshot")
        for field in ("ids", "topology_versions"):
            if not isinstance(payload[field], list):
                raise ValueError(f"{field} must be a list")
            for value in payload[field]:
                uint64(value, field)
        count = len(payload["ids"])
        observations = ObservationBatch(payload["tick"], np.array(payload["ids"], dtype=np.uint64),
                                        np.array(payload["topology_versions"], dtype=np.uint64),
                                        np.zeros((count, 3), dtype=np.float32))
        memory = np.asarray(payload["memory"])
        if count == 0 and memory.shape == (0,):
            memory = memory.reshape(0, 2)
        if memory.dtype.kind not in "fi" or memory.shape != (count, 2):
            raise ValueError("invalid controller memory")
        if not np.isfinite(memory).all() or np.any(np.abs(memory[:, 0]) > 1) or np.any(
                (memory[:, 1] < 0) | (memory[:, 1] > 1)):
            raise ValueError("controller memory is outside its permitted bounds")
        values = ActionBatch(observations.tick, observations.ids, observations.topology_versions,
                             memory.astype(np.float32))
        result = cls.initialize(observations, device)
        result._memory = torch.tensor(values.values, dtype=torch.float32, device=result._memory.device)
        return result
