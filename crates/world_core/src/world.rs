use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::clock::Clock;
use crate::config::{FixtureConfig, FixtureOrganism, validate_organisms};
use crate::contracts::{
    ACTION_WIDTH, ActionBatch, ContractError, ObservationBatch, Result, SCHEMA_VERSION,
    validate_action,
};
use crate::ids::OrganismId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureState {
    organism: FixtureOrganism,
    last_action: [f32; ACTION_WIDTH],
    action_total: [f64; ACTION_WIDTH],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema_version: u32,
    tick: u64,
    organisms: Vec<FixtureState>,
}

pub struct FixtureWorld {
    clock: Clock,
    organisms: Vec<FixtureState>,
    index: BTreeMap<OrganismId, usize>,
}

impl FixtureWorld {
    pub fn from_config_json(text: &str) -> Result<Self> {
        let config = FixtureConfig::from_json(text)?;
        let organisms = config
            .organisms
            .into_iter()
            .map(|organism| FixtureState {
                organism,
                last_action: [0.0; ACTION_WIDTH],
                action_total: [0.0; ACTION_WIDTH],
            })
            .collect();
        Self::restore(Snapshot {
            schema_version: SCHEMA_VERSION,
            tick: 0,
            organisms,
        })
    }

    fn restore(snapshot: Snapshot) -> Result<Self> {
        if snapshot.schema_version != SCHEMA_VERSION {
            return Err(ContractError("unsupported snapshot schema".into()));
        }
        validate_organisms(
            &snapshot
                .organisms
                .iter()
                .map(|state| state.organism.clone())
                .collect::<Vec<_>>(),
        )?;
        let mut index = BTreeMap::new();
        for (row, state) in snapshot.organisms.iter().enumerate() {
            validate_action(&state.last_action)?;
            if !state.action_total.iter().all(|value| value.is_finite()) {
                return Err(ContractError("non-finite accumulated action".into()));
            }
            index.insert(state.organism.id, row);
        }
        Ok(Self {
            clock: Clock(snapshot.tick),
            organisms: snapshot.organisms,
            index,
        })
    }

    pub fn tick(&self) -> u64 {
        self.clock.0
    }

    pub fn observe(&self) -> ObservationBatch {
        ObservationBatch {
            schema_version: SCHEMA_VERSION,
            tick: self.clock.0,
            ids: self
                .organisms
                .iter()
                .map(|state| state.organism.id)
                .collect(),
            topology_versions: self
                .organisms
                .iter()
                .map(|state| state.organism.topology_version)
                .collect(),
            values: self
                .organisms
                .iter()
                .map(|state| state.organism.signals)
                .collect(),
        }
    }

    pub fn apply_actions(&mut self, batch: ActionBatch) -> Result<()> {
        if batch.schema_version != SCHEMA_VERSION || batch.tick != self.clock.0 {
            return Err(ContractError(
                "unsupported action schema or stale tick".into(),
            ));
        }
        let count = self.organisms.len();
        if batch.ids.len() != count
            || batch.topology_versions.len() != count
            || batch.values.len() != count
        {
            return Err(ContractError("one action per organism is required".into()));
        }
        let next_clock = self.clock.next()?;
        let mut seen = BTreeSet::new();
        let mut updates = Vec::with_capacity(count);
        for ((id, version), action) in batch
            .ids
            .iter()
            .zip(&batch.topology_versions)
            .zip(&batch.values)
        {
            if !seen.insert(*id) {
                return Err(ContractError("duplicate action ID".into()));
            }
            let row = *self
                .index
                .get(id)
                .ok_or_else(|| ContractError("unknown action ID".into()))?;
            let current = &self.organisms[row];
            if *version != current.organism.topology_version {
                return Err(ContractError("stale controller version".into()));
            }
            validate_action(action)?;
            let total = std::array::from_fn(|column| {
                current.action_total[column] + f64::from(action[column])
            });
            if !total.iter().all(|value| value.is_finite()) {
                return Err(ContractError("accumulated action overflow".into()));
            }
            updates.push((row, *action, total));
        }
        // All fallible work finishes before any world state changes.
        for (row, action, total) in updates {
            self.organisms[row].last_action = action;
            self.organisms[row].action_total = total;
        }
        self.clock = next_clock;
        Ok(())
    }

    pub fn snapshot_json(&self) -> Result<String> {
        serde_json::to_string(&Snapshot {
            schema_version: SCHEMA_VERSION,
            tick: self.clock.0,
            organisms: self.organisms.clone(),
        })
        .map_err(|error| ContractError(error.to_string()))
    }

    pub fn from_snapshot_json(text: &str) -> Result<Self> {
        let snapshot = serde_json::from_str(text)
            .map_err(|error| ContractError(format!("invalid fixture snapshot: {error}")))?;
        Self::restore(snapshot)
    }
}
