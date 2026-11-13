import argparse
import json
import torch
from pathlib import Path

from world.ecosystem import write_json
from world.experiments.liquid import compare_online_liquids


def main():
    parser = argparse.ArgumentParser(
        description="Compare online ncps LTC and CfC in controlled physical feeding."
    )
    parser.add_argument("--decisions", type=int, default=200)
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(args.output)
    torch.set_num_threads(1)
    rules = json.loads(
        (Path(__file__).resolve().parents[1] / "configs/circle_rules.json").read_text()
    )
    write_json(args.output, compare_online_liquids(rules, args.seeds, args.decisions))


if __name__ == "__main__":
    main()
