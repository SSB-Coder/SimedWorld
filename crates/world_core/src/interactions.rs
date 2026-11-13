use crate::{
    environment::Position,
    ids::OrganismId,
    resources::{AccountingError, Resources, Result},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filament {
    pub owner: OrganismId,
    pub anchor: Position,
    pub end: Position,
    pub cell: usize,
    pub material: Resources,
    pub strength: f64,
    pub remaining: f64,
    pub captured: Option<OrganismId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signal {
    pub owner: OrganismId,
    pub position: Position,
    pub value: f64,
    pub range: f64,
    pub remaining: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InteractionState {
    pub filaments: Vec<Filament>,
    pub signals: Vec<Signal>,
    pub food_web: crate::food_web::FoodWeb,
}

impl InteractionState {
    pub fn validate(&self, habitat: &crate::habitat::Habitat) -> Result<()> {
        let mut bound = vec![std::collections::BTreeMap::new(); habitat.grid.cells.len()];
        let mut owners = std::collections::BTreeSet::new();
        for trap in &self.filaments {
            trap.material.validate()?;
            habitat.grid.cell_index(trap.anchor)?;
            habitat.grid.cell_index(trap.end)?;
            if !owners.insert(trap.owner)
                || !habitat.organisms().contains_key(&trap.owner)
                || trap.cell >= bound.len()
                || !trap.strength.is_finite()
                || trap.strength <= 0.0
                || !trap.remaining.is_finite()
                || trap.remaining <= 0.0
                || trap
                    .captured
                    .is_some_and(|id| !habitat.organisms().contains_key(&id))
            {
                return Err(AccountingError::Invalid("invalid filament state".into()));
            }
            bound[trap.cell].insert(trap.owner, trap.material);
        }
        for (reserved, cell) in bound.iter().zip(&habitat.grid.cells) {
            if *reserved != cell.filaments {
                return Err(AccountingError::Invariant(
                    "filament inventory disagrees with active traps".into(),
                ));
            }
        }

        let mut signals = std::collections::BTreeSet::new();
        for s in &self.signals {
            habitat.grid.cell_index(s.position)?;
            if !signals.insert(s.owner)
                || !habitat.organisms().contains_key(&s.owner)
                || !s.value.is_finite()
                || !(0.0..=1.0).contains(&s.value)
                || !s.range.is_finite()
                || s.range <= 0.0
                || !s.remaining.is_finite()
                || s.remaining <= 0.0
            {
                return Err(AccountingError::Invalid("invalid local signal".into()));
            }
        }
        for (id, v) in &self.food_web.trophic {
            if !habitat.organisms().contains_key(id) || !v.is_finite() || *v < 1.0 {
                return Err(AccountingError::Invalid("invalid trophic history".into()));
            }
        }
        if self
            .food_web
            .flows
            .values()
            .chain(self.food_web.intake.values())
            .chain(self.food_web.assimilated_energy.values())
            .chain(self.food_web.digestion_energy.values())
            .chain(self.food_web.retained_energy.values())
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(AccountingError::Invalid("invalid food flow".into()));
        }
        Ok(())
    }
}
