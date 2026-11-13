use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    audit::{AuditReport, audit_view, summarize},
    events::{OrganismChange, OrganismInventory, ResourceEvent},
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
};

fn one() -> f64 {
    1.0
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Photosynthesize {
        organism_id: OrganismId,
        light_energy: f64,
        #[serde(default = "one")]
        duration: f64,
    },
    Absorb {
        organism_id: OrganismId,
        nutrient: f64,
        #[serde(default = "one")]
        duration: f64,
    },
    Feed {
        consumer_id: OrganismId,
        target_id: OrganismId,
        tissue_nutrient: f64,
        contact: bool,
        #[serde(default = "one")]
        duration: f64,
    },
    Grow {
        organism_id: OrganismId,
        tissue_nutrient: f64,
        #[serde(default = "one")]
        duration: f64,
    },
    Maintain {
        organism_id: OrganismId,
        #[serde(default = "one")]
        duration: f64,
    },
    Reproduce {
        parent_id: OrganismId,
        child_id: OrganismId,
        placement_available: bool,
    },
    Damage {
        organism_id: OrganismId,
        tissue_nutrient: f64,
    },
    Decompose {
        fraction: f64,
        #[serde(default = "one")]
        duration: f64,
    },
    Sense {
        organism_id: OrganismId,
        #[serde(default = "one")]
        duration: f64,
    },
    Actuate {
        organism_id: OrganismId,
        energy: f64,
    },
    DissipateMotion {
        energy: f64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    schema_version: u32,
    operation: Operation,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pools {
    pub mineral_nutrient: f64,
    pub detritus: Resources,
    pub light_available: f64,
    pub absorbed_light: f64,
    pub heat: f64,
    pub mechanical: f64,
}

impl Pools {
    pub fn validate(self) -> Result<()> {
        self.detritus.validate()?;
        for (name, value) in [
            ("mineral nutrient", self.mineral_nutrient),
            ("available light", self.light_available),
            ("absorbed light", self.absorbed_light),
            ("heat", self.heat),
            ("mechanical energy", self.mechanical),
        ] {
            finite(value, name, false)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerInitial {
    pub schema_version: u32,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub mineral_nutrient: f64,
    #[serde(default)]
    pub light_available: f64,
    #[serde(default)]
    pub detritus: Resources,
    pub organisms: Vec<Organism>,
}

#[derive(Clone)]
pub struct ResourceLedger {
    rules: WorldRules,
    pools: Pools,
    initial_nutrient: f64,
    initial_energy: f64,
    organisms: BTreeMap<OrganismId, Organism>,
    events: Vec<ResourceEvent>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PopulationState {
    pub pools: Pools,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub organisms: Vec<Organism>,
}

pub(crate) struct Draft<'a> {
    pub rules: &'a WorldRules,
    pub pools: Pools,
    pub(crate) death_remains: BTreeMap<OrganismId, Resources>,
    pub(crate) preserve_corpses: bool,
    base: &'a BTreeMap<OrganismId, Organism>,
    edits: BTreeMap<OrganismId, Organism>,
}

impl Draft<'_> {
    pub(crate) fn get(&self, id: OrganismId) -> Result<&Organism> {
        self.edits
            .get(&id)
            .or_else(|| self.base.get(&id))
            .ok_or_else(|| AccountingError::Invalid(format!("unknown organism {}", id.0)))
    }

    pub fn living(&self, id: OrganismId) -> Result<Organism> {
        let organism = self.get(id)?;
        if !organism.alive {
            return Err(AccountingError::Invalid("organism is dead".into()));
        }
        Ok(organism.clone())
    }

    pub fn exists(&self, id: OrganismId) -> bool {
        self.edits.contains_key(&id) || self.base.contains_key(&id)
    }
    pub fn put(&mut self, organism: Organism) {
        self.edits.insert(organism.id, organism);
    }

    pub(crate) fn organisms(&self) -> impl Iterator<Item = (&OrganismId, &Organism)> {
        self.base
            .iter()
            .map(|(id, organism)| (id, self.edits.get(id).unwrap_or(organism)))
            .chain(
                self.edits
                    .iter()
                    .filter(|(id, _)| !self.base.contains_key(id)),
            )
    }

    pub fn set_reserves(&mut self, mut organism: Organism, reserves: Resources) -> Result<()> {
        reserves.validate()?;
        let capacity = organism.reserve_capacity(self.rules)?;
        let n_cap = capacity.nutrient;
        let e_cap = capacity.energy;
        finite(n_cap, "nutrient capacity", false)?;
        finite(e_cap, "energy capacity", false)?;
        self.pools.mineral_nutrient += (reserves.nutrient - n_cap).max(0.0);
        self.pools.heat += (reserves.energy - e_cap).max(0.0);
        organism.reserves =
            Resources::new(reserves.nutrient.min(n_cap), reserves.energy.min(e_cap))?;
        self.put(organism);
        Ok(())
    }
}

impl ResourceLedger {
    pub(crate) fn population_state(&self) -> PopulationState {
        PopulationState {
            pools: self.pools,
            initial_nutrient: self.initial_nutrient,
            initial_energy: self.initial_energy,
            organisms: self.organisms.values().cloned().collect(),
        }
    }

    pub(crate) fn from_population_state(rules: WorldRules, state: PopulationState) -> Result<Self> {
        rules.validate()?;
        finite(state.initial_nutrient, "initial nutrient", false)?;
        finite(state.initial_energy, "initial energy", false)?;
        let mut organisms = BTreeMap::new();
        for organism in state.organisms {
            if organisms.insert(organism.id, organism).is_some() {
                return Err(AccountingError::Invalid(
                    "duplicate checkpoint organism ID".into(),
                ));
            }
        }
        let ledger = Self {
            rules,
            pools: state.pools,
            initial_nutrient: state.initial_nutrient,
            initial_energy: state.initial_energy,
            organisms,
            events: Vec::new(),
        };
        ledger.audit()?;
        Ok(ledger)
    }

    pub fn new(rules: WorldRules, initial: LedgerInitial) -> Result<Self> {
        rules.validate()?;
        if initial.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported inventory schema".into(),
            ));
        }
        finite(initial.initial_nutrient, "initial nutrient", false)?;
        finite(initial.initial_energy, "initial energy", false)?;
        let mut organisms = BTreeMap::new();
        for organism in initial.organisms {
            if organisms.insert(organism.id, organism).is_some() {
                return Err(AccountingError::Invalid(
                    "duplicate initial organism ID".into(),
                ));
            }
        }
        let ledger = Self {
            rules,
            pools: Pools {
                mineral_nutrient: initial.mineral_nutrient,
                detritus: initial.detritus,
                light_available: initial.light_available,
                ..Pools::default()
            },
            initial_nutrient: initial.initial_nutrient,
            initial_energy: initial.initial_energy,
            organisms,
            events: Vec::new(),
        };
        ledger.audit()?;
        Ok(ledger)
    }

    pub fn from_json(rules_json: &str, initial_json: &str) -> Result<Self> {
        let initial = serde_json::from_str(initial_json)
            .map_err(|error| AccountingError::Invalid(format!("invalid inventory: {error}")))?;
        Self::new(WorldRules::from_json(rules_json)?, initial)
    }

    pub fn audit(&self) -> Result<AuditReport> {
        Ok(audit_view(
            &self.rules,
            self.pools,
            self.initial_nutrient,
            self.initial_energy,
            self.organisms.iter(),
        )?
        .audit)
    }

    pub fn audit_json(&self) -> Result<String> {
        encode(&self.audit()?)
    }
    pub fn summary_json(&self, label: &str) -> Result<String> {
        encode(&self.summary(label)?)
    }

    pub(crate) fn summary(&self, label: &str) -> Result<serde_json::Value> {
        let totals = audit_view(
            &self.rules,
            self.pools,
            self.initial_nutrient,
            self.initial_energy,
            self.organisms.iter(),
        )?;
        Ok(summarize(label, self.pools, totals))
    }

    pub(crate) fn pools(&self) -> Pools {
        self.pools
    }

    pub(crate) fn update<T>(
        &mut self,
        execute: impl FnOnce(&mut Draft<'_>) -> Result<T>,
    ) -> Result<T> {
        let mut draft = Draft {
            rules: &self.rules,
            pools: self.pools,
            base: &self.organisms,
            edits: BTreeMap::new(),
            death_remains: BTreeMap::new(),
            preserve_corpses: false,
        };
        let result = execute(&mut draft)?;
        audit_view(
            &self.rules,
            draft.pools,
            self.initial_nutrient,
            self.initial_energy,
            draft.organisms(),
        )?;
        let Draft { pools, edits, .. } = draft;
        self.organisms.extend(edits);
        self.pools = pools;
        Ok(result)
    }

    pub fn organisms(&self) -> &BTreeMap<OrganismId, Organism> {
        &self.organisms
    }
    pub fn events(&self) -> &[ResourceEvent] {
        &self.events
    }

    pub fn snapshot_json(&self) -> Result<String> {
        self.audit()?;
        encode(
            &serde_json::json!({"schema_version": 1, "rules": self.rules,
            "initial_nutrient": self.initial_nutrient, "initial_energy": self.initial_energy,
            "pools": self.pools, "organisms": self.organisms.values().collect::<Vec<_>>(), "events": self.events}),
        )
    }

    pub fn apply_json(&mut self, text: &str) -> Result<()> {
        let command: Command = serde_json::from_str(text).map_err(|error| {
            AccountingError::Invalid(format!("invalid resource command: {error}"))
        })?;
        if command.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported command schema".into(),
            ));
        }
        self.apply(command.operation)
    }

    pub fn apply(&mut self, operation: Operation) -> Result<()> {
        self.transact(&operation, |draft| {
            use crate::{lifecycle, physiology};
            match operation {
                Operation::Photosynthesize {
                    organism_id,
                    light_energy,
                    duration,
                } => physiology::photosynthesize(draft, organism_id, light_energy, duration),
                Operation::Absorb {
                    organism_id,
                    nutrient,
                    duration,
                } => physiology::absorb(draft, organism_id, nutrient, duration),
                Operation::Feed {
                    consumer_id,
                    target_id,
                    tissue_nutrient,
                    contact,
                    duration,
                } => physiology::feed(
                    draft,
                    consumer_id,
                    target_id,
                    tissue_nutrient,
                    contact,
                    duration,
                ),
                Operation::Grow {
                    organism_id,
                    tissue_nutrient,
                    duration,
                } => physiology::grow(draft, organism_id, tissue_nutrient, duration),
                Operation::Maintain {
                    organism_id,
                    duration,
                } => physiology::maintain(draft, organism_id, duration),
                Operation::Reproduce {
                    parent_id,
                    child_id,
                    placement_available,
                } => lifecycle::reproduce(draft, parent_id, child_id, placement_available),
                Operation::Damage {
                    organism_id,
                    tissue_nutrient,
                } => lifecycle::damage(draft, organism_id, tissue_nutrient),
                Operation::Decompose { fraction, duration } => {
                    physiology::decompose(draft, fraction, duration)
                }
                Operation::Sense {
                    organism_id,
                    duration,
                } => physiology::sense(draft, organism_id, duration),
                Operation::Actuate {
                    organism_id,
                    energy,
                } => physiology::actuate(draft, organism_id, energy),
                Operation::DissipateMotion { energy } => {
                    physiology::dissipate_motion(draft, energy)
                }
            }
        })
    }

    fn transact(
        &mut self,
        operation: &Operation,
        execute: impl FnOnce(&mut Draft<'_>) -> Result<()>,
    ) -> Result<()> {
        self.audit()?;
        let mut draft = Draft {
            rules: &self.rules,
            pools: self.pools,
            base: &self.organisms,
            edits: BTreeMap::new(),
            death_remains: BTreeMap::new(),
            preserve_corpses: false,
        };
        execute(&mut draft)?;
        let totals = audit_view(
            &self.rules,
            draft.pools,
            self.initial_nutrient,
            self.initial_energy,
            draft.organisms(),
        )?;
        let changes = draft
            .edits
            .iter()
            .map(|(id, organism)| OrganismChange {
                id: *id,
                before: self.organisms.get(id).map(OrganismInventory::from),
                after: OrganismInventory::from(organism),
            })
            .collect();
        let sequence = u64::try_from(self.events.len())
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| AccountingError::Invalid("event sequence overflow".into()))?;
        let event = ResourceEvent {
            sequence,
            operation: *operation,
            pools_before: self.pools,
            pools_after: draft.pools,
            organisms: changes,
            nutrient_error: totals.audit.nutrient_error,
            energy_error: totals.audit.energy_error,
        };
        let Draft { pools, edits, .. } = draft;
        self.events
            .try_reserve(1)
            .map_err(|error| AccountingError::Invalid(error.to_string()))?;
        self.organisms.extend(edits);
        self.pools = pools;
        self.events.push(event);
        Ok(())
    }
}

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|error| AccountingError::Invalid(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_commit_audit_discards_staged_pools_and_organisms() {
        let rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        let initial = LedgerInitial {
            schema_version: 1,
            initial_nutrient: 1.0,
            initial_energy: 2.0,
            mineral_nutrient: 0.0,
            light_available: 0.0,
            detritus: Resources::default(),
            organisms: vec![Organism {
                expressed_brain: None,
                id: OrganismId(1),
                genome: world_genetics::genome::Genome::default().into(),
                tissue: Resources::new(1.0, 2.0).unwrap(),
                reserves: Resources::default(),
                age: 0.0,
                alive: true,
                parent_id: None,
                next_reproduction_age: 0.0,
                death_cause: None,
                body: None,
            }],
        };
        let mut ledger = ResourceLedger::new(rules, initial).unwrap();
        let before = ledger.snapshot_json().unwrap();
        let result = ledger.transact(&Operation::DissipateMotion { energy: 1.0 }, |draft| {
            let mut organism = draft.living(OrganismId(1))?;
            organism.age = 1.0;
            draft.put(organism);
            draft.pools.heat += 1.0;
            Ok(())
        });
        assert!(matches!(result, Err(AccountingError::Invariant(_))));
        assert_eq!(ledger.snapshot_json().unwrap(), before);
    }
}
