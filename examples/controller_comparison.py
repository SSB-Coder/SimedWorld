import argparse
import json
from pathlib import Path

from world.brains.candidates import compare_candidates
from world.brains.evolving import NeuralPopulation
from world.ecosystem import EcosystemRuntime, write_json


def main():
    parser = argparse.ArgumentParser(description="Compare the sparse recurrent interface with fixed wired liquid candidates.")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--iterations", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("artifacts/controller-comparison.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    runtime = EcosystemRuntime.create(json.loads((root / "configs/evolution_rules.json").read_text()),
                                       json.loads((root / "configs/evolution_habitat.json").read_text()), args.device)
    frame = runtime.frame()
    report = compare_candidates(frame, args.iterations, args.device)
    baseline = NeuralPopulation(frame, args.device)
    actions, _ = baseline.propose(frame)
    report["baseline"] = {"family": baseline.kind, "nodes": len(baseline.keys), "connections": baseline.sources.numel(),
                          "population_actions": len(actions["agents"]), "scalar_reference": "tests/test_evolution.py"}
    write_json(args.output, report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
