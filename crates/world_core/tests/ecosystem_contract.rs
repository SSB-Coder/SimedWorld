use world_core::habitat::Habitat;

const RULES: &str = include_str!("../../../configs/ecosystem_rules.json");
const CONFIG: &str = include_str!("../../../configs/ecosystem_habitat.json");

#[test]
fn native_checkpoint_continues_without_a_viewer_or_python_runtime() {
    let mut uninterrupted = Habitat::from_json(RULES, CONFIG).unwrap();
    for _ in 0..137 {
        uninterrupted.advance().unwrap();
    }
    let mut restored =
        Habitat::from_checkpoint_json(&uninterrupted.checkpoint_json().unwrap()).unwrap();
    for _ in 0..600 {
        uninterrupted.advance().unwrap();
        restored.advance().unwrap();
    }
    assert_eq!(
        uninterrupted.snapshot_json().unwrap(),
        restored.snapshot_json().unwrap()
    );
    assert!(restored.metrics().births > 0);
    assert!(restored.metrics().feeding_events > 0);
    assert!(restored.metrics().decomposed_nutrient > 0.0);
    restored.audit().unwrap();
}
