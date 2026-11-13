use crate::{
    environment::{EnvironmentGrid, Position},
    ids::OrganismId,
    organism::DeathCause,
    resources::{AccountingError, Resources, Result, finite},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpse {
    pub source: OrganismId,
    pub position: Position,
    pub age: f64,
    pub density: f64,
    pub remaining: Resources,
    pub cause: DeathCause,
}

impl Corpse {
    pub(crate) fn food_position(&self, at: Position) -> Position {
        let dx = at.x - self.position.x;
        let dy = at.y - self.position.y;
        let distance = dx.hypot(dy);
        let fraction = (self.radius() / distance.max(1e-12)).min(1.0);
        Position {
            x: self.position.x + dx * fraction,
            y: self.position.y + dy * fraction,
        }
    }
    pub fn radius(&self) -> f64 {
        (self.remaining.nutrient / (12.0 * std::f64::consts::PI * self.density)).sqrt()
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.remaining.validate()?;
        finite(self.age, "corpse age", false)?;
        finite(self.density, "corpse density", true)?;
        if !self.position.x.is_finite()
            || !self.position.y.is_finite()
            || self.remaining == Resources::default()
        {
            return Err(AccountingError::Invalid("invalid corpse inventory".into()));
        }
        Ok(())
    }
}

impl EnvironmentGrid {
    pub fn corpses(&self) -> impl Iterator<Item = &Corpse> {
        self.cells.iter().flat_map(|cell| cell.corpses.values())
    }
}
