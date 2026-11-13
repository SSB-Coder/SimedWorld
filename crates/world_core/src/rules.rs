use serde::{Deserialize, Serialize};

use crate::resources::{AccountingError, Result, finite};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldRules {
    pub schema_version: u32,
    pub model_id: String,
    pub world_width: f64,
    pub world_height: f64,
    pub grid_width: u64,
    pub grid_height: u64,
    pub physics_dt: f64,
    pub controller_interval_ticks: u64,
    pub ecology_interval_ticks: u64,
    pub tissue_energy_per_nutrient: f64,
    pub construction_heat_per_nutrient: f64,
    pub maintenance_per_tissue_nutrient_per_time: f64,
    pub minimum_viable_tissue_nutrient: f64,
    pub maturity_tissue_nutrient: f64,
    pub minimum_reproduction_age: f64,
    pub reserve_nutrient_capacity_per_tissue_nutrient: f64,
    pub reserve_energy_capacity_per_tissue_nutrient: f64,
    pub nutrient_assimilation_fraction: f64,
    pub energy_assimilation_fraction: f64,
    pub digestion_energy_per_ingested_nutrient: f64,
    pub max_ingestion_nutrient_per_time: f64,
    pub max_absorption_nutrient_per_time: f64,
    pub max_growth_nutrient_per_time: f64,
    pub max_decomposition_nutrient_per_time: f64,
    pub max_decomposition_energy_per_time: f64,
    pub photosynthesis_efficiency: f64,
    pub max_absorbed_light_per_time: f64,
    pub offspring_tissue_nutrient: f64,
    pub offspring_reserve_nutrient: f64,
    pub offspring_reserve_energy: f64,
    pub reproduction_cooldown: f64,
    pub sensor_energy_per_time: f64,
    pub movement_efficiency: f64,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}

impl WorldRules {
    pub fn from_json(text: &str) -> Result<Self> {
        let rules: Self = serde_json::from_str(text)
            .map_err(|error| AccountingError::Invalid(format!("invalid rules: {error}")))?;
        rules.validate()?;
        Ok(rules)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.model_id.trim().is_empty() {
            return Err(AccountingError::Invalid(
                "unsupported rules schema or empty model ID".into(),
            ));
        }
        for (name, value) in [
            ("world_width", self.world_width),
            ("world_height", self.world_height),
            ("physics_dt", self.physics_dt),
            (
                "tissue_energy_per_nutrient",
                self.tissue_energy_per_nutrient,
            ),
            (
                "construction_heat_per_nutrient",
                self.construction_heat_per_nutrient,
            ),
            (
                "maintenance_per_tissue_nutrient_per_time",
                self.maintenance_per_tissue_nutrient_per_time,
            ),
            (
                "minimum_viable_tissue_nutrient",
                self.minimum_viable_tissue_nutrient,
            ),
            ("maturity_tissue_nutrient", self.maturity_tissue_nutrient),
            ("minimum_reproduction_age", self.minimum_reproduction_age),
            (
                "reserve_nutrient_capacity_per_tissue_nutrient",
                self.reserve_nutrient_capacity_per_tissue_nutrient,
            ),
            (
                "reserve_energy_capacity_per_tissue_nutrient",
                self.reserve_energy_capacity_per_tissue_nutrient,
            ),
            (
                "nutrient_assimilation_fraction",
                self.nutrient_assimilation_fraction,
            ),
            (
                "energy_assimilation_fraction",
                self.energy_assimilation_fraction,
            ),
            (
                "digestion_energy_per_ingested_nutrient",
                self.digestion_energy_per_ingested_nutrient,
            ),
            (
                "max_ingestion_nutrient_per_time",
                self.max_ingestion_nutrient_per_time,
            ),
            (
                "max_absorption_nutrient_per_time",
                self.max_absorption_nutrient_per_time,
            ),
            (
                "max_growth_nutrient_per_time",
                self.max_growth_nutrient_per_time,
            ),
            (
                "max_decomposition_nutrient_per_time",
                self.max_decomposition_nutrient_per_time,
            ),
            (
                "max_decomposition_energy_per_time",
                self.max_decomposition_energy_per_time,
            ),
            ("photosynthesis_efficiency", self.photosynthesis_efficiency),
            (
                "max_absorbed_light_per_time",
                self.max_absorbed_light_per_time,
            ),
            ("offspring_tissue_nutrient", self.offspring_tissue_nutrient),
            (
                "offspring_reserve_nutrient",
                self.offspring_reserve_nutrient,
            ),
            ("offspring_reserve_energy", self.offspring_reserve_energy),
            ("reproduction_cooldown", self.reproduction_cooldown),
            ("sensor_energy_per_time", self.sensor_energy_per_time),
            ("movement_efficiency", self.movement_efficiency),
            ("absolute_tolerance", self.absolute_tolerance),
            ("relative_tolerance", self.relative_tolerance),
        ] {
            finite(value, name, true)?;
        }
        if [
            self.grid_width,
            self.grid_height,
            self.controller_interval_ticks,
            self.ecology_interval_ticks,
        ]
        .contains(&0)
        {
            return Err(AccountingError::Invalid(
                "grid sizes and update intervals must be positive".into(),
            ));
        }
        if [
            self.nutrient_assimilation_fraction,
            self.energy_assimilation_fraction,
            self.photosynthesis_efficiency,
            self.movement_efficiency,
        ]
        .iter()
        .any(|value| *value > 1.0)
        {
            return Err(AccountingError::Invalid(
                "efficiencies cannot exceed one".into(),
            ));
        }
        if self.maturity_tissue_nutrient < self.minimum_viable_tissue_nutrient
            || self.offspring_tissue_nutrient < self.minimum_viable_tissue_nutrient
        {
            return Err(AccountingError::Invalid(
                "mature and newborn tissue must be viable".into(),
            ));
        }
        let nutrient_capacity =
            self.offspring_tissue_nutrient * self.reserve_nutrient_capacity_per_tissue_nutrient;
        let energy_capacity =
            self.offspring_tissue_nutrient * self.reserve_energy_capacity_per_tissue_nutrient;
        finite(nutrient_capacity, "offspring nutrient capacity", true)?;
        finite(energy_capacity, "offspring energy capacity", true)?;
        if self.offspring_reserve_nutrient > nutrient_capacity
            || self.offspring_reserve_energy > energy_capacity
        {
            return Err(AccountingError::Invalid(
                "offspring investment exceeds capacity".into(),
            ));
        }
        Ok(())
    }

    pub fn close(&self, a: f64, b: f64) -> bool {
        a.is_finite()
            && b.is_finite()
            && (a - b).abs()
                <= self
                    .absolute_tolerance
                    .max(self.relative_tolerance * a.abs().max(b.abs()))
    }
}
