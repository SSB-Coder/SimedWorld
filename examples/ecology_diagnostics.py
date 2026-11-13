import argparse
import json
from pathlib import Path

from world.ecosystem import write_json
from world.experiments.diagnostics import diagnose_legacy


def main():
    parser = argparse.ArgumentParser(description="Inspect consumer extinction in the original diagnostic ecosystem.")
    parser.add_argument("--ticks", type=int, default=12000)
    parser.add_argument("--sample-every", type=int, default=300)
    parser.add_argument("--output", type=Path, default=Path("artifacts/consumer-diagnostics.json"))
    args = parser.parse_args()
    if args.ticks < 0 or args.sample_every < 1:
        parser.error("invalid tick/sample interval")
    root = Path(__file__).resolve().parents[1]
    report = diagnose_legacy(json.loads((root / "configs/ecosystem_rules.json").read_text()),
                             json.loads((root / "configs/ecosystem_habitat.json").read_text()), args.ticks, args.sample_every)
    write_json(args.output, report)
    print(json.dumps({"samples": report["samples"], "energy_flows": report["energy_flows"], "audit": report["audit"]}, indent=2))


if __name__ == "__main__":
    main()
