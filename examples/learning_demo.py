import argparse
import json
from pathlib import Path

from world.ecosystem import write_json
from world.experiments.learning import compare_learning


def main():
    parser = argparse.ArgumentParser(
        description="Compare online plasticity with frozen-weight feeding controls."
    )
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--decisions", type=int, default=500)
    parser.add_argument(
        "--output", type=Path, default=Path("artifacts/learning-controls.json")
    )
    args = parser.parse_args()
    if args.decisions < 1 or any(not 0 <= s < 2**64 for s in args.seeds):
        parser.error("invalid run length or seed")
    root = Path(__file__).resolve().parents[1]
    report = compare_learning(
        json.loads((root / "configs/world_rules_v0.json").read_text()),
        args.seeds,
        args.decisions,
    )
    write_json(args.output, report)
    for row in report["runs"]:
        print(
            f"seed={row['seed']} {row['condition']}: food={row['food_ingested']:.3f}, alive={row['alive']}, parameter_change={row['parameter_change']}"
        )


if __name__ == "__main__":
    main()
