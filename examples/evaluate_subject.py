import argparse
import json
from pathlib import Path
import torch

from world.ecosystem import read_json, write_json
from world.experiments.evaluation import evaluate_subject


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate an archived, data-only neural subject."
    )
    parser.add_argument("subject", type=Path)
    parser.add_argument(
        "--suite",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "configs/evaluation.json",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--threads", type=int, default=1)
    args = parser.parse_args()
    if args.threads < 1:
        parser.error("threads must be positive")
    torch.set_num_threads(args.threads)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    journal = args.output.with_suffix(".trials.jsonl")
    if journal.exists() or args.output.exists():
        parser.error("evaluation output already exists; choose a new path")

    def progress(row):
        with journal.open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(row, allow_nan=False) + "\n")
        print(
            f"Completed {row['task']} seed={row['seed']} control={row['control']}",
            flush=True,
        )

    result = evaluate_subject(
        args.subject, read_json(args.suite), args.device, progress=progress
    )
    write_json(args.output, result)
    print(
        f"Saved {len(result['results'])} embodied trials and memory probe to {args.output}"
    )


if __name__ == "__main__":
    main()
