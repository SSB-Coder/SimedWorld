use crate::{ids::OrganismId, organism::Organism};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use world_genetics::{body::Diet, traits::Adaptations};

pub fn adaptations(o: &Organism) -> Option<&Adaptations> {
    o.genome
        .body
        .as_ref()?
        .circle
        .as_ref()?
        .adaptations
        .as_ref()
}

pub fn guild(o: &Organism) -> &'static str {
    if o.genome.photosynthetic() {
        return "producer";
    }
    match o.genome.body.as_ref().map(|b| b.diet) {
        Some(Diet::Predator) => "predator",
        Some(Diet::Omnivore) => "omnivore",
        Some(Diet::Scavenger) => "scavenger",
        Some(Diet::Recycler) => "recycler",
        _ => "grazer",
    }
}

pub fn detritus_energy_fraction(
    o: &Organism,
    rules: &crate::rules::WorldRules,
    embodied: bool,
) -> f64 {
    let investment = adaptations(o).map_or(0.5, |a| a.digestion);
    let recycler = o
        .genome
        .body
        .as_ref()
        .is_some_and(|b| b.diet == Diet::Recycler);
    rules.energy_assimilation_fraction
        * if recycler {
            if embodied {
                0.6 + 0.35 * investment
            } else {
                0.35 + 0.25 * investment
            }
        } else {
            0.5 + 0.5 * investment
        }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FoodWeb {
    pub retained_energy: BTreeMap<OrganismId, f64>,
    pub assimilated_energy: BTreeMap<OrganismId, f64>,
    pub digestion_energy: BTreeMap<OrganismId, f64>,
    pub flows: BTreeMap<String, f64>,
    pub trophic: BTreeMap<OrganismId, f64>,
    pub intake: BTreeMap<OrganismId, f64>,
}

impl FoodWeb {
    pub fn record_energy(
        &mut self,
        consumer: OrganismId,
        assimilated: f64,
        digestion: f64,
        retained: f64,
    ) {
        *self.retained_energy.entry(consumer).or_default() += retained;
        *self.assimilated_energy.entry(consumer).or_default() += assimilated;
        *self.digestion_energy.entry(consumer).or_default() += digestion;
    }
    pub fn record(
        &mut self,
        consumer: OrganismId,
        source: &str,
        destination: &str,
        level: f64,
        amount: f64,
    ) {
        *self
            .flows
            .entry(format!("{source}->{destination}"))
            .or_default() += amount;
        let total = self.intake.entry(consumer).or_default();
        let current = self.trophic.entry(consumer).or_insert(1.0);
        *current = (*current * *total + (1.0 + level) * amount) / (*total + amount);
        *total += amount;
    }
}
