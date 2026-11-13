use world_genetics::inheritance::inherit;

use crate::{
    ids::OrganismId,
    organism::{DeathCause, Organism},
    resources::{AccountingError, Resources, Result, finite},
    transactions::Draft,
};

pub(crate) fn die(draft: &mut Draft<'_>, mut organism: Organism, cause: DeathCause) -> Result<()> {
    draft
        .death_remains
        .insert(organism.id, organism.tissue.plus(organism.reserves)?);
    if !draft.preserve_corpses {
        draft.pools.detritus = draft
            .pools
            .detritus
            .plus(organism.tissue)?
            .plus(organism.reserves)?;
    }
    organism.tissue = Resources::default();
    if let Some(body) = &mut organism.body {
        for module in &mut body.modules {
            module.nutrient = 0.0;
        }
    }
    organism.reserves = Resources::default();
    organism.alive = false;
    organism.death_cause = Some(cause);
    draft.put(organism);
    Ok(())
}

pub(crate) fn after_tissue_loss(
    draft: &mut Draft<'_>,
    organism: Organism,
    cause: DeathCause,
) -> Result<()> {
    let core_failed = organism
        .genome
        .body
        .as_ref()
        .zip(organism.body.as_ref())
        .is_some_and(|(g, b)| {
            g.modules
                .iter()
                .find(|m| m.parent.is_none())
                .is_none_or(|m| {
                    b.modules
                        .iter()
                        .find(|s| s.id == m.id)
                        .is_none_or(|s| s.nutrient < 0.015)
                })
        });
    if organism.tissue.nutrient < draft.rules.minimum_viable_tissue_nutrient || core_failed {
        die(draft, organism, cause)
    } else {
        let reserves = organism.reserves;
        draft.set_reserves(organism, reserves)
    }
}

pub(crate) fn damage(draft: &mut Draft<'_>, id: OrganismId, amount: f64) -> Result<()> {
    finite(amount, "damaged tissue", true)?;
    let mut organism = draft.living(id)?;
    let lost = Resources::new(amount, amount * draft.rules.tissue_energy_per_nutrient)?;
    if let Some(body) = &mut organism.body {
        body.remove(amount, None)?;
    }
    organism.tissue = organism.tissue.minus(lost)?;
    draft.pools.detritus = draft.pools.detritus.plus(lost)?;
    after_tissue_loss(draft, organism, DeathCause::Damage)
}

pub(crate) fn reproduce(
    draft: &mut Draft<'_>,
    parent_id: OrganismId,
    child_id: OrganismId,
    placement: bool,
) -> Result<()> {
    let genome = inherit(&draft.living(parent_id)?.genome);
    reproduce_blueprint(
        draft,
        parent_id,
        child_id,
        placement,
        genome,
        draft.rules.offspring_tissue_nutrient,
    )
}

pub(crate) fn reproduce_blueprint(
    draft: &mut Draft<'_>,
    parent_id: OrganismId,
    child_id: OrganismId,
    placement: bool,
    genome: world_genetics::genome::Genome,
    tissue_nutrient: f64,
) -> Result<()> {
    let reserves = Resources::new(
        draft.rules.offspring_reserve_nutrient,
        draft.rules.offspring_reserve_energy,
    )?;
    reproduce_funded(
        draft,
        parent_id,
        child_id,
        placement,
        genome,
        tissue_nutrient,
        reserves,
    )
}

pub(crate) fn reproduce_funded(
    draft: &mut Draft<'_>,
    parent_id: OrganismId,
    child_id: OrganismId,
    placement: bool,
    genome: world_genetics::genome::Genome,
    tissue_nutrient: f64,
    reserves: Resources,
) -> Result<()> {
    genome
        .validate()
        .map_err(|e| AccountingError::Invalid(e.into()))?;
    let mut parent = draft.living(parent_id)?;
    let rules = draft.rules;
    if draft.exists(child_id)
        || !parent.mature(rules)
        || parent.age < parent.next_reproduction_age
        || !placement
    {
        return Err(AccountingError::Invalid(
            "birth requires a new ID, maturity, completed cooldown, and placement".into(),
        ));
    }
    let tissue = Resources::new(
        tissue_nutrient,
        tissue_nutrient * rules.tissue_energy_per_nutrient,
    )?;
    let heat = tissue_nutrient * rules.construction_heat_per_nutrient;
    let investment = tissue.plus(reserves)?.plus(Resources::new(0.0, heat)?)?;
    parent.reserves = parent.reserves.minus(investment)?;
    parent.next_reproduction_age = parent.age + rules.reproduction_cooldown;
    finite(
        parent.next_reproduction_age,
        "reproduction cooldown age",
        false,
    )?;
    let child = Organism {
        expressed_brain: None,
        id: child_id,
        body: genome
            .body
            .as_ref()
            .map(|b| crate::anatomy::BodyState::new(b, tissue_nutrient))
            .transpose()?,
        genome: genome.into(),
        tissue,
        reserves,
        age: 0.0,
        alive: true,
        parent_id: Some(parent_id),
        next_reproduction_age: 0.0,
        death_cause: None,
    };
    draft.put(parent);
    draft.put(child);
    draft.pools.heat += heat;
    Ok(())
}

pub(crate) fn birth_investment(rules: &crate::rules::WorldRules) -> Result<Resources> {
    birth_investment_for(rules, rules.offspring_tissue_nutrient)
}

pub(crate) fn birth_investment_for(
    rules: &crate::rules::WorldRules,
    tissue: f64,
) -> Result<Resources> {
    Resources::new(
        tissue + rules.offspring_reserve_nutrient,
        tissue * rules.tissue_energy_per_nutrient
            + rules.offspring_reserve_energy
            + tissue * rules.construction_heat_per_nutrient,
    )
}
