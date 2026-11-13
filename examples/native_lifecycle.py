import argparse
import json
import math
from pathlib import Path

from world import ResourceLedger


ROOT = Path(__file__).resolve().parents[1]
IDS = {"producer": 1, "consumer": 2, "child": 3}


def initial_from_fixture(fixture: dict) -> dict:
    inventory = fixture["initial_inventory"]
    organisms = [{"id": IDS[item["id"]],
                  "genome": {"photosynthetic": item["photosynthetic"], "can_ingest": item["can_ingest"],
                             "initial_weights": item["weights"]},
                  "tissue": {"nutrient": item["tissue_nutrient"], "energy": item["tissue_energy"]},
                  "reserves": {"nutrient": item["reserve_nutrient"], "energy": item["reserve_energy"]},
                  "age": item["age"]} for item in fixture["organisms"]]
    return {"schema_version": 1, "initial_nutrient": inventory["nutrient"],
            "initial_energy": inventory["chemical_energy"], "mineral_nutrient": inventory["mineral_nutrient"],
            "light_available": inventory["available_external_light"], "organisms": organisms}


def run_example():
    rules = json.loads((ROOT / "configs/world_rules_v0.json").read_text(encoding="utf-8"))
    fixture = json.loads((ROOT / "examples/lifecycle_expected.json").read_text(encoding="utf-8"))
    if rules["model_id"] != fixture["model_id"]:
        raise ValueError("fixture and rules have different model IDs")
    ledger = ResourceLedger(rules, initial_from_fixture(fixture))
    operations = [
        ("photosynthesis", "photosynthesize", {"organism_id": 1, "light_energy": 10.0}),
        ("feeding", "feed", {"consumer_id": 2, "target_id": 1, "tissue_nutrient": 2.0, "contact": True}),
        ("growth", "grow", {"organism_id": 2, "tissue_nutrient": 1.0}),
        ("maintenance", "maintain", {"organism_id": 2}),
        ("birth", "reproduce", {"parent_id": 2, "child_id": 3, "placement_available": True}),
        ("damage_and_death", "damage", {"organism_id": 1, "tissue_nutrient": 8.0}),
        ("decomposition", "decompose", {"fraction": 1.0}),
    ]
    snapshots = [ledger.summary("initial")]
    for label, operation, arguments in operations:
        ledger.apply(operation, **arguments)
        snapshots.append(ledger.summary(label))
    for actual, expected in zip(snapshots, fixture["expected_snapshots"], strict=True):
        if actual.keys() != expected.keys():
            raise AssertionError("native lifecycle snapshot fields differ")
        for key, target in expected.items():
            matches = (math.isclose(actual[key], target, abs_tol=rules["absolute_tolerance"],
                                   rel_tol=rules["relative_tolerance"]) if isinstance(target, (int, float))
                       else actual[key] == target)
            if not matches:
                raise AssertionError(f"{expected['label']}.{key}: {actual[key]!r} != {target!r}")
    return ledger, snapshots


def main():
    parser = argparse.ArgumentParser(description="Run the native resource lifecycle against expected results.")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    ledger, snapshots = run_example()
    for row in snapshots:
        print(f"{row['label']:20s} N={row['total_nutrient']:7.3f} accounted_E={row['accounted_energy']:7.3f} alive={row['living_count']} checked")
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({"snapshots": snapshots, "ledger": ledger.snapshot(),
                                          "scope": "Resource transactions; no scheduler or spatial world."},
                                         indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print("All native lifecycle snapshots match the hand-calculated reference.")


if __name__ == "__main__":
    main()
