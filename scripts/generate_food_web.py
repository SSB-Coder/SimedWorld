import argparse
import json
from pathlib import Path

from world.experiments.ecosystem import ecosystem_preset


def main():
    parser = argparse.ArgumentParser(description="Regenerate the interactive food-web preset.")
    parser.add_argument("--seed", type=int, default=73)
    parser.add_argument("--scale", type=int, default=1)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    rules = json.loads((root / "configs/circle_rules.json").read_text(encoding="utf-8"))
    rules, config = ecosystem_preset(rules, seed=args.seed, scale=args.scale)
    for name, value in [("rules", rules), ("habitat", config)]:
        (root / f"configs/food_web_{name}.json").write_text(
            json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8"
        )


if __name__ == "__main__":
    main()
