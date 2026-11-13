use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Adaptations {
    pub energy_senses: bool,
    pub social_senses: bool,
    pub foraging_choices: bool,
    pub offense: f64,
    pub sociality: f64,
    pub agency: bool,
    pub anchored: bool,
    pub camouflage: f64,
    pub pigment: f64,
    pub armor: f64,
    pub capture: f64,
    pub digestion: f64,
    pub filament: f64,
    pub communication: f64,
    pub burst: f64,
    pub neural_growth: f64,
    pub neural_pruning: f64,
    pub inherit_structure: bool,
}

impl Adaptations {
    pub fn validate(&self) -> Result<(), &'static str> {
        if [
            self.offense,
            self.sociality,
            self.camouflage,
            self.pigment,
            self.armor,
            self.capture,
            self.digestion,
            self.filament,
            self.communication,
            self.burst,
            self.neural_growth,
            self.neural_pruning,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("adaptation investment must lie within [0,1]");
        }
        Ok(())
    }

    pub fn investments(&self) -> f64 {
        self.camouflage
            + self.offense
            + self.sociality * 0.2
            + self.armor
            + self.filament
            + self.communication
            + self.burst
            + self.digestion
    }
}
