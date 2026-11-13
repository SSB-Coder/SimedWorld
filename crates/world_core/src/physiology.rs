use crate::{
    ids::OrganismId,
    lifecycle::{after_tissue_loss, die},
    organism::DeathCause,
    resources::{AccountingError, Resources, Result, finite},
    transactions::Draft,
};

fn rate(amount: f64, duration: f64, limit: f64) -> Result<()> {
    finite(amount, "amount", true)?;
    finite(duration, "duration", true)?;
    let capacity = limit * duration;
    finite(capacity, "process capacity", true)?;
    if amount > capacity {
        return Err(AccountingError::Invalid("process capacity exceeded".into()));
    }
    Ok(())
}

pub(crate) fn photosynthesize(
    draft: &mut Draft<'_>,
    id: OrganismId,
    light: f64,
    duration: f64,
) -> Result<()> {
    let organism = draft.living(id)?;
    let rules = draft.rules;
    if !organism.genome.photosynthetic() {
        return Err(AccountingError::Invalid(
            "no light-capture capability".into(),
        ));
    }
    rate(light, duration, rules.max_absorbed_light_per_time)?;
    if light > draft.pools.light_available {
        return Err(AccountingError::Invalid(
            "insufficient available light".into(),
        ));
    }
    draft.pools.light_available -= light;
    draft.pools.absorbed_light += light;
    let usable = light * rules.photosynthesis_efficiency;
    draft.pools.heat += light - usable;
    let reserves = organism.reserves.plus(Resources::new(0.0, usable)?)?;
    draft.set_reserves(organism, reserves)
}

pub(crate) fn absorb(
    draft: &mut Draft<'_>,
    id: OrganismId,
    nutrient: f64,
    duration: f64,
) -> Result<()> {
    let organism = draft.living(id)?;
    rate(
        nutrient,
        duration,
        draft.rules.max_absorption_nutrient_per_time,
    )?;
    if !organism.genome.photosynthetic() || nutrient > draft.pools.mineral_nutrient {
        return Err(AccountingError::Invalid(
            "uptake requires capability and available minerals".into(),
        ));
    }
    draft.pools.mineral_nutrient -= nutrient;
    let reserves = organism.reserves.plus(Resources::new(nutrient, 0.0)?)?;
    draft.set_reserves(organism, reserves)
}

pub(crate) fn feed(
    draft: &mut Draft<'_>,
    consumer_id: OrganismId,
    target_id: OrganismId,
    amount: f64,
    contact: bool,
    duration: f64,
) -> Result<()> {
    if !contact {
        return Err(AccountingError::Invalid("feeding requires contact".into()));
    }
    feed_from_module(draft, consumer_id, target_id, amount, duration, None)
}

pub(crate) fn feed_from_module(
    draft: &mut Draft<'_>,
    consumer_id: OrganismId,
    target_id: OrganismId,
    amount: f64,
    duration: f64,
    module: Option<u64>,
) -> Result<()> {
    feed_funded(
        draft,
        consumer_id,
        target_id,
        amount,
        duration,
        module,
        false,
    )
}

pub(crate) fn feed_funded(
    draft: &mut Draft<'_>,
    consumer_id: OrganismId,
    target_id: OrganismId,
    amount: f64,
    duration: f64,
    module: Option<u64>,
    food_funded: bool,
) -> Result<()> {
    let mut consumer = draft.living(consumer_id)?;
    let mut target = draft.living(target_id)?;
    let rules = draft.rules;
    if consumer_id == target_id || !consumer.genome.can_ingest() {
        return Err(AccountingError::Invalid(
            "feeding requires ingestion capability and contact with another organism".into(),
        ));
    }
    rate(amount, duration, rules.max_ingestion_nutrient_per_time)?;
    let bite = Resources::new(amount, amount * rules.tissue_energy_per_nutrient)?;
    if let Some(body) = &mut target.body {
        let removed = body.remove(amount, module)?;
        if (removed - amount).abs() > 1e-10 {
            return Err(AccountingError::Invalid(
                "bite exceeds module tissue".into(),
            ));
        }
    }
    target.tissue = target.tissue.minus(bite)?;
    let digestion = amount * rules.digestion_energy_per_ingested_nutrient;
    finite(digestion, "digestion cost", true)?;
    let assimilated = Resources::new(
        bite.nutrient * rules.nutrient_assimilation_fraction,
        bite.energy * rules.energy_assimilation_fraction,
    )?;
    if food_funded {
        consumer.reserves = consumer
            .reserves
            .plus(assimilated)?
            .minus(Resources::new(0.0, digestion)?)?;
    } else {
        consumer.reserves = consumer
            .reserves
            .minus(Resources::new(0.0, digestion)?)?
            .plus(assimilated)?;
    }
    draft.pools.heat += digestion;
    draft.pools.detritus = draft.pools.detritus.plus(bite.minus(assimilated)?)?;
    let reserves = consumer.reserves;
    draft.set_reserves(consumer, reserves)?;
    after_tissue_loss(draft, target, DeathCause::Consumption)
}

pub(crate) fn grow(
    draft: &mut Draft<'_>,
    id: OrganismId,
    amount: f64,
    duration: f64,
) -> Result<()> {
    let mut organism = draft.living(id)?;
    let rules = draft.rules;
    rate(amount, duration, rules.max_growth_nutrient_per_time)?;
    let embodied = amount * rules.tissue_energy_per_nutrient;
    let heat = amount * rules.construction_heat_per_nutrient;
    organism.reserves = organism
        .reserves
        .minus(Resources::new(amount, embodied + heat)?)?;
    organism.tissue = organism.tissue.plus(Resources::new(amount, embodied)?)?;
    if let (Some(body), Some(genome)) = (&mut organism.body, &organism.genome.body) {
        body.grow(genome, amount)?;
    }
    draft.pools.heat += heat;
    draft.put(organism);
    Ok(())
}

pub(crate) fn grow_available(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<f64> {
    let rules = draft.rules;
    let organism = draft.get(id)?;
    let cost = rules.tissue_energy_per_nutrient + rules.construction_heat_per_nutrient;
    finite(cost, "growth energy per nutrient", true)?;
    let mut amount = (rules.max_growth_nutrient_per_time * duration)
        .min(organism.reserves.nutrient)
        .min(organism.reserves.energy / cost);
    if amount * rules.tissue_energy_per_nutrient + amount * rules.construction_heat_per_nutrient
        > organism.reserves.energy
    {
        amount = amount.next_down();
    }
    if amount > 0.0 {
        grow(draft, id, amount, duration)?;
    }
    Ok(amount.max(0.0))
}

pub(crate) fn maintain(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<()> {
    let mut organism = draft.living(id)?;
    finite(duration, "duration", true)?;
    let required =
        organism.tissue.nutrient * draft.rules.maintenance_per_tissue_nutrient_per_time * duration;
    finite(required, "maintenance requirement", false)?;
    let paid = required.min(organism.reserves.energy);
    organism.reserves = organism.reserves.minus(Resources::new(0.0, paid)?)?;
    organism.age += duration;
    finite(organism.age, "age", false)?;
    draft.pools.heat += paid;
    if paid < required {
        die(draft, organism, DeathCause::Starvation)
    } else {
        draft.put(organism);
        Ok(())
    }
}

pub(crate) fn decompose(draft: &mut Draft<'_>, fraction: f64, duration: f64) -> Result<()> {
    finite(fraction, "fraction", true)?;
    finite(duration, "duration", true)?;
    if fraction > 1.0 || draft.pools.detritus == Resources::default() {
        return Err(AccountingError::Invalid(
            "decomposition requires existing detritus and fraction at most one".into(),
        ));
    }
    let pool = draft.pools.detritus;
    let consumed = if fraction == 1.0 {
        pool
    } else {
        Resources::new(pool.nutrient * fraction, pool.energy * fraction)?
    };
    let n_limit = draft.rules.max_decomposition_nutrient_per_time * duration;
    let e_limit = draft.rules.max_decomposition_energy_per_time * duration;
    finite(n_limit, "decomposition nutrient capacity", true)?;
    finite(e_limit, "decomposition energy capacity", true)?;
    if consumed.nutrient > n_limit || consumed.energy > e_limit {
        return Err(AccountingError::Invalid(
            "decomposition capacity exceeded".into(),
        ));
    }
    draft.pools.detritus = pool.minus(consumed)?;
    draft.pools.mineral_nutrient += consumed.nutrient;
    draft.pools.heat += consumed.energy;
    Ok(())
}

pub(crate) fn sense(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<()> {
    finite(duration, "duration", true)?;
    let mut organism = draft.living(id)?;
    let cost = draft.rules.sensor_energy_per_time * duration;
    finite(cost, "sensing cost", true)?;
    organism.reserves = organism.reserves.minus(Resources::new(0.0, cost)?)?;
    draft.pools.heat += cost;
    draft.put(organism);
    Ok(())
}

pub(crate) fn actuate(draft: &mut Draft<'_>, id: OrganismId, energy: f64) -> Result<()> {
    finite(energy, "energy", true)?;
    let mut organism = draft.living(id)?;
    organism.reserves = organism.reserves.minus(Resources::new(0.0, energy)?)?;
    let work = energy * draft.rules.movement_efficiency;
    draft.pools.mechanical += work;
    draft.pools.heat += energy - work;
    draft.put(organism);
    Ok(())
}

pub(crate) fn dissipate_motion(draft: &mut Draft<'_>, energy: f64) -> Result<()> {
    finite(energy, "mechanical dissipation", true)?;
    if energy > draft.pools.mechanical {
        return Err(AccountingError::Invalid(
            "insufficient mechanical energy".into(),
        ));
    }
    draft.pools.mechanical -= energy;
    draft.pools.heat += energy;
    Ok(())
}
