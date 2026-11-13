import hashlib
import json
from pathlib import Path
import time
import uuid

from .ecosystem import write_json


class RunRecorder:
    def __init__(self, runtime, output, sample_ticks=120, checkpoint_ticks=6000):
        if type(sample_ticks) is not int or sample_ticks < 1:
            raise ValueError("sample_ticks must be a positive integer")
        if type(checkpoint_ticks) is not int or checkpoint_ticks < 1:
            raise ValueError("checkpoint_ticks must be a positive integer")
        self.prefix = Path(output).with_suffix("")
        self.prefix.parent.mkdir(parents=True, exist_ok=True)
        self.sample_ticks = sample_ticks
        self.checkpoint_ticks = checkpoint_ticks
        self.last_sample = runtime.tick - sample_ticks
        self.last_checkpoint = runtime.tick
        self.previous_runtime = runtime
        self.segment = 0
        self.run_id = uuid.uuid4().hex
        self.slot = 0
        self.peaks = {}
        self.captured = set()
        self.last_row = None
        self.start(runtime)
        self.sample(runtime, force=True)

    def path(self, suffix):
        return self.prefix.with_name(self.prefix.name + suffix)

    def start(self, runtime):
        from . import _native
        import torch

        state = runtime.snapshot()
        sources = [Path(_native.__file__), *Path(__file__).parent.rglob("*.py")]
        manifest = {
            "schema_version": 1,
            "run_id": self.run_id,
            "started_at_unix": time.time(),
            "segment": self.segment,
            "start_tick": runtime.tick,
            "rules": state["world"]["rules"],
            "configuration": state["world"]["config"],
            "controller": runtime.brains.kind,
            "device": str(runtime.brains.device),
            "torch_threads": torch.get_num_threads(),
            "source_sha256": {
                p.relative_to(Path(__file__).parent).as_posix(): hashlib.sha256(
                    p.read_bytes()
                ).hexdigest()
                for p in sources
            },
            "sampling_ticks": self.sample_ticks,
            "checkpoint_ticks": self.checkpoint_ticks,
            "checkpoint_slots": 3,
        }
        write_json(self.path(f"-manifest-{self.run_id}-{self.segment}.json"), manifest)
        write_json(self.path("-autosave-0.json"), state)
        self.slot = 1

    def sample(self, runtime, force=False):
        if runtime is not self.previous_runtime:
            self.segment += 1
            self.previous_runtime = runtime
            self.last_sample = runtime.tick - self.sample_ticks
            self.last_checkpoint = runtime.tick
            self.peaks.clear()
            self.captured.clear()
            self.start(runtime)
        if not force and runtime.tick - self.last_sample < self.sample_ticks:
            return self.last_row
        row = runtime.telemetry()
        row["audit"] = runtime.audit()
        row["segment"] = self.segment
        row["run_id"] = self.run_id
        row["recorded_at_unix"] = time.time()
        row["controller"] = {
            "kind": runtime.brains.kind,
            "active_neural_organisms": len(runtime.brains.rows),
            "active_neurons": len(runtime.brains.keys),
            "learning_updates": sum(getattr(runtime.brains, "updates", {}).values()),
            "parameter_change": sum(getattr(runtime.brains, "changes", {}).values()),
        }
        with self.path("-telemetry.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(row, allow_nan=False, separators=(",", ":")) + "\n")
            stream.flush()
        for role in ["producer", "consumer"]:
            count = row["roles"][role]["count"]
            self.peaks[role] = max(count, self.peaks.get(role, 0))
            event = "extinction" if count == 0 and self.peaks[role] else "decline"
            key = (role, event)
            capture = self.peaks[role] > 0 and count < self.peaks[role] * 0.5
            if capture and key not in self.captured:
                runtime.save(
                    self.path(
                        f"-{self.run_id}-segment-{self.segment}-{role}-{event}.json"
                    )
                )
                self.captured.add(key)
        if runtime.tick - self.last_checkpoint >= self.checkpoint_ticks:
            runtime.save(self.path(f"-autosave-{self.slot}.json"))
            self.slot = (self.slot + 1) % 3
            self.last_checkpoint = runtime.tick
        self.last_sample = runtime.tick
        self.last_row = row
        return row
