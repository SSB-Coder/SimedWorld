import argparse
import json
from pathlib import Path

from world.ecosystem import write_json
from world.experiments.adaptation import run_controls


def main():
    parser = argparse.ArgumentParser(description="Run mutation-disabled, brain-only, body-only, and combined pilot controls.")
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--decisions", type=int, default=120)
    parser.add_argument("--evaluation-decisions", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("artifacts/adaptation-controls.json"))
    args = parser.parse_args()
    if args.decisions < 0 or args.evaluation_decisions < 0 or any(not 0 <= s < 2**64 - 200003 for s in args.seeds):
        parser.error("invalid decision count or seed")
    root = Path(__file__).resolve().parents[1]
    rules = json.loads((root / "configs/world_rules_v0.json").read_text())
    report = run_controls(rules, args.seeds, args.decisions, args.evaluation_decisions)
    write_json(args.output, report)
    print(f"Saved {len(report['runs'])} controlled runs to {args.output}")


if __name__ == "__main__":
    main()
