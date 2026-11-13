use serde::{Deserialize, Serialize};

use crate::{
    environment::{CellOverride, Position},
    resources::{AccountingError, Result, finite},
    rules::WorldRules,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FounderPopulation {
    pub count: usize,
    pub tissue_nutrient: f64,
    pub reserve_nutrient: f64,
    pub reserve_energy: f64,
    #[serde(default)]
    pub positions: Option<Vec<Position>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genome: Option<world_genetics::genome::Genome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HabitatConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub mineral_nutrient_per_cell: f64,
    pub light_energy_per_cell_per_time: f64,
    pub light_gradient: f64,
    pub producers: FounderPopulation,
    #[serde(default)]
    pub consumers: Option<FounderPopulation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub populations: Vec<FounderPopulation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution: Option<crate::evolution_config::EvolutionConfig>,
    #[serde(default)]
    pub ecology: EcologyConfig,
    #[serde(default)]
    pub cell_overrides: Vec<CellOverride>,
}

impl HabitatConfig {
    pub fn from_json(text: &str) -> Result<Self> {
        serde_json::from_str(text)
            .map_err(|error| AccountingError::Invalid(format!("invalid habitat config: {error}")))
    }

    pub fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported habitat schema".into(),
            ));
        }
        finite(self.initial_nutrient, "initial nutrient", false)?;
        finite(self.initial_energy, "initial energy", false)?;
        self.producers.validate(rules)?;
        if let Some(consumers) = &self.consumers {
            consumers.validate(rules)?;
        }
        self.ecology.validate(rules)?;
        if let Some(config) = &self.evolution {
            config.validate()?;
            for patch in &config.terrain {
                patch.validate(rules.world_width, rules.world_height)?;
            }
            if rules.ecology_interval_ticks != rules.controller_interval_ticks {
                return Err(AccountingError::Invalid(
                    "neural and ecology intervals must match".into(),
                ));
            }
        }
        let mut count = self
            .producers
            .count
            .checked_add(self.consumers.as_ref().map_or(0, |p| p.count))
            .ok_or_else(|| AccountingError::Invalid("founder count overflow".into()))?;
        for founders in &self.populations {
            founders.validate(rules)?;
            count = count
                .checked_add(founders.count)
                .ok_or_else(|| AccountingError::Invalid("founder count overflow".into()))?;
        }
        for founders in std::iter::once(&self.producers)
            .chain(self.consumers.iter())
            .chain(self.populations.iter())
        {
            if self.evolution.is_some()
                && founders.count > 0
                && founders.genome.as_ref().is_none_or(|g| g.body.is_none())
            {
                return Err(AccountingError::Invalid(
                    "evolving founders require body blueprints".into(),
                ));
            }
            if self.evolution.is_none()
                && founders
                    .genome
                    .as_ref()
                    .is_some_and(|g| g.body.is_some() || g.brain.is_some())
            {
                return Err(AccountingError::Invalid(
                    "body/neural blueprints require an evolution configuration".into(),
                ));
            }
        }
        if count > self.ecology.max_population {
            return Err(AccountingError::Invalid(
                "founders exceed the live population limit".into(),
            ));
        }
        Ok(())
    }
}

impl FounderPopulation {
    fn validate(&self, rules: &WorldRules) -> Result<()> {
        if let Some(genome) = &self.genome {
            genome
                .validate()
                .map_err(|e| AccountingError::Invalid(e.into()))?;
        }
        finite(self.tissue_nutrient, "founder tissue", true)?;
        finite(self.reserve_nutrient, "founder nutrient reserve", false)?;
        finite(self.reserve_energy, "founder energy reserve", false)?;
        let genome = self.genome.as_ref().and_then(|g| g.body.as_ref());
        let body = genome
            .map(|g| crate::anatomy::BodyState::new(g, self.tissue_nutrient))
            .transpose()?;
        let capacity =
            crate::anatomy::reserve_capacity(self.tissue_nutrient, genome, body.as_ref(), rules)?;
        let n_cap = capacity.nutrient;
        let e_cap = capacity.energy;
        finite(n_cap, "founder nutrient capacity", true)?;
        finite(e_cap, "founder energy capacity", true)?;
        if self.tissue_nutrient < rules.minimum_viable_tissue_nutrient
            || self.reserve_nutrient > n_cap
            || self.reserve_energy > e_cap
        {
            return Err(AccountingError::Invalid(
                "founder tissue or reserves are invalid".into(),
            ));
        }
        if let Some(positions) = &self.positions
            && positions.len() != self.count
        {
            return Err(AccountingError::Invalid(
                "one position per founder is required".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EcologyConfig {
    pub turn_rate_per_time: f64,
    pub producer_litter_fraction_per_time: f64,
    pub producer_minimum_reproduction_age: f64,
    pub producer_reproduction_cooldown: f64,
    pub producer_reproduction_reserve_fraction: f64,
    pub reproduction: bool,
    pub decomposition: bool,
    pub consumer_speed: f64,
    pub movement_energy_per_distance: f64,
    pub perception_radius: f64,
    pub feeding_radius: f64,
    pub birth_radius: f64,
    pub minimum_spacing: f64,
    pub decomposition_fraction_per_time: f64,
    pub grazing_residual_fraction: f64,
    pub mineral_diffusion_per_time: f64,
    pub max_population: usize,
}

impl Default for EcologyConfig {
    fn default() -> Self {
        Self {
            turn_rate_per_time: 3.0,
            producer_litter_fraction_per_time: 0.0,
            producer_minimum_reproduction_age: 0.0,
            producer_reproduction_cooldown: 0.0,
            producer_reproduction_reserve_fraction: 0.0,
            reproduction: false,
            decomposition: false,
            consumer_speed: 1.3,
            movement_energy_per_distance: 0.1,
            perception_radius: 3.0,
            feeding_radius: 0.65,
            birth_radius: 0.9,
            minimum_spacing: 0.25,
            decomposition_fraction_per_time: 0.1,
            grazing_residual_fraction: 0.0,
            mineral_diffusion_per_time: 0.0,
            max_population: 1024,
        }
    }
}

impl EcologyConfig {
    pub fn validate(&self, rules: &WorldRules) -> Result<()> {
        finite(self.turn_rate_per_time, "turn rate", true)?;
        finite(
            self.producer_litter_fraction_per_time,
            "producer litter rate",
            false,
        )?;
        if self.producer_litter_fraction_per_time > 1.0 {
            return Err(AccountingError::Invalid(
                "producer litter rate exceeds one per time unit".into(),
            ));
        }
        finite(
            self.producer_minimum_reproduction_age,
            "producer reproduction age",
            false,
        )?;
        finite(
            self.producer_reproduction_cooldown,
            "producer reproduction cooldown",
            false,
        )?;
        if !self.producer_reproduction_reserve_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.producer_reproduction_reserve_fraction)
        {
            return Err(AccountingError::Invalid(
                "producer reserve fraction must lie in [0,1]".into(),
            ));
        }
        finite(
            self.mineral_diffusion_per_time,
            "mineral diffusivity",
            false,
        )?;
        if !self.grazing_residual_fraction.is_finite()
            || !(0.0..=0.9).contains(&self.grazing_residual_fraction)
        {
            return Err(AccountingError::Invalid(
                "grazing residual fraction must lie in [0,0.9]".into(),
            ));
        }
        finite(self.consumer_speed, "consumer speed", false)?;
        finite(
            self.decomposition_fraction_per_time,
            "decomposition fraction per time",
            false,
        )?;
        for (name, value) in [
            ("movement cost", self.movement_energy_per_distance),
            ("perception radius", self.perception_radius),
            ("feeding radius", self.feeding_radius),
            ("birth radius", self.birth_radius),
            ("minimum spacing", self.minimum_spacing),
        ] {
            finite(value, name, true)?;
        }
        if self.max_population == 0
            || self.feeding_radius > self.perception_radius
            || self.minimum_spacing >= self.feeding_radius
            || self.minimum_spacing > self.birth_radius
        {
            return Err(AccountingError::Invalid(
                "invalid population limit or interaction radii".into(),
            ));
        }
        finite(
            rules.world_width.hypot(rules.world_height),
            "world diagonal",
            true,
        )?;
        Ok(())
    }
}

pub(crate) fn seeded_position(seed: u64, ordinal: u64, width: f64, height: f64) -> Position {
    fn unit(seed: u64, counter: u64) -> f64 {
        let mut value = seed.wrapping_add(counter.wrapping_add(1).wrapping_mul(0x9e3779b97f4a7c15));
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((value ^ (value >> 31)) >> 11) as f64 / 9_007_199_254_740_992.0
    }
    Position {
        x: width * unit(seed, ordinal.wrapping_mul(2)),
        y: height * unit(seed, ordinal.wrapping_mul(2).wrapping_add(1)),
    }
}
