use std::fmt;

use crate::ids::OrganismId;

pub const SCHEMA_VERSION: u32 = 1;
pub const OBSERVATION_WIDTH: usize = 3;
pub const ACTION_WIDTH: usize = 2;

#[derive(Debug)]
pub struct ContractError(pub String);

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ContractError {}

pub type Result<T> = std::result::Result<T, ContractError>;

#[derive(Clone, Debug)]
pub struct ObservationBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub ids: Vec<OrganismId>,
    pub topology_versions: Vec<u64>,
    pub values: Vec<[f32; OBSERVATION_WIDTH]>,
}

#[derive(Clone, Debug)]
pub struct ActionBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub ids: Vec<OrganismId>,
    pub topology_versions: Vec<u64>,
    pub values: Vec<[f32; ACTION_WIDTH]>,
}

pub fn validate_action(action: &[f32; ACTION_WIDTH]) -> Result<()> {
    if !action.iter().all(|value| value.is_finite())
        || !(-1.0..=1.0).contains(&action[0])
        || !(0.0..=1.0).contains(&action[1])
    {
        return Err(ContractError(
            "actions require turn in [-1, 1] and thrust in [0, 1]".into(),
        ));
    }
    Ok(())
}
