use serde::Serialize;

use crate::{
    ids::OrganismId,
    organism::{DeathCause, Organism},
    resources::Resources,
    transactions::{Operation, Pools},
};

#[derive(Clone, Debug, Serialize)]
pub struct OrganismInventory {
    pub tissue: Resources,
    pub reserves: Resources,
    pub age: f64,
    pub alive: bool,
    pub parent_id: Option<OrganismId>,
    pub next_reproduction_age: f64,
    pub death_cause: Option<DeathCause>,
}

impl From<&Organism> for OrganismInventory {
    fn from(value: &Organism) -> Self {
        Self {
            tissue: value.tissue,
            reserves: value.reserves,
            age: value.age,
            alive: value.alive,
            parent_id: value.parent_id,
            next_reproduction_age: value.next_reproduction_age,
            death_cause: value.death_cause,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OrganismChange {
    pub id: OrganismId,
    pub before: Option<OrganismInventory>,
    pub after: OrganismInventory,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResourceEvent {
    pub sequence: u64,
    pub operation: Operation,
    pub pools_before: Pools,
    pub pools_after: Pools,
    pub organisms: Vec<OrganismChange>,
    pub nutrient_error: f64,
    pub energy_error: f64,
}
