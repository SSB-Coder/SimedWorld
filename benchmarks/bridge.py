import argparse
import json
import time

from world import FixtureWorld
from world.brains.contracts import ObservationBatch
from world.brains.registry import ControllerRegistry
from world.runtime import BridgeRuntime


def main():
    parser = argparse.ArgumentParser(description="Measure completed diagnostic batch exchanges.")
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cpu")
    parser.add_argument("--population", type=int, default=128)
    parser.add_argument("--ticks", type=int, default=200)
    args = parser.parse_args()
    if args.population < 1 or args.ticks < 1:
        parser.error("population and ticks must be positive")
    config = {"schema_version": 1, "organisms": [
        {"id": identity, "topology_version": identity % 3,
         "signals": [(identity % 4) / 4, ((identity + 1) % 4) / 4, 0.5]}
        for identity in range(args.population)]}
    world = FixtureWorld(json.dumps(config))
    registry = ControllerRegistry.initialize(ObservationBatch.from_native(world.observe()), args.device)
    runtime = BridgeRuntime(world, registry)
    for _ in range(10):
        runtime.advance()
    started = time.perf_counter()
    for _ in range(args.ticks):
        runtime.advance()
    elapsed = time.perf_counter() - started
    print(json.dumps({"device": args.device, "population": args.population,
                      "measured_ticks": args.ticks, "elapsed_seconds": elapsed,
                      "completed_ticks_per_second": args.ticks / elapsed,
                      "scope": "Diagnostic fixture with copies and synchronization; not ecosystem throughput."},
                     indent=2))


if __name__ == "__main__":
    main()
