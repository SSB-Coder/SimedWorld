use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AccountingError {
    Invalid(String),
    Invariant(String),
}

impl fmt::Display for AccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Invariant(message) => message.fmt(formatter),
        }
    }
}

impl std::error::Error for AccountingError {}
pub type Result<T> = std::result::Result<T, AccountingError>;

pub fn finite(value: f64, name: &str, positive: bool) -> Result<()> {
    if !value.is_finite() || value < 0.0 || (positive && value == 0.0) {
        return Err(AccountingError::Invalid(format!(
            "{name} must be finite and {}",
            if positive { "positive" } else { "nonnegative" }
        )));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub nutrient: f64,
    pub energy: f64,
}

impl Resources {
    pub fn new(nutrient: f64, energy: f64) -> Result<Self> {
        let resources = Self { nutrient, energy };
        resources.validate()?;
        Ok(resources)
    }

    pub fn validate(self) -> Result<()> {
        finite(self.nutrient, "nutrient", false)?;
        finite(self.energy, "energy", false)
    }

    pub fn plus(self, other: Self) -> Result<Self> {
        self.validate()?;
        other.validate()?;
        Self::new(self.nutrient + other.nutrient, self.energy + other.energy)
    }

    pub fn minus(self, other: Self) -> Result<Self> {
        self.validate()?;
        other.validate()?;
        if other.nutrient > self.nutrient || other.energy > self.energy {
            return Err(AccountingError::Invalid(format!(
                "insufficient resources: requested ({}, {}), available ({}, {})",
                other.nutrient, other.energy, self.nutrient, self.energy
            )));
        }
        Self::new(self.nutrient - other.nutrient, self.energy - other.energy)
    }
}
