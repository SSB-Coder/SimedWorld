import argparse
import json
from pathlib import Path

from world.ecosystem import write_json
from world.experiments.batch import run_batch


def main():
    parser = argparse.ArgumentParser(
        description="Run independent, checkpointed ecosystem trials."
    )
    parser.add_argument(
        "--rules",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "configs/circle_rules.json",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--controls", nargs="+", default=["full"])
    parser.add_argument("--decisions", type=int, default=10000)
    parser.add_argument("--scale", type=int, default=1)
    parser.add_argument(
        "--scenario",
        choices=["ecosystem", "hunting", "herding", "defense"],
        default="ecosystem",
    )
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--controller", choices=["sparse", "ltc"], default="sparse")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    args = parser.parse_args()
    rules = json.loads(args.rules.read_text())
    requests = [
        {
            "rules": rules,
            "seed": seed,
            "control": control,
            "decisions": args.decisions,
            "scale": args.scale,
            "scenario": args.scenario,
            "device": args.device,
            "controller": args.controller,
            "output": str(args.output / f"{control}-{seed}"),
        }
        for control in args.controls
        for seed in args.seeds
    ]
    results = run_batch(requests, args.workers)
    write_json(args.output / "study.json", results)
    print(json.dumps(results))
    if any(r["status"] != "completed" for r in results):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
