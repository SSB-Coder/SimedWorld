import argparse
import json
from pathlib import Path
import time

import torch

from world.ecosystem import EcosystemRuntime, write_json
from world.experiments.adaptation import compact_preset
from world.presets import founder_genome


def main():
    parser = argparse.ArgumentParser(description="Profile the complete biological controller boundary and native world update.")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--decisions", type=int, default=10)
    parser.add_argument("--hidden", type=int, default=8)
    parser.add_argument("--output", type=Path, default=Path("artifacts/evolution-profile.json"))
    args = parser.parse_args()
    if args.decisions < 1 or not 0 <= args.hidden <= 3000:
        parser.error("invalid decision or hidden-neuron count")
    root = Path(__file__).resolve().parents[1]
    rules, config = compact_preset(json.loads((root / "configs/world_rules_v0.json").read_text()))
    config["evolution"]["mutation"]["max_nodes"] = 4096
    config["evolution"]["mutation"]["max_connections"] = 65536
    config["populations"][0]["genome"] = founder_genome("grazer", hidden=args.hidden)
    runtime = EcosystemRuntime.create(rules, config, args.device)
    totals = dict(observation=0.0, inference_and_action_transfer=0.0, world_fork=0.0, native_update=0.0, state_reconciliation=0.0)
    initial_nodes = len(runtime.brains.keys)
    initial_connections = runtime.brains.sources.numel()
    for _ in range(args.decisions):
        if args.device == "cuda":
            torch.cuda.synchronize()
        start = time.perf_counter(); frame = runtime.frame(); observation = time.perf_counter()
        actions, memory = runtime.brains.propose(frame)
        if args.device == "cuda":
            torch.cuda.synchronize()
        inference = time.perf_counter(); candidate = runtime.world.fork(); copied = time.perf_counter()
        candidate.advance_controlled(json.dumps(actions, allow_nan=False)); advanced = time.perf_counter()
        next_frame = json.loads(candidate.agent_frame_json())
        brains = runtime.brains.reconcile(next_frame, memory)
        if args.device == "cuda":
            torch.cuda.synchronize()
        end = time.perf_counter()
        for key, duration in zip(totals, [observation-start, inference-observation, copied-inference, advanced-copied, end-advanced], strict=True):
            totals[key] += duration
        runtime.world, runtime.brains = candidate, brains
    report = {"schema_version": 1, "device": args.device, "precision": "float64 reference", "decisions": args.decisions,
              "initial_nodes": initial_nodes, "initial_connections": initial_connections,
              "component_wall_seconds": totals, "total_component_seconds": sum(totals.values()),
              "final_living": runtime.summary()["living_count"], "audit": runtime.audit(),
              "cuda_allocated_bytes": torch.cuda.memory_allocated() if args.device == "cuda" else 0,
              "scope": "Small declared scenario; includes cohort JSON, sparse execution/transfer, cloning, native biology/geometry, and reconciliation. Not a large-population capacity claim."}
    write_json(args.output, report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
