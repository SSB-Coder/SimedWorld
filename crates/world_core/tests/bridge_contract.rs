use world_core::{contracts::ActionBatch, ids::OrganismId, world::FixtureWorld};

const CONFIG: &str = include_str!("../../../configs/bridge_fixture.json");

fn actions(world: &FixtureWorld) -> ActionBatch {
    let observations = world.observe();
    ActionBatch {
        schema_version: observations.schema_version,
        tick: observations.tick,
        ids: observations.ids,
        topology_versions: observations.topology_versions,
        values: vec![[0.5, 0.25], [-0.75, 0.5], [0.0, 1.0]],
    }
}

#[test]
fn reordered_actions_are_routed_by_full_width_identity() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    let mut batch = actions(&world);
    batch.ids.reverse();
    batch.topology_versions.reverse();
    batch.values.reverse();
    world.apply_actions(batch).unwrap();
    let snapshot: serde_json::Value =
        serde_json::from_str(&world.snapshot_json().unwrap()).unwrap();
    assert_eq!(snapshot["tick"], 1);
    assert_eq!(snapshot["organisms"][0]["last_action"][0], 0.5);
    assert_eq!(
        snapshot["organisms"][2]["organism"]["id"],
        9_007_199_254_740_993_u64
    );
    assert_eq!(snapshot["organisms"][2]["action_total"][1], 1.0);
}

#[test]
fn invalid_batches_never_partially_commit() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    let good = actions(&world);
    let mut invalid = Vec::new();
    let mut batch = good.clone();
    batch.ids[2] = batch.ids[0];
    invalid.push(batch);
    let mut batch = good.clone();
    batch.ids[2] = OrganismId(999);
    invalid.push(batch);
    let mut batch = good.clone();
    batch.topology_versions[2] += 1;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.tick += 1;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.schema_version += 1;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.values[2][0] = f32::NAN;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.values[2][1] = -0.1;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.values[2][0] = 1.1;
    invalid.push(batch);
    let mut batch = good.clone();
    batch.ids.pop();
    invalid.push(batch);
    let mut batch = good.clone();
    batch.topology_versions.pop();
    invalid.push(batch);
    let mut batch = good.clone();
    batch.values.pop();
    invalid.push(batch);
    for batch in invalid {
        let before = world.snapshot_json().unwrap();
        assert!(world.apply_actions(batch).is_err());
        assert_eq!(world.snapshot_json().unwrap(), before);
    }
    world.apply_actions(good.clone()).unwrap();
    let before = world.snapshot_json().unwrap();
    assert!(world.apply_actions(good).is_err());
    assert_eq!(world.snapshot_json().unwrap(), before);
}

#[test]
fn invalid_configs_are_rejected() {
    for text in [
        r#"{"schema_version":2,"organisms":[]}"#,
        r#"{"schema_version":1,"organisms":[],"extra":0}"#,
        r#"{"schema_version":1,"organisms":[{"id":1,"topology_version":0,"signals":[0,0,2]}]}"#,
        r#"{"schema_version":1,"organisms":[{"id":1,"topology_version":0,"signals":[0,0,0]},{"id":1,"topology_version":0,"signals":[0,0,0]}]}"#,
    ] {
        assert!(FixtureWorld::from_config_json(text).is_err());
    }
}

#[test]
fn snapshots_preserve_state_and_clock_overflow_is_atomic() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    world.apply_actions(actions(&world)).unwrap();
    let saved = world.snapshot_json().unwrap();
    let restored = FixtureWorld::from_snapshot_json(&saved).unwrap();
    assert_eq!(restored.snapshot_json().unwrap(), saved);
    let mut payload: serde_json::Value = serde_json::from_str(&saved).unwrap();
    payload["tick"] = serde_json::json!(u64::MAX);
    let mut exhausted = FixtureWorld::from_snapshot_json(&payload.to_string()).unwrap();
    let before = exhausted.snapshot_json().unwrap();
    assert!(exhausted.apply_actions(actions(&exhausted)).is_err());
    assert_eq!(exhausted.snapshot_json().unwrap(), before);
}
