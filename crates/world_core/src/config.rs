use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::contracts::{ContractError, OBSERVATION_WIDTH, Result, SCHEMA_VERSION};
use crate::ids::OrganismId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureOrganism {
    pub id: OrganismId,
    pub topology_version: u64,
    pub signals: [f32; OBSERVATION_WIDTH],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureConfig {
    pub schema_version: u32,
    pub organisms: Vec<FixtureOrganism>,
}

pub fn validate_organisms(organisms: &[FixtureOrganism]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for organism in organisms {
        if !seen.insert(organism.id) {
            return Err(ContractError("duplicate organism ID".into()));
        }
        if !organism
            .signals
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(ContractError("signals must be finite and in [0, 1]".into()));
        }
    }
    Ok(())
}

impl FixtureConfig {
    pub fn from_json(text: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(text)
            .map_err(|error| ContractError(format!("invalid fixture config: {error}")))?;
        if config.schema_version != SCHEMA_VERSION {
            return Err(ContractError("unsupported config schema".into()));
        }
        validate_organisms(&config.organisms)?;
        Ok(config)
    }
}
