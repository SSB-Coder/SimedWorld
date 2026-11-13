use crate::{
    ids::OrganismId,
    resources::{AccountingError, Result, finite},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, VecDeque};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};
use world_genetics::mutation::Edit;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvolutionEvent {
    pub sequence: u64,
    pub tick: u64,
    pub kind: String,
    pub actor: OrganismId,
    pub target: Option<OrganismId>,
    pub module: Option<u64>,
    pub amount: f64,
    pub edits: Vec<Edit>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Diagnostics {
    pub catabolized_nutrient: f64,
    pub catabolic_energy: f64,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub energy_history: BTreeMap<OrganismId, VecDeque<SharedEnergySample>>,
    #[serde(skip_serializing_if = "VecDeque::is_empty")]
    pub death_history: VecDeque<DeathHistory>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub behavior: BTreeMap<OrganismId, Behavior>,
    pub costs: BTreeMap<String, f64>,
    pub encounters: u64,
    pub blocked_moves: u64,
    pub failed_births: u64,
    pub birth_blocks: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub birth_blocks_by_role: BTreeMap<String, BTreeMap<String, u64>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub births_by_role: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub last_birth_tick: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub growth_blocks: BTreeMap<String, u64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub deaths_by_role: BTreeMap<String, BTreeMap<String, u64>>,
    pub attack_hits: u64,
    pub damage_nutrient: f64,
    pub repaired_nutrient: f64,
    pub accepted_mutations: u64,
    pub rejected_mutations: u64,
    pub deaths: BTreeMap<String, u64>,
    pub next_event: u64,
    pub events: Vec<EvolutionEvent>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Behavior {
    pub upkeep_per_time: f64,
    pub costs: BTreeMap<String, f64>,
    pub assimilated_energy: f64,
    pub retained_food_energy: f64,
    pub light_energy: f64,
    pub catabolized_nutrient: f64,
    pub catabolic_energy: f64,
    pub maintenance_shortfall: f64,
    pub requested_distance: f64,
    pub target: Option<OrganismId>,
    pub attacker: Option<OrganismId>,
    pub attacker_tick: u64,
    pub requested: BTreeMap<u8, f64>,
    pub executed: BTreeMap<u8, f64>,
    pub energy_before: f64,
    pub energy_after: f64,
    pub failures: BTreeMap<String, u64>,
    pub ingested: f64,
    pub damage_received: f64,
    pub damage_dealt: f64,
    pub reward: f64,
    pub distance: f64,
    pub blocked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergySample {
    pub tick: u64,
    pub tissue: f64,
    pub upkeep_per_time: f64,
    pub behavior: Behavior,
}

#[derive(Clone, Debug)]
pub struct SharedEnergySample {
    sample: Arc<EnergySample>,
    validated: Arc<OnceLock<()>>,
}

impl From<EnergySample> for SharedEnergySample {
    fn from(sample: EnergySample) -> Self {
        Self {
            sample: Arc::new(sample),
            validated: Arc::new(OnceLock::new()),
        }
    }
}

impl Deref for SharedEnergySample {
    type Target = EnergySample;
    fn deref(&self) -> &EnergySample {
        &self.sample
    }
}

impl Serialize for SharedEnergySample {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.sample.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SharedEnergySample {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(EnergySample::deserialize(deserializer)?.into())
    }
}

impl SharedEnergySample {
    fn validate(&self) -> Result<()> {
        if self.validated.get().is_none() {
            finite(self.tissue, "history tissue", false)?;
            finite(self.upkeep_per_time, "history upkeep", false)?;
            self.behavior.validate_energy()?;
            let _ = self.validated.set(());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathHistory {
    pub id: OrganismId,
    pub tick: u64,
    pub cause: crate::organism::DeathCause,
    pub contributing_signals: Vec<String>,
    pub samples: VecDeque<SharedEnergySample>,
}

impl Diagnostics {
    pub fn agent_cost(&mut self, id: OrganismId, kind: &str, amount: f64) -> Result<()> {
        self.cost(kind, amount)?;
        *self
            .behavior
            .entry(id)
            .or_default()
            .costs
            .entry(kind.into())
            .or_default() += amount;
        Ok(())
    }

    pub(crate) fn record_energy(
        &mut self,
        id: OrganismId,
        sample: EnergySample,
        limit: usize,
        cause: Option<crate::organism::DeathCause>,
    ) {
        if limit == 0 {
            return;
        }
        let history = self.energy_history.entry(id).or_default();
        history.push_back(sample.into());
        while history.len() > limit {
            history.pop_front();
        }
        if let Some(cause) = cause {
            let mut signals = Vec::new();
            if history
                .iter()
                .any(|s| s.behavior.costs.get("offspring").copied().unwrap_or(0.0) > 0.0)
            {
                signals.push("recent_reproduction".into());
            }
            if history.iter().any(|s| s.behavior.damage_received > 0.0) {
                signals.push("recent_injury".into());
            }
            if history.iter().any(|s| s.behavior.blocked) {
                signals.push("blocked_search".into());
            }
            if history
                .iter()
                .any(|s| s.behavior.catabolized_nutrient > 0.0)
            {
                signals.push("tissue_fuel_used".into());
            }
            if history
                .iter()
                .all(|s| s.behavior.retained_food_energy <= 1e-12)
            {
                signals.push("no_recent_retained_food_energy".into());
            }
            self.death_history.push_back(DeathHistory {
                id,
                tick: history.back().expect("sample").tick,
                cause,
                contributing_signals: signals,
                samples: self.energy_history.remove(&id).expect("history"),
            });
            while self.death_history.len() > 64 {
                self.death_history.pop_front();
            }
        }
    }
    pub fn birth_block(&mut self, role: &str, reason: &str) -> Result<()> {
        increment(&mut self.failed_births, 1)?;
        increment(self.birth_blocks.entry(reason.into()).or_default(), 1)?;
        increment(
            self.birth_blocks_by_role
                .entry(role.into())
                .or_default()
                .entry(reason.into())
                .or_default(),
            1,
        )
    }
    pub fn cost(&mut self, kind: &str, amount: f64) -> Result<()> {
        finite(amount, "expenditure", false)?;
        let value = self.costs.entry(kind.into()).or_default();
        *value += amount;
        finite(*value, "cumulative expenditure", false)
    }
    pub fn event(&mut self, mut event: EvolutionEvent, limit: usize) -> Result<()> {
        event.sequence = self.next_event;
        self.next_event = self
            .next_event
            .checked_add(1)
            .ok_or_else(|| AccountingError::Invalid("event counter overflow".into()))?;
        self.events.push(event);
        if self.events.len() > limit {
            self.events.drain(..self.events.len() - limit);
        }
        Ok(())
    }
    pub fn validate(&self, limit: usize) -> Result<()> {
        if self.death_history.len() > 64
            || self.energy_history.values().any(|h| h.len() > 64)
            || self.death_history.iter().any(|d| d.samples.len() > 64)
        {
            return Err(AccountingError::Invalid(
                "energy history exceeds limits".into(),
            ));
        }
        for history in self
            .energy_history
            .values()
            .chain(self.death_history.iter().map(|d| &d.samples))
        {
            let mut last = None;
            for sample in history {
                sample.validate()?;
                if last.is_some_and(|tick| tick >= sample.tick) {
                    return Err(AccountingError::Invalid(
                        "invalid energy history clock".into(),
                    ));
                }
                last = Some(sample.tick);
            }
        }
        for behavior in self.behavior.values() {
            behavior.validate_energy()?;
            if !behavior.reward.is_finite() {
                return Err(AccountingError::Invalid(
                    "non-finite learning feedback".into(),
                ));
            }
            finite(behavior.distance, "recent displacement", false)?;
            for value in behavior.requested.values() {
                if !value.is_finite() || value.abs() > 1.0 {
                    return Err(AccountingError::Invalid(
                        "invalid requested action diagnostic".into(),
                    ));
                }
            }
            for value in behavior.executed.values().copied().chain([
                behavior.ingested,
                behavior.damage_received,
                behavior.damage_dealt,
                behavior.energy_before,
                behavior.energy_after,
            ]) {
                finite(value, "action diagnostic", false)?;
            }
        }
        for v in self
            .costs
            .values()
            .copied()
            .chain([self.damage_nutrient, self.repaired_nutrient])
            .chain([self.catabolized_nutrient, self.catabolic_energy])
        {
            finite(v, "diagnostic quantity", false)?;
        }
        if self.events.len() > limit {
            return Err(AccountingError::Invalid(
                "event history exceeds limit".into(),
            ));
        }
        let mut previous = None;
        for event in &self.events {
            finite(event.amount, "event quantity", false)?;
            if event.sequence >= self.next_event
                || previous.is_some_and(|p| event.sequence != p + 1)
            {
                return Err(AccountingError::Invalid("invalid event sequence".into()));
            }
            previous = Some(event.sequence);
        }
        Ok(())
    }
}

impl Behavior {
    fn validate_energy(&self) -> Result<()> {
        for value in self.costs.values().copied().chain([
            self.assimilated_energy,
            self.retained_food_energy,
            self.light_energy,
            self.upkeep_per_time,
            self.catabolized_nutrient,
            self.catabolic_energy,
            self.maintenance_shortfall,
            self.requested_distance,
            self.energy_before,
            self.energy_after,
            self.distance,
        ]) {
            finite(value, "energy diagnostic", false)?;
        }
        Ok(())
    }
}

pub(crate) fn increment(value: &mut u64, amount: u64) -> Result<()> {
    *value = value
        .checked_add(amount)
        .ok_or_else(|| AccountingError::Invalid("diagnostic counter overflow".into()))?;
    Ok(())
}
