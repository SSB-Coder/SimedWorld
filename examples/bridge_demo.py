import argparse
from pathlib import Path

from world.runtime import BridgeRuntime


def main():
    parser = argparse.ArgumentParser(description="Run the diagnostic batched Rust/PyTorch fixture.")
    parser.add_argument("--config", type=Path, default=Path(__file__).resolve().parents[1] / "configs/bridge_fixture.json")
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cpu")
    parser.add_argument("--ticks", type=int, default=5)
    parser.add_argument("--checkpoint", type=Path)
    parser.add_argument("--resume", type=Path)
    args = parser.parse_args()
    if args.ticks < 0:
        parser.error("ticks must be nonnegative")
    runtime = (BridgeRuntime.load(args.resume, args.device) if args.resume
               else BridgeRuntime.from_config(args.config, args.device))
    print(f"Diagnostic fixture on {args.device}; no ecology or learning.")
    for _ in range(args.ticks):
        actions = runtime.advance()
        print(f"tick={runtime.world.tick} ids={actions.ids.tolist()} actions={actions.values.tolist()}")
    if args.checkpoint:
        runtime.save(args.checkpoint)
        print(f"Checkpoint: {args.checkpoint}")


if __name__ == "__main__":
    main()
