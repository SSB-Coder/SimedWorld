use serde::Serialize;

use crate::{
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    transactions::Pools,
};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AuditReport {
    pub nutrient: f64,
    pub energy: f64,
    pub nutrient_error: f64,
    pub energy_error: f64,
}

pub(crate) struct Totals {
    pub audit: AuditReport,
    living_nutrient: f64,
    living_energy: f64,
    living_count: usize,
}

#[derive(Default)]
pub(crate) struct Sum {
    total: f64,
    correction: f64,
}

impl Sum {
    pub(crate) fn add(&mut self, value: f64) {
        let updated = self.total + value;
        self.correction += if self.total.abs() >= value.abs() {
            (self.total - updated) + value
        } else {
            (value - updated) + self.total
        };
        self.total = updated;
    }
    pub(crate) fn value(&self) -> f64 {
        self.total + self.correction
    }
}

pub(crate) fn audit_view<'a>(
    rules: &WorldRules,
    pools: Pools,
    initial_nutrient: f64,
    initial_energy: f64,
    organisms: impl Iterator<Item = (&'a OrganismId, &'a Organism)>,
) -> Result<Totals> {
    pools.validate()?;
    let mut n_live = Sum::default();
    let mut e_live = Sum::default();
    let mut living_count = 0;
    for (id, organism) in organisms {
        organism.validate()?;
        if *id != organism.id
            || !rules.close(
                organism.tissue.energy,
                organism.tissue.nutrient * rules.tissue_energy_per_nutrient,
            )
        {
            return Err(AccountingError::Invariant(
                "organism identity or tissue energy mismatch".into(),
            ));
        }
        let capacity = organism.reserve_capacity(rules)?;
        let n_cap = capacity.nutrient;
        let e_cap = capacity.energy;
        finite(n_cap, "nutrient capacity", false)?;
        finite(e_cap, "energy capacity", false)?;
        if organism.alive {
            if organism.tissue.nutrient < rules.minimum_viable_tissue_nutrient
                || organism.death_cause.is_some()
                || organism.reserves.nutrient > n_cap
                || organism.reserves.energy > e_cap
            {
                return Err(AccountingError::Invariant(
                    "invalid living tissue, death state, or reserves".into(),
                ));
            }
            living_count += 1;
        } else if organism.tissue != Resources::default()
            || organism.reserves != Resources::default()
            || organism.death_cause.is_none()
        {
            return Err(AccountingError::Invariant(
                "dead organism retains resources or lacks a cause".into(),
            ));
        }
        n_live.add(organism.tissue.nutrient + organism.reserves.nutrient);
        e_live.add(organism.tissue.energy + organism.reserves.energy);
    }
    let mut n = Sum::default();
    for value in [
        pools.mineral_nutrient,
        pools.detritus.nutrient,
        n_live.value(),
    ] {
        n.add(value);
    }
    let mut e = Sum::default();
    for value in [
        pools.detritus.energy,
        e_live.value(),
        pools.heat,
        pools.mechanical,
    ] {
        e.add(value);
    }
    let expected = initial_energy + pools.absorbed_light;
    if !rules.close(n.value(), initial_nutrient) || !rules.close(e.value(), expected) {
        return Err(AccountingError::Invariant(format!(
            "unbalanced inventory: nutrient {}/{initial_nutrient}; energy {}/{expected}",
            n.value(),
            e.value()
        )));
    }
    Ok(Totals {
        audit: AuditReport {
            nutrient: n.value(),
            energy: e.value(),
            nutrient_error: n.value() - initial_nutrient,
            energy_error: e.value() - expected,
        },
        living_nutrient: n_live.value(),
        living_energy: e_live.value(),
        living_count,
    })
}

pub(crate) fn summarize(label: &str, pools: Pools, totals: Totals) -> serde_json::Value {
    serde_json::json!({"label": label, "mineral_nutrient": pools.mineral_nutrient,
        "living_nutrient": totals.living_nutrient, "living_energy": totals.living_energy,
        "detritus_nutrient": pools.detritus.nutrient, "detritus_energy": pools.detritus.energy,
        "heat": pools.heat, "mechanical_energy": pools.mechanical, "absorbed_light": pools.absorbed_light,
        "living_count": totals.living_count, "total_nutrient": totals.audit.nutrient, "accounted_energy": totals.audit.energy})
}
