"""Run the hand-calculated lifecycle: python -m reference.worked_example."""

import argparse
from dataclasses import asdict
import json
import math
from pathlib import Path

from .ledger import Genome, Ledger, Organism, Resources
from .rules import WorldRules, load_rules


FIXTURE_PATH = Path(__file__).resolve().parents[1] / "examples/lifecycle_expected.json"


def load_fixture() -> dict:
    return json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))


def build_reference_world(rules: WorldRules | None = None) -> Ledger:
    rules = rules or load_rules()
    fixture = load_fixture()
    if fixture["model_id"] != rules.model_id:
        raise ValueError("Reference fixture and model rule versions differ")
    organisms = [Organism(
        item["id"], Genome(item["photosynthetic"], item["can_ingest"], tuple(item["weights"])),
        Resources(item["tissue_nutrient"], item["tissue_energy"]),
        Resources(item["reserve_nutrient"], item["reserve_energy"]), age=item["age"])
        for item in fixture["organisms"]]
    inventory = fixture["initial_inventory"]
    return Ledger(rules, mineral_nutrient=inventory["mineral_nutrient"], organisms=organisms,
                  initial_nutrient=inventory["nutrient"], initial_energy=inventory["chemical_energy"],
                  light_available=inventory["available_external_light"])


def check_expected(actual: dict, expected: dict, rules: WorldRules) -> None:
    if actual.keys() != expected.keys():
        raise AssertionError("Worked-example snapshot fields differ from the hand-calculated fixture")
    for key, value in expected.items():
        if isinstance(value, (float, int)):
            matches = math.isclose(actual[key], value, rel_tol=rules.relative_tolerance,
                                   abs_tol=rules.absolute_tolerance)
        else:
            matches = actual[key] == value
        if not matches:
            raise AssertionError(f"{expected['label']}.{key}: actual {actual[key]!r}, expected {value!r}")


def run_example() -> tuple[Ledger, list[dict]]:
    world = build_reference_world()
    actions = [
        ("photosynthesis", lambda: world.photosynthesize("producer", 10.0)),
        ("feeding", lambda: world.feed("consumer", "producer", 2.0, contact=True)),
        ("growth", lambda: world.grow("consumer", 1.0)),
        ("maintenance", lambda: world.maintain("consumer")),
        ("birth", lambda: world.reproduce("consumer", "child", placement_available=True)),
        ("damage_and_death", lambda: world.damage("producer", 8.0)),
        ("decomposition", lambda: world.decompose(fraction=1.0)),
    ]
    snapshots = [world.summary("initial")]
    for label, operation in actions:
        operation()
        snapshots.append(world.summary(label))
    expected = load_fixture()["expected_snapshots"]
    if len(snapshots) != len(expected):
        raise AssertionError("Worked-example event count changed without a corresponding specification")
    for actual, target in zip(snapshots, expected, strict=True):
        check_expected(actual, target, world.rules)
    return world, snapshots


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Optional JSON trace destination")
    args = parser.parse_args()
    world, snapshots = run_example()
    for row in snapshots:
        print(f"{row['label']:20s} N={row['total_nutrient']:7.3f} "
              f"accounted_E={row['accounted_energy']:7.3f} alive={row['living_count']} checked")
    if args.output is not None:
        payload = {"model_id": world.rules.model_id, "rules": asdict(world.rules),
                   "snapshots": snapshots, "events": world.events,
                   "final_organisms": {key: asdict(value) for key, value in world.organisms.items()},
                   "note": "Transaction reference only; not an autonomous simulation or evidence of evolution."}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(payload, indent=2, allow_nan=False) + "\n", encoding="utf-8")
        print(f"Trace written to {args.output}")
    print("All 8 snapshots match the hand-calculated reference; resource audits passed.")


if __name__ == "__main__":
    main()
