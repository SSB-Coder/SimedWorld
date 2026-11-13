import argparse
import json
from pathlib import Path

from world import Habitat


ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description="Run a headless diagnostic habitat.")
    parser.add_argument("--rules", type=Path, default=ROOT / "configs/world_rules_v0.json")
    parser.add_argument("--config", type=Path, default=ROOT / "configs/producer_habitat.json")
    parser.add_argument("--ticks", type=int, default=3600)
    parser.add_argument("--sample-every", type=int, default=600)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--checkpoint", type=Path)
    parser.add_argument("--resume", type=Path)
    args = parser.parse_args()
    if args.ticks < 0 or args.sample_every < 1:
        parser.error("ticks must be nonnegative and sample-every must be positive")
    if args.resume:
        habitat = Habitat.load(args.resume)
    else:
        rules = json.loads(args.rules.read_text(encoding="utf-8"))
        config = json.loads(args.config.read_text(encoding="utf-8"))
        habitat = Habitat(rules, config)
    stop_at = habitat.tick + args.ticks
    samples = [habitat.summary()]
    print("Diagnostic ecology; fixed behavior and clone inheritance.")
    while habitat.tick < stop_at:
        habitat.advance(min(args.sample_every, stop_at - habitat.tick))
        row = habitat.summary()
        samples.append(row)
        print(f"tick={row['tick']:6d} time={row['elapsed_time']:7.2f} alive={row['living_count']:4d} "
              f"N={row['total_nutrient']:.6f} living_N={row['living_nutrient']:.6f} "
              f"captured_E={row['absorbed_light']:.6f} "
              f"births={row['ecology']['births']} deaths={row['ecology']['deaths']} "
              f"feeds={row['ecology']['feeding_events']}")
    audit = habitat.audit()
    print(f"Audits passed: nutrient_error={audit['nutrient_error']:.3g} "
          f"energy_error={audit['energy_error']:.3g} light_error={audit['light_error']:.3g}")
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({"samples": samples, "audit": audit,
                                          "habitat": habitat.snapshot()}, indent=2, allow_nan=False) + "\n",
                               encoding="utf-8")
    if args.checkpoint:
        habitat.save(args.checkpoint)


if __name__ == "__main__":
    main()
