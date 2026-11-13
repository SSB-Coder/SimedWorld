use crate::resources::{AccountingError, Result, finite};
use serde::{Deserialize, Serialize};
use world_genetics::mutation::MutationConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvolutionConfig {
    pub energy_budgeting: bool,
    pub catabolism_fraction_per_time: f64,
    pub catabolism_efficiency: f64,
    pub foraging_reserve_time: f64,
    pub juvenile_reserve_time: f64,
    pub energy_history_samples: usize,
    pub energy_history_interval_ticks: u64,
    pub embodied_ecology: bool,
    pub social_observations: bool,
    pub inherit_learning: bool,
    pub founder_age_spread: f64,
    pub trace_time: f64,
    pub exploration: f64,
    pub injury_feedback: f64,
    pub reproduction_feedback: f64,
    pub temporal_feedback: bool,
    pub plasticity_decay_per_time: f64,
    pub learned_weight_updates: bool,
    pub neural_development: bool,
    pub allow_communication: bool,
    pub allow_filaments: bool,
    pub allow_camouflage: bool,
    pub receive_signals: bool,
    pub terrain: Vec<crate::terrain::TerrainPatch>,
    pub development_interval_ticks: u64,
    pub online_learning: bool,
    pub maintenance_reserve_time: f64,
    pub mutation: MutationConfig,
    pub neuron_energy_per_time: f64,
    pub connection_energy_per_time: f64,
    pub plasticity_energy_per_connection_time: f64,
    pub module_upkeep_per_time: f64,
    pub attack_energy_per_time: f64,
    pub attack_nutrient_per_time: f64,
    pub repair_nutrient_per_time: f64,
    pub body_drag: f64,
    pub work_energy_scale: f64,
    pub history_limit: usize,
    pub offspring_module_nutrient: f64,
    pub offspring_neuron_nutrient: f64,
    pub offspring_connection_nutrient: f64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            energy_budgeting: false,
            catabolism_fraction_per_time: 0.0,
            catabolism_efficiency: 0.8,
            foraging_reserve_time: 0.0,
            juvenile_reserve_time: 0.0,
            energy_history_samples: 0,
            energy_history_interval_ticks: 60,
            embodied_ecology: false,
            social_observations: true,
            inherit_learning: true,
            founder_age_spread: 0.0,
            trace_time: 1.5,
            exploration: 0.18,
            injury_feedback: 0.0,
            reproduction_feedback: 0.0,
            temporal_feedback: false,
            plasticity_decay_per_time: 0.0,
            learned_weight_updates: true,
            neural_development: true,
            allow_communication: true,
            allow_filaments: true,
            allow_camouflage: true,
            receive_signals: true,
            terrain: Vec::new(),
            development_interval_ticks: 600,
            online_learning: false,
            maintenance_reserve_time: 0.0,
            mutation: MutationConfig::default(),
            neuron_energy_per_time: 0.002,
            connection_energy_per_time: 0.0001,
            plasticity_energy_per_connection_time: 0.0,
            module_upkeep_per_time: 0.01,
            attack_energy_per_time: 1.0,
            attack_nutrient_per_time: 1.0,
            repair_nutrient_per_time: 0.5,
            body_drag: 1.0,
            work_energy_scale: 0.04,
            history_limit: 4096,
            offspring_module_nutrient: 0.08,
            offspring_neuron_nutrient: 0.0001,
            offspring_connection_nutrient: 0.00001,
        }
    }
}

impl EvolutionConfig {
    pub fn validate(&self) -> Result<()> {
        for value in [
            self.catabolism_fraction_per_time,
            self.foraging_reserve_time,
            self.juvenile_reserve_time,
        ] {
            finite(value, "metabolic setting", false)?;
        }
        if !(0.0..=1.0).contains(&self.catabolism_fraction_per_time)
            || !self.catabolism_efficiency.is_finite()
            || !(0.0..=1.0).contains(&self.catabolism_efficiency)
            || self.catabolism_efficiency == 0.0
            || self.foraging_reserve_time > 1000.0
            || self.juvenile_reserve_time > 1000.0
            || self.energy_history_samples > 64
            || self.energy_history_interval_ticks == 0
        {
            return Err(AccountingError::Invalid(
                "invalid metabolic settings".into(),
            ));
        }
        for value in [
            self.founder_age_spread,
            self.injury_feedback,
            self.reproduction_feedback,
            self.plasticity_decay_per_time,
        ] {
            finite(value, "ecological feedback", false)?;
        }
        if !(0.1..=100.0).contains(&self.trace_time)
            || !(0.01..=0.5).contains(&self.exploration)
            || self.founder_age_spread > 1000.0
            || self.injury_feedback > 10.0
            || self.reproduction_feedback > 10.0
            || self.plasticity_decay_per_time > 1.0
        {
            return Err(AccountingError::Invalid(
                "invalid learning or founder settings".into(),
            ));
        }
        if self.terrain.len() > 256 || self.development_interval_ticks == 0 {
            return Err(AccountingError::Invalid(
                "invalid terrain/development limits".into(),
            ));
        }
        finite(
            self.plasticity_energy_per_connection_time,
            "plasticity cost",
            false,
        )?;
        finite(
            self.maintenance_reserve_time,
            "maintenance reserve horizon",
            false,
        )?;
        self.mutation
            .validate()
            .map_err(|e| AccountingError::Invalid(e.into()))?;
        for v in [
            self.neuron_energy_per_time,
            self.connection_energy_per_time,
            self.module_upkeep_per_time,
            self.attack_energy_per_time,
            self.attack_nutrient_per_time,
            self.repair_nutrient_per_time,
            self.body_drag,
            self.work_energy_scale,
            self.offspring_module_nutrient,
            self.offspring_neuron_nutrient,
            self.offspring_connection_nutrient,
        ] {
            finite(v, "evolution cost or rate", true)?;
        }
        finite(
            self.neuron_energy_per_time * self.mutation.max_nodes as f64
                + (self.connection_energy_per_time + self.plasticity_energy_per_connection_time)
                    * self.mutation.max_connections as f64,
            "maximum neural expenditure rate",
            true,
        )?;
        if !(128..=65536).contains(&self.history_limit) {
            return Err(AccountingError::Invalid("invalid history limit".into()));
        }
        Ok(())
    }
}
