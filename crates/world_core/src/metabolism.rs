use crate::{
    agents,
    evolution_config::EvolutionConfig,
    ids::OrganismId,
    organism::{DeathCause, Organism},
    resources::{Resources, Result},
    rules::WorldRules,
    transactions::Draft,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upkeep {
    pub maintenance: f64,
    pub body: f64,
    pub neural: f64,
    pub sensing: f64,
}

impl Upkeep {
    pub fn total(self) -> f64 {
        self.maintenance + self.body + self.neural + self.sensing
    }
    pub fn body_total(self) -> f64 {
        self.maintenance + self.body
    }
}

pub fn upkeep(o: &Organism, rules: &WorldRules, config: &EvolutionConfig, duration: f64) -> Upkeep {
    Upkeep {
        maintenance: o.tissue.nutrient * rules.maintenance_per_tissue_nutrient_per_time * duration,
        body: (crate::evolution::body_upkeep(o, config.module_upkeep_per_time)
            + crate::food_web::adaptations(o)
                .map_or(0.0, |a| a.investments() * 0.008 * o.tissue.nutrient))
            * duration,
        neural: agents::neural_cost(o, config, duration),
        sensing: if o.brain().is_some() {
            agents::sensor_cost(o, rules.sensor_energy_per_time, duration)
        } else {
            0.0
        },
    }
}

pub fn fuel_available(
    o: &Organism,
    rules: &WorldRules,
    config: &EvolutionConfig,
    duration: f64,
) -> f64 {
    let core_limit = o
        .genome
        .body
        .as_ref()
        .zip(o.body.as_ref())
        .and_then(|(genome, body)| {
            let core = genome.modules.iter().find(|m| m.parent.is_none())?;
            let nutrient = body.modules.iter().find(|m| m.id == core.id)?.nutrient;
            Some((nutrient - 0.015).max(0.0) * o.tissue.nutrient / nutrient.max(1e-12))
        })
        .unwrap_or(o.tissue.nutrient);
    let material = (o.tissue.nutrient - rules.minimum_viable_tissue_nutrient)
        .max(0.0)
        .min(core_limit)
        .min(o.tissue.nutrient * config.catabolism_fraction_per_time * duration);
    material * rules.tissue_energy_per_nutrient * config.catabolism_efficiency
}

pub(crate) fn mobilize(
    draft: &mut Draft<'_>,
    id: OrganismId,
    config: &EvolutionConfig,
    duration: f64,
    needed: f64,
) -> Result<(f64, f64)> {
    let mut o = draft.living(id)?;
    let yield_per_nutrient = draft.rules.tissue_energy_per_nutrient * config.catabolism_efficiency;
    let mut amount = ((needed - o.reserves.energy).max(0.0) / yield_per_nutrient)
        .min(fuel_available(&o, draft.rules, config, duration) / yield_per_nutrient);
    if amount <= 0.0 {
        return Ok((0.0, 0.0));
    }
    if o.tissue.nutrient - amount < draft.rules.minimum_viable_tissue_nutrient {
        amount = amount.next_down().max(0.0);
    }
    if let Some(body) = &mut o.body {
        amount = body.remove(amount, None)?;
    }
    let material = Resources::new(amount, amount * draft.rules.tissue_energy_per_nutrient)?;
    o.tissue = o.tissue.minus(material)?;
    let fuel = material.energy * config.catabolism_efficiency;
    let reserves = o.reserves.plus(Resources::new(amount, fuel)?)?;
    draft.pools.heat += material.energy - fuel;
    draft.set_reserves(o, reserves)?;
    Ok((amount, fuel))
}

pub(crate) fn pay_upkeep(
    draft: &mut Draft<'_>,
    id: OrganismId,
    config: &EvolutionConfig,
    duration: f64,
) -> Result<(Upkeep, f64)> {
    let mut o = draft.living(id)?;
    let needed = upkeep(&o, draft.rules, config, duration);
    let total = needed.body_total();
    let paid = total.min(o.reserves.energy);
    o.reserves = o.reserves.minus(Resources::new(0.0, paid)?)?;
    o.age += duration;
    draft.pools.heat += paid;
    let shortage = (total - paid).max(0.0);
    let receipt = Upkeep {
        maintenance: if total > 0.0 {
            paid * needed.maintenance / total
        } else {
            0.0
        },
        body: if total > 0.0 {
            paid * needed.body / total
        } else {
            0.0
        },
        ..Upkeep::default()
    };
    if shortage > total * 1e-12 {
        crate::lifecycle::die(draft, o, DeathCause::Starvation)?;
    } else {
        draft.put(o);
    }
    Ok((receipt, shortage))
}
