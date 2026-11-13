use serde_json::{Value, json};
use world_core::{
    environment::{EnvironmentGrid, Position},
    habitat::Habitat,
    resources::AccountingError,
    rules::WorldRules,
};

const RULES: &str = include_str!("../../../configs/world_rules_v0.json");
const PRESET: &str = include_str!("../../../configs/producer_habitat.json");

fn rules() -> Value {
    let mut rules: Value = serde_json::from_str(RULES).unwrap();
    for (name, value) in [
        ("world_width", json!(2)),
        ("world_height", json!(1)),
        ("grid_width", json!(2)),
        ("grid_height", json!(1)),
        ("physics_dt", json!(0.5)),
        ("ecology_interval_ticks", json!(2)),
    ] {
        rules[name] = value;
    }
    rules
}

fn config() -> Value {
    json!({"schema_version":1,"seed":0,"initial_nutrient":4.5,"initial_energy":9,
        "mineral_nutrient_per_cell":1,"light_energy_per_cell_per_time":8,"light_gradient":0,
        "producers":{"count":1,"tissue_nutrient":2,"reserve_nutrient":0.5,"reserve_energy":5,
            "positions":[{"x":0.25,"y":0.5}]}})
}

fn summary(habitat: &Habitat) -> Value {
    serde_json::from_str(&habitat.summary_json().unwrap()).unwrap()
}

#[test]
fn preset_founders_are_funded_and_repeatable() {
    let first = Habitat::from_json(RULES, PRESET).unwrap();
    let second = Habitat::from_json(RULES, PRESET).unwrap();
    assert_eq!(
        first.snapshot_json().unwrap(),
        second.snapshot_json().unwrap()
    );
    assert_eq!(summary(&first)["living_count"], 64);
    assert_eq!(first.audit().unwrap().nutrient, 4256.0);
    assert_eq!(first.audit().unwrap().energy, 576.0);
}

#[test]
fn local_window_matches_hand_calculated_inventories() {
    let mut habitat = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    habitat.advance().unwrap();
    assert_eq!(summary(&habitat)["absorbed_light"], 0.0);
    habitat.advance().unwrap();
    let result = summary(&habitat);
    for (key, expected) in [
        ("total_nutrient", 4.5),
        ("mineral_nutrient", 1.0),
        ("living_nutrient", 3.5),
        ("living_energy", 12.8),
        ("heat", 2.45),
        ("absorbed_light", 6.25),
        ("incident_light", 16.0),
        ("available_light", 9.75),
        ("accounted_energy", 15.25),
    ] {
        assert!(
            (result[key].as_f64().unwrap() - expected).abs() < 1e-10,
            "{key}"
        );
    }
    let snapshot: Value = serde_json::from_str(&habitat.snapshot_json().unwrap()).unwrap();
    assert_eq!(snapshot["organisms"][0]["organism"]["age"], 1.0);
    assert_eq!(snapshot["environment"]["cells"][1]["mineral_nutrient"], 1.0);
}

#[test]
fn colocated_producers_share_only_their_cell_inputs() {
    let mut config = config();
    config["initial_nutrient"] = json!(6);
    config["initial_energy"] = json!(8);
    config["light_energy_per_cell_per_time"] = json!(2);
    config["producers"]["count"] = json!(2);
    config["producers"]["reserve_nutrient"] = json!(0);
    config["producers"]["reserve_energy"] = json!(0);
    config["producers"]["positions"] = json!([{"x":0.25,"y":0.5},{"x":0.75,"y":0.5}]);
    let mut habitat = Habitat::from_json(&rules().to_string(), &config.to_string()).unwrap();
    habitat.advance().unwrap();
    habitat.advance().unwrap();
    let snapshot: Value = serde_json::from_str(&habitat.snapshot_json().unwrap()).unwrap();
    assert_eq!(
        snapshot["organisms"][0]["organism"]["tissue"],
        snapshot["organisms"][1]["organism"]["tissue"]
    );
    assert_eq!(summary(&habitat)["absorbed_light"], 2.0);
    assert_eq!(summary(&habitat)["last_update"]["nutrient_absorbed"], 1.0);
}

#[test]
fn starvation_returns_tissue_to_the_occupied_cell_once() {
    let mut config = config();
    config["initial_nutrient"] = json!(3);
    config["initial_energy"] = json!(4);
    config["producers"]["reserve_nutrient"] = json!(0);
    config["producers"]["reserve_energy"] = json!(0);
    config["cell_overrides"] =
        json!([{"column":0,"row":0,"mineral_nutrient":0,"light_energy_per_time":0}]);
    let mut habitat = Habitat::from_json(&rules().to_string(), &config.to_string()).unwrap();
    for _ in 0..8 {
        habitat.advance().unwrap();
    }
    let snapshot: Value = serde_json::from_str(&habitat.snapshot_json().unwrap()).unwrap();
    assert_eq!(summary(&habitat)["living_count"], 0);
    assert_eq!(
        snapshot["environment"]["cells"][0]["detritus"],
        json!({"nutrient":2.0,"energy":4.0})
    );
    assert_eq!(
        snapshot["environment"]["cells"][1]["detritus"],
        json!({"nutrient":0.0,"energy":0.0})
    );
    habitat.audit().unwrap();
}

#[test]
fn world_boundaries_map_to_valid_cells_and_reject_outside_points() {
    let rules = WorldRules::from_json(&rules().to_string()).unwrap();
    let grid = EnvironmentGrid::new(&rules, 1.0, 8.0, 0.0, &[]).unwrap();
    assert_eq!(grid.cell_index(Position { x: 0.0, y: 0.0 }).unwrap(), 0);
    assert_eq!(grid.cell_index(Position { x: 2.0, y: 1.0 }).unwrap(), 1);
    for point in [
        Position { x: -0.01, y: 0.0 },
        Position { x: 2.01, y: 0.0 },
        Position {
            x: f64::NAN,
            y: 0.0,
        },
    ] {
        assert!(grid.cell_index(point).is_err());
    }
}

#[test]
fn invalid_starting_budgets_and_overrides_are_rejected() {
    let mut invalid = config();
    invalid["initial_nutrient"] = json!(100);
    assert!(matches!(
        Habitat::from_json(&rules().to_string(), &invalid.to_string()),
        Err(AccountingError::Invariant(_))
    ));
    invalid = config();
    invalid["cell_overrides"] = json!([{"column":2,"row":0,"mineral_nutrient":0}]);
    assert!(Habitat::from_json(&rules().to_string(), &invalid.to_string()).is_err());
}

#[test]
fn failed_processing_discards_staged_cells_organisms_and_clock() {
    let mut rules = rules();
    rules["physics_dt"] = json!(1e308);
    rules["ecology_interval_ticks"] = json!(1);
    rules["maintenance_per_tissue_nutrient_per_time"] = json!(1e-308);
    rules["max_growth_nutrient_per_time"] = json!(1e308);
    let mut config = config();
    config["light_energy_per_cell_per_time"] = json!(0);
    let mut habitat = Habitat::from_json(&rules.to_string(), &config.to_string()).unwrap();
    let before = habitat.snapshot_json().unwrap();
    assert!(habitat.advance().is_err());
    assert_eq!(habitat.tick(), 0);
    assert_eq!(habitat.snapshot_json().unwrap(), before);
}

#[test]
fn repeated_scheduled_runs_and_light_budgets_agree() {
    let mut first = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    let mut second = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    for _ in 0..120 {
        first.advance().unwrap();
        second.advance().unwrap();
    }
    assert_eq!(
        first.snapshot_json().unwrap(),
        second.snapshot_json().unwrap()
    );
    let result = summary(&first);
    let input = result["incident_light"].as_f64().unwrap();
    let output: f64 = ["absorbed_light", "escaped_light", "available_light"]
        .iter()
        .map(|key| result[key].as_f64().unwrap())
        .sum();
    assert!((input - output).abs() < 1e-8);
    first.audit().unwrap();
}
