use serde_json::{Value, json};
use world_core::{
    ids::OrganismId,
    resources::{AccountingError, Resources},
    rules::WorldRules,
    transactions::{Operation, ResourceLedger},
};
use world_genetics::genome::Genome;

const RULES: &str = include_str!("../../../configs/world_rules_v0.json");
const FIXTURE: &str = include_str!("../../../examples/lifecycle_expected.json");

fn initial() -> Value {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let organisms = fixture["organisms"].as_array().unwrap().iter().enumerate().map(|(row, item)| json!({
        "id": row + 1,
        "genome": {"photosynthetic": item["photosynthetic"], "can_ingest": item["can_ingest"], "initial_weights": item["weights"]},
        "tissue": {"nutrient": item["tissue_nutrient"], "energy": item["tissue_energy"]},
        "reserves": {"nutrient": item["reserve_nutrient"], "energy": item["reserve_energy"]}, "age": item["age"]
    })).collect::<Vec<_>>();
    json!({"schema_version": 1, "initial_nutrient": fixture["initial_inventory"]["nutrient"],
        "initial_energy": fixture["initial_inventory"]["chemical_energy"],
        "mineral_nutrient": fixture["initial_inventory"]["mineral_nutrient"],
        "light_available": fixture["initial_inventory"]["available_external_light"], "organisms": organisms})
}

fn ledger() -> ResourceLedger {
    ResourceLedger::from_json(RULES, &initial().to_string()).unwrap()
}

#[test]
fn full_lifecycle_matches_independent_expected_snapshots_and_inheritance() {
    let rules = WorldRules::from_json(RULES).unwrap();
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let expected = fixture["expected_snapshots"].as_array().unwrap();
    let mut ledger = ledger();
    let mut snapshots = vec![ledger.summary_json("initial").unwrap()];
    let operations = [
        (
            "photosynthesis",
            Operation::Photosynthesize {
                organism_id: OrganismId(1),
                light_energy: 10.0,
                duration: 1.0,
            },
        ),
        (
            "feeding",
            Operation::Feed {
                consumer_id: OrganismId(2),
                target_id: OrganismId(1),
                tissue_nutrient: 2.0,
                contact: true,
                duration: 1.0,
            },
        ),
        (
            "growth",
            Operation::Grow {
                organism_id: OrganismId(2),
                tissue_nutrient: 1.0,
                duration: 1.0,
            },
        ),
        (
            "maintenance",
            Operation::Maintain {
                organism_id: OrganismId(2),
                duration: 1.0,
            },
        ),
        (
            "birth",
            Operation::Reproduce {
                parent_id: OrganismId(2),
                child_id: OrganismId(3),
                placement_available: true,
            },
        ),
        (
            "damage_and_death",
            Operation::Damage {
                organism_id: OrganismId(1),
                tissue_nutrient: 8.0,
            },
        ),
        (
            "decomposition",
            Operation::Decompose {
                fraction: 1.0,
                duration: 1.0,
            },
        ),
    ];
    for (label, operation) in operations {
        ledger.apply(operation).unwrap();
        snapshots.push(ledger.summary_json(label).unwrap());
    }
    for (actual, expected) in snapshots.iter().zip(expected) {
        let actual: Value = serde_json::from_str(actual).unwrap();
        assert_eq!(
            actual.as_object().unwrap().keys().collect::<Vec<_>>(),
            expected.as_object().unwrap().keys().collect::<Vec<_>>()
        );
        for (key, value) in expected.as_object().unwrap() {
            if let Some(target) = value.as_f64() {
                assert!(rules.close(actual[key].as_f64().unwrap(), target), "{key}");
            } else {
                assert_eq!(&actual[key], value);
            }
        }
    }
    let parent = &ledger.organisms()[&OrganismId(2)];
    let child = &ledger.organisms()[&OrganismId(3)];
    assert_eq!(child.parent_id, Some(parent.id));
    assert_eq!(child.genome, parent.genome);
    assert_eq!(child.genome.initial_weights(), &[0.25, -0.5]);
    assert_eq!(ledger.events().len(), 7);
    assert!(!ledger.organisms()[&OrganismId(1)].alive);
}

#[test]
fn rejected_operations_leave_inventory_and_receipts_unchanged() {
    let mut ledger = ledger();
    for operation in [
        Operation::Feed {
            consumer_id: OrganismId(2),
            target_id: OrganismId(1),
            tissue_nutrient: 2.0,
            contact: false,
            duration: 1.0,
        },
        Operation::Feed {
            consumer_id: OrganismId(2),
            target_id: OrganismId(2),
            tissue_nutrient: 1.0,
            contact: true,
            duration: 1.0,
        },
        Operation::Reproduce {
            parent_id: OrganismId(2),
            child_id: OrganismId(3),
            placement_available: true,
        },
        Operation::Grow {
            organism_id: OrganismId(2),
            tissue_nutrient: 3.0,
            duration: 1.0,
        },
        Operation::Actuate {
            organism_id: OrganismId(2),
            energy: f64::NAN,
        },
        Operation::DissipateMotion { energy: 1.0 },
    ] {
        let before = ledger.snapshot_json().unwrap();
        assert!(ledger.apply(operation).is_err());
        assert_eq!(ledger.snapshot_json().unwrap(), before);
    }
}

#[test]
fn starvation_commits_and_death_does_not_erase_mechanical_energy() {
    let mut ledger = ledger();
    ledger
        .apply(Operation::Actuate {
            organism_id: OrganismId(2),
            energy: 4.0,
        })
        .unwrap();
    ledger
        .apply(Operation::Maintain {
            organism_id: OrganismId(2),
            duration: 100.0,
        })
        .unwrap();
    let organism = &ledger.organisms()[&OrganismId(2)];
    assert!(!organism.alive);
    assert_eq!(organism.tissue, Resources::default());
    let summary: Value = serde_json::from_str(&ledger.summary_json("death").unwrap()).unwrap();
    assert_eq!(summary["mechanical_energy"], 2.0);
    let before = ledger.snapshot_json().unwrap();
    assert!(
        ledger
            .apply(Operation::Maintain {
                organism_id: OrganismId(2),
                duration: 1.0
            })
            .is_err()
    );
    assert_eq!(ledger.snapshot_json().unwrap(), before);
    ledger
        .apply(Operation::DissipateMotion { energy: 2.0 })
        .unwrap();
    ledger.audit().unwrap();
}

#[test]
fn malformed_initial_states_and_rule_investments_fail() {
    let mut value = initial();
    value["initial_nutrient"] = json!(101);
    assert!(matches!(
        ResourceLedger::from_json(RULES, &value.to_string()),
        Err(AccountingError::Invariant(_))
    ));
    let mut rules: Value = serde_json::from_str(RULES).unwrap();
    rules["offspring_reserve_energy"] = json!(100);
    assert!(WorldRules::from_json(&rules.to_string()).is_err());
    let mut value = initial();
    value["organisms"][0]["genome"]["initial_weights"] = json!(["bad"]);
    assert!(ResourceLedger::from_json(RULES, &value.to_string()).is_err());
}

#[test]
fn resource_arithmetic_and_inherited_parameters_reject_nonfinite_values() {
    assert!(Resources::new(-1.0, 0.0).is_err());
    assert!(Resources::new(0.0, f64::INFINITY).is_err());
    assert!(
        Resources::new(0.0, 1e308)
            .unwrap()
            .plus(Resources::new(0.0, 1e308).unwrap())
            .is_err()
    );
    assert!(
        Resources::new(1.0, 1.0)
            .unwrap()
            .minus(Resources::new(2.0, 0.0).unwrap())
            .is_err()
    );
    assert!(Genome::new(false, true, vec![f64::NAN]).is_err());
}
