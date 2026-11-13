import argparse
import json
from pathlib import Path
import time

import torch

from world.ecosystem import EcosystemRuntime, write_json
from world.experiments.ecosystem import ecosystem_preset
from world.presets import neural_blueprint


def benchmark(
    rules,
    seed=73,
    scale=1,
    decisions=100,
    device="cpu",
    resume=None,
    hidden=None,
    controller="sparse",
):
    torch.set_num_threads(1)
    if resume:
        if hidden is not None:
            raise ValueError("resumed worlds retain their stored neural graphs")
        runtime = EcosystemRuntime.load(resume, device)
    else:
        rules, config = ecosystem_preset(rules, seed, scale)
        if hidden is not None:
            for ordinal, group in enumerate(config["populations"]):
                body = group["genome"]["body"]
                kind = "pursuit" if body["diet"] == "predator" else body["diet"]
                group["genome"]["brain"] = neural_blueprint(
                    body, hidden, seed + ordinal, kind
                )
        runtime = EcosystemRuntime.create(
            rules, config, device=device, controller=controller
        )
    runtime.advance(2)
    tick = runtime.tick
    if device == "cuda":
        torch.cuda.synchronize()
        torch.cuda.reset_peak_memory_stats()
    started = time.perf_counter()
    runtime.advance(decisions)
    if device == "cuda":
        torch.cuda.synchronize()
    elapsed = time.perf_counter() - started
    tensors = [
        getattr(runtime.brains, name, None)
        for name in [
            "weights",
            "biases",
            "memory",
            "sources",
            "targets",
            "edge_trace",
            "bias_trace",
            "leaks",
            "input_indices",
            "node_owners",
            "edge_owners",
            "activity",
        ]
    ]
    storage = sum(t.numel() * t.element_size() for t in tensors if t is not None)
    return {
        "schema_version": 1,
        "seed": seed,
        "scale": scale,
        "hidden_neurons_per_founder": hidden,
        "device": device,
        "controller": runtime.brains.kind,
        "value_parameters": sum(
            len(v) for v in getattr(runtime.brains, "critic", {}).values()
        ),
        "decisions": decisions,
        "start_tick": tick,
        "end_tick": runtime.tick,
        "wall_seconds": elapsed,
        "ticks_per_second": (runtime.tick - tick) / elapsed,
        "actual_speed": decisions * runtime.decision_duration / elapsed,
        "active_organisms": len(runtime.display()["organisms"]),
        "neural_organisms": len(runtime.brains.rows),
        "neurons": len(runtime.brains.keys),
        "connections": len(runtime.brains.weights),
        "packed_tensor_bytes": storage,
        "gpu_peak_allocated_bytes": torch.cuda.max_memory_allocated()
        if device == "cuda"
        else None,
        "audit": runtime.audit(),
        "limits": runtime.telemetry()["limits"],
        "scope": "synchronized headless throughput; tensor storage excludes Python/native metadata and allocator overhead",
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--rules",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "configs/circle_rules.json",
    )
    parser.add_argument("--resume", type=Path)
    parser.add_argument("--scale", type=int, default=1)
    parser.add_argument("--hidden", type=int)
    parser.add_argument("--decisions", type=int, default=100)
    parser.add_argument("--controller", choices=["sparse", "ltc"], default="sparse")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.decisions < 1:
        parser.error("decisions must be positive")
    result = benchmark(
        json.loads(args.rules.read_text()),
        scale=args.scale,
        decisions=args.decisions,
        device=args.device,
        resume=args.resume,
        hidden=args.hidden,
        controller=args.controller,
    )
    write_json(args.output, result)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
