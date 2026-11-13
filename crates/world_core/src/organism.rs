use serde::{Deserialize, Serialize};

use crate::{
    ids::OrganismId,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    Starvation,
    Consumption,
    Damage,
}

fn alive() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organism {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expressed_brain: Option<world_genetics::neural::NeuralGenome>,
    pub id: OrganismId,
    pub genome: crate::inherited::InheritedGenome,
    pub tissue: Resources,
    pub reserves: Resources,
    #[serde(default)]
    pub age: f64,
    #[serde(default = "alive")]
    pub alive: bool,
    #[serde(default)]
    pub parent_id: Option<OrganismId>,
    #[serde(default)]
    pub next_reproduction_age: f64,
    #[serde(default)]
    pub death_cause: Option<DeathCause>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<crate::anatomy::BodyState>,
}

impl Organism {
    pub fn brain(&self) -> Option<&world_genetics::neural::NeuralGenome> {
        self.expressed_brain.as_ref().or(self.genome.brain.as_ref())
    }
    pub fn reserve_capacity(&self, rules: &WorldRules) -> Result<Resources> {
        crate::anatomy::reserve_capacity(
            self.tissue.nutrient,
            self.genome.body.as_ref(),
            self.body.as_ref(),
            rules,
        )
    }
    pub fn validate(&self) -> Result<()> {
        if let Some(brain) = &self.expressed_brain {
            let body = self
                .genome
                .body
                .as_ref()
                .ok_or_else(|| AccountingError::Invalid("expressed brain lacks body".into()))?;
            brain
                .validate(
                    &body
                        .ports()
                        .map_err(|e| AccountingError::Invalid(e.into()))?,
                )
                .map_err(|e| AccountingError::Invalid(e.into()))?;
        }
        self.genome
            .validate()
            .map_err(|message| AccountingError::Invalid(message.into()))?;
        self.tissue.validate()?;
        self.reserves.validate()?;
        match (&self.genome.body, &self.body) {
            (Some(genome), Some(body)) => body.validate(genome, self.tissue.nutrient)?,
            (None, None) => {}
            _ => {
                return Err(AccountingError::Invariant(
                    "body blueprint and developed state disagree".into(),
                ));
            }
        }
        finite(self.age, "age", false)?;
        finite(self.next_reproduction_age, "next_reproduction_age", false)
    }

    pub fn mature(&self, rules: &WorldRules) -> bool {
        self.alive
            && self.age >= rules.minimum_reproduction_age
            && self.tissue.nutrient
                >= rules.maturity_tissue_nutrient.max(
                    self.genome
                        .body
                        .as_ref()
                        .map_or(0.0, |b| b.target_nutrient()),
                )
    }
}
