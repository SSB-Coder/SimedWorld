use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use world_genetics::genome::Genome;

use crate::{
    audit::{AuditReport, Sum},
    ecology::{EcologyReport, EcologyState, EcologyWindow, initial_heading, reserve_for_birth},
    environment::{EnvironmentGrid, Position},
    habitat_config::{HabitatConfig, seeded_position},
    ids::OrganismId,
    organism::Organism,
    physiology,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    scheduler::Scheduler,
    transactions::{Draft, LedgerInitial, Pools, ResourceLedger},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WindowReport {
    pub(crate) tick: u64,
    pub(crate) duration: f64,
    pub(crate) producers_processed: usize,
    pub(crate) deaths: usize,
    pub(crate) light_captured: f64,
    pub(crate) nutrient_absorbed: f64,
    pub(crate) tissue_grown: f64,
    pub(crate) ecology: EcologyReport,
}

impl WindowReport {
    pub(crate) fn validate(&self, scheduler: Scheduler, rules: &WorldRules) -> Result<()> {
        if self.tick != scheduler.last_ecology_tick()
            || !rules.close(
                self.duration,
                rules.physics_dt * rules.ecology_interval_ticks as f64,
            )
        {
            return Err(AccountingError::Invariant(
                "window receipt does not match the clock".into(),
            ));
        }
        for value in [
            self.duration,
            self.light_captured,
            self.nutrient_absorbed,
            self.tissue_grown,
        ] {
            finite(value, "window receipt", false)?;
        }
        self.ecology.validate()
    }
}

#[derive(Clone)]
pub struct Habitat {
    pub(crate) audit_cache: OnceLock<(u64, AuditReport)>,
    pub(crate) rules: WorldRules,
    pub(crate) config: HabitatConfig,
    pub(crate) grid: EnvironmentGrid,
    pub(crate) positions: BTreeMap<OrganismId, Position>,
    pub(crate) ledger: ResourceLedger,
    pub(crate) scheduler: Scheduler,
    pub(crate) incident_light: f64,
    pub(crate) escaped_light: f64,
    pub(crate) last_update: Option<WindowReport>,
    pub(crate) ecology: EcologyState,
    pub(crate) diagnostics: Option<crate::diagnostics::Diagnostics>,
}

impl Habitat {
    pub fn from_json(rules_json: &str, config_json: &str) -> Result<Self> {
        let rules = WorldRules::from_json(rules_json)?;
        let config = HabitatConfig::from_json(config_json)?;
        Self::new(rules, config)
    }

    pub fn new(rules: WorldRules, config: HabitatConfig) -> Result<Self> {
        rules.validate()?;
        config.validate(&rules)?;
        let grid = EnvironmentGrid::new(
            &rules,
            config.mineral_nutrient_per_cell,
            config.light_energy_per_cell_per_time,
            config.light_gradient,
            &config.cell_overrides,
        )?;
        let totals = grid.totals()?;
        let duration = rules.physics_dt * rules.ecology_interval_ticks as f64;
        finite(duration, "ecology duration", true)?;
        finite(totals.power * duration, "scheduled light input", false)?;
        let populations = std::iter::once((&config.producers, true))
            .chain(config.consumers.as_ref().map(|p| (p, false)))
            .chain(config.populations.iter().map(|p| (p, false)));
        let mut expected = Resources::new(totals.mineral, 0.0)?.plus(totals.detritus)?;
        let mut count = 0usize;
        for (founders, _) in populations.clone() {
            let per_founder = Resources::new(
                founders.tissue_nutrient + founders.reserve_nutrient,
                founders.tissue_nutrient * rules.tissue_energy_per_nutrient
                    + founders.reserve_energy,
            )?;
            expected = expected.plus(Resources::new(
                per_founder.nutrient * founders.count as f64,
                per_founder.energy * founders.count as f64,
            )?)?;
            count = count
                .checked_add(founders.count)
                .ok_or_else(|| AccountingError::Invalid("founder count overflow".into()))?;
        }
        if !rules.close(expected.nutrient, config.initial_nutrient)
            || !rules.close(expected.energy, config.initial_energy)
        {
            return Err(AccountingError::Invariant(
                "declared habitat budgets do not fund the grid and founders".into(),
            ));
        }
        let mut organisms = Vec::new();
        organisms
            .try_reserve_exact(count)
            .map_err(|error| AccountingError::Invalid(error.to_string()))?;
        let mut positions = BTreeMap::new();
        for (founders, producer) in populations {
            let tissue = Resources::new(
                founders.tissue_nutrient,
                founders.tissue_nutrient * rules.tissue_energy_per_nutrient,
            )?;
            let reserves = Resources::new(founders.reserve_nutrient, founders.reserve_energy)?;
            for ordinal in 0..founders.count {
                let id = OrganismId(
                    u64::try_from(organisms.len())
                        .ok()
                        .and_then(|n| n.checked_add(1))
                        .ok_or_else(|| {
                            AccountingError::Invalid("founder identity overflow".into())
                        })?,
                );
                let position = match &founders.positions {
                    Some(values) => values[ordinal],
                    None => seeded_position(
                        config.seed,
                        organisms.len() as u64,
                        rules.world_width,
                        rules.world_height,
                    ),
                };
                grid.cell_index(position)?;
                let genome = founders.genome.clone().unwrap_or(
                    Genome::new(producer, !producer, vec![])
                        .map_err(|e| AccountingError::Invalid(e.into()))?,
                );
                let body = genome
                    .body
                    .as_ref()
                    .map(|g| crate::anatomy::BodyState::new(g, tissue.nutrient))
                    .transpose()?;
                let organism = Organism {
                    expressed_brain: None,
                    id,
                    genome: genome.into(),
                    body,
                    tissue,
                    reserves,
                    age: config.evolution.as_ref().map_or(0.0, |e| {
                        e.founder_age_spread
                            * seeded_position(config.seed.wrapping_add(id.0), id.0, 1.0, 1.0).x
                    }),
                    alive: true,
                    parent_id: None,
                    next_reproduction_age: 0.0,
                    death_cause: None,
                };
                let position = if config.evolution.is_some() {
                    crate::evolution::founder_position(
                        &grid,
                        &organisms,
                        &positions,
                        &organism,
                        config.seed,
                        founders.positions.as_ref().map(|p| p[ordinal]),
                    )?
                } else {
                    position
                };
                let mut position = position;
                if let Some(evolution) = &config.evolution {
                    let radius = crate::agents::circles(
                        &organism,
                        position,
                        initial_heading(config.seed, id),
                    )?[0]
                        .radius;
                    for attempt in 0..4096 {
                        if !crate::development::rock_collision(&evolution.terrain, position, radius)
                        {
                            break;
                        }
                        position = crate::evolution::founder_position(
                            &grid,
                            &organisms,
                            &positions,
                            &organism,
                            config.seed.wrapping_add(attempt + 1),
                            None,
                        )?;
                    }
                    if crate::development::rock_collision(&evolution.terrain, position, radius) {
                        return Err(AccountingError::Invalid(
                            "no terrain-clear founder placement".into(),
                        ));
                    }
                }
                positions.insert(id, position);
                organisms.push(organism);
            }
        }
        let mut ecology = EcologyState::new(
            u64::try_from(count)
                .ok()
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| AccountingError::Invalid("founder identity overflow".into()))?,
        );
        for organism in &organisms {
            if organism.genome.can_ingest() || organism.body.is_some() {
                ecology
                    .headings
                    .insert(organism.id, initial_heading(config.seed, organism.id));
            }
        }
        let ledger = ResourceLedger::new(
            rules.clone(),
            LedgerInitial {
                schema_version: 1,
                initial_nutrient: config.initial_nutrient,
                initial_energy: config.initial_energy,
                mineral_nutrient: totals.mineral,
                light_available: 0.0,
                detritus: totals.detritus,
                organisms,
            },
        )?;
        let diagnostics = config
            .evolution
            .as_ref()
            .map(|_| crate::diagnostics::Diagnostics::default());
        let habitat = Self {
            audit_cache: OnceLock::new(),
            rules,
            config,
            grid,
            positions,
            ledger,
            scheduler: Scheduler::default(),
            incident_light: 0.0,
            escaped_light: 0.0,
            last_update: None,
            ecology,
            diagnostics,
        };
        habitat.audit()?;
        Ok(habitat)
    }

    pub fn tick(&self) -> u64 {
        self.scheduler.tick()
    }

    pub fn rules(&self) -> &WorldRules {
        &self.rules
    }

    pub fn grid(&self) -> &EnvironmentGrid {
        &self.grid
    }

    pub fn organisms(&self) -> &BTreeMap<OrganismId, Organism> {
        self.ledger.organisms()
    }

    pub fn positions(&self) -> &BTreeMap<OrganismId, Position> {
        &self.positions
    }

    pub fn metrics(&self) -> &EcologyReport {
        &self.ecology.totals
    }
    pub fn heading(&self, id: OrganismId) -> Option<f64> {
        self.ecology.headings.get(&id).copied()
    }

    pub fn advance(&mut self) -> Result<()> {
        if self.config.evolution.is_some() {
            return Err(AccountingError::Invalid(
                "use the biological controller runtime to advance this habitat".into(),
            ));
        }
        self.advance_impl(None)
    }

    pub(crate) fn advance_impl(
        &mut self,
        controls: Option<&crate::agents::Controls>,
    ) -> Result<()> {
        let (scheduler, window) = self.scheduler.next(&self.rules)?;
        if let Some(duration) = window {
            self.audit()?;
            let mut grid = self.grid.clone();
            let mut positions = self.positions.clone();
            let mut ecology = self.ecology.clone();
            let mut diagnostics = self.diagnostics.clone();
            let initial_alive = self.ledger.organisms().values().filter(|o| o.alive).count();
            let escaped = self.escaped_light + grid.totals()?.light;
            finite(escaped, "escaped light", false)?;
            for cell in &mut grid.cells {
                cell.light_available = cell.light_energy_per_time * duration;
            }
            let incident = self.incident_light + grid.totals()?.light;
            finite(incident, "incident light", false)?;
            let mut cohorts = vec![Vec::new(); grid.cells.len()];
            for organism in self
                .ledger
                .organisms()
                .values()
                .filter(|organism| organism.alive && organism.genome.photosynthetic())
            {
                cohorts[grid.cell_index(self.positions[&organism.id])?].push(organism.id);
            }
            let rules = &self.rules;
            let config = &self.config;
            let report = self.ledger.update(|draft| {
                let mut report = WindowReport {
                    tick: scheduler.tick(),
                    duration,
                    ..WindowReport::default()
                };
                let mut eco_report = EcologyReport::default();
                let window = EcologyWindow {
                    config: &config.ecology,
                    duration,
                    seed: config.seed,
                    tick: scheduler.tick(),
                };
                ecology.recycle(draft, &mut grid, &config.ecology, duration, &mut eco_report)?;
                grid.diffuse_minerals(config.ecology.mineral_diffusion_per_time, duration)?;
                if config.evolution.is_some() {
                    let controls = controls.ok_or_else(|| {
                        AccountingError::Invalid(
                            "neural actions are required at this boundary".into(),
                        )
                    })?;
                    let mut extended = crate::evolution::PopulationWindow {
                        blocked_controls: Default::default(),
                        spatial: None,
                        largest_extent: 0.0,
                        grid: &mut grid,
                        positions: &mut positions,
                        ecology: &mut ecology,
                        diagnostics: diagnostics.as_mut().ok_or_else(|| {
                            AccountingError::Invariant("diagnostics missing".into())
                        })?,
                        config,
                        tick: scheduler.tick(),
                        duration,
                        controls,
                    }
                    .run(draft)?;
                    extended.ecology.decomposed_nutrient = eco_report.decomposed_nutrient;
                    extended.ecology.decomposed_energy = eco_report.decomposed_energy;
                    ecology.totals.accumulate(&extended.ecology)?;
                    grid.write_totals(&mut draft.pools)?;
                    light_balance(rules, incident, escaped, draft.pools)?;
                    return Ok(extended);
                }

                for (cell, cohort) in grid.cells.iter_mut().zip(cohorts) {
                    if cohort.is_empty() {
                        continue;
                    }
                    let light_share = cell.light_available / cohort.len() as f64;
                    let nutrient_share = cell.mineral_nutrient / cohort.len() as f64;
                    // Physiology sees one cell; captured light, heat, and work remain global.
                    cell.load_pools(&mut draft.pools);
                    for id in cohort {
                        process_producer(
                            draft,
                            id,
                            light_share,
                            nutrient_share,
                            duration,
                            &config.ecology,
                            &mut report,
                        )?;
                    }
                    cell.store_pools(draft.pools);
                }
                ecology.consumers(draft, &mut grid, &mut positions, window, &mut eco_report)?;
                ecology.births(draft, &grid, &mut positions, window, &mut eco_report)?;
                let final_alive = draft.organisms().filter(|(_, o)| o.alive).count();
                eco_report.deaths = (initial_alive as u64)
                    .checked_add(eco_report.births)
                    .and_then(|n| n.checked_sub(final_alive as u64))
                    .ok_or_else(|| {
                        AccountingError::Invariant("invalid population accounting".into())
                    })?;
                report.deaths = usize::try_from(eco_report.deaths)
                    .map_err(|_| AccountingError::Invalid("death count overflow".into()))?;
                report.tissue_grown += eco_report.tissue_grown;
                eco_report.tissue_grown = report.tissue_grown;
                ecology.totals.accumulate(&eco_report)?;
                report.ecology = eco_report;
                grid.write_totals(&mut draft.pools)?;
                light_balance(rules, incident, escaped, draft.pools)?;
                for value in [
                    report.light_captured,
                    report.nutrient_absorbed,
                    report.tissue_grown,
                ] {
                    finite(value, "window receipt", false)?;
                }
                Ok(report)
            })?;
            self.grid = grid;
            self.positions = positions;
            self.ecology = ecology;
            self.diagnostics = diagnostics;
            self.incident_light = incident;
            self.escaped_light = escaped;
            self.last_update = Some(report);
            self.audit_cache.take();
        }
        self.scheduler = scheduler;
        Ok(())
    }

    pub fn audit(&self) -> Result<AuditReport> {
        self.scheduler.validate(&self.rules)?;
        let tick = self.scheduler.last_ecology_tick();
        if let Some((validated_tick, report)) = self.audit_cache.get()
            && *validated_tick == tick
        {
            return Ok(*report);
        }
        let report = self.audit_uncached()?;
        let _ = self.audit_cache.set((tick, report));
        Ok(report)
    }

    fn audit_uncached(&self) -> Result<AuditReport> {
        let report = self.ledger.audit()?;
        self.scheduler.validate(&self.rules)?;
        self.grid.validate(&self.rules)?;
        let totals = self.grid.totals()?;
        let pools = self.ledger.pools();
        let mut corpse_ids = std::collections::BTreeSet::new();
        for (cell_index, cell) in self.grid.cells.iter().enumerate() {
            for (id, corpse) in &cell.corpses {
                if *id != corpse.source
                    || !corpse_ids.insert(*id)
                    || self.grid.cell_index(corpse.position)? != cell_index
                    || self
                        .positions
                        .get(id)
                        .is_none_or(|p| p.x != corpse.position.x || p.y != corpse.position.y)
                    || self
                        .organisms()
                        .get(id)
                        .is_none_or(|o| o.alive || o.death_cause != Some(corpse.cause))
                {
                    return Err(AccountingError::Invariant(
                        "corpse does not match dead organism".into(),
                    ));
                }
            }
        }
        for (a, b) in [
            (totals.mineral, pools.mineral_nutrient),
            (totals.detritus.nutrient, pools.detritus.nutrient),
            (totals.detritus.energy, pools.detritus.energy),
            (totals.light, pools.light_available),
        ] {
            if !self.rules.close(a, b) {
                return Err(AccountingError::Invariant(
                    "grid and ledger aggregates disagree".into(),
                ));
            }
        }
        if self.positions.len() != self.ledger.organisms().len() {
            return Err(AccountingError::Invariant(
                "organism positions do not match inventory".into(),
            ));
        }
        for id in self.ledger.organisms().keys() {
            let position = self
                .positions
                .get(id)
                .ok_or_else(|| AccountingError::Invariant("organism position is missing".into()))?;
            self.grid.cell_index(*position)?;
        }
        self.ecology.totals.validate()?;
        self.ecology.interactions.validate(self)?;
        match (&self.config.evolution, &self.diagnostics) {
            (Some(config), Some(diagnostics)) => {
                diagnostics.validate(config.history_limit)?;
                for organism in self.organisms().values() {
                    if organism
                        .genome
                        .body
                        .as_ref()
                        .is_none_or(|b| b.modules.len() > config.mutation.max_modules)
                        || organism.brain().is_some_and(|b| {
                            b.nodes.len() > config.mutation.max_nodes
                                || b.connections.len() > config.mutation.max_connections
                        })
                    {
                        return Err(AccountingError::Invariant(
                            "expressed blueprint exceeds configured limits".into(),
                        ));
                    }
                }
            }
            (None, None) => {}
            _ => {
                return Err(AccountingError::Invariant(
                    "diagnostics/configuration mismatch".into(),
                ));
            }
        }

        let consumers: Vec<_> = self
            .ledger
            .organisms()
            .values()
            .filter(|o| o.genome.can_ingest() || o.body.is_some())
            .map(|o| o.id)
            .collect();
        if self.ecology.headings.len() != consumers.len() {
            return Err(AccountingError::Invariant(
                "consumer motion registry does not match population".into(),
            ));
        }
        for id in consumers {
            let heading =
                self.ecology.headings.get(&id).ok_or_else(|| {
                    AccountingError::Invariant("consumer heading is missing".into())
                })?;
            if !heading.is_finite() || heading.abs() > std::f64::consts::PI {
                return Err(AccountingError::Invalid("invalid consumer heading".into()));
            }
        }
        if self
            .ledger
            .organisms()
            .keys()
            .any(|id| id.0 >= self.ecology.next_id)
        {
            return Err(AccountingError::Invariant(
                "next offspring ID is not unused".into(),
            ));
        }
        for organism in self.ledger.organisms().values() {
            let cooldown = if organism.genome.photosynthetic() {
                self.rules
                    .reproduction_cooldown
                    .max(self.config.ecology.producer_reproduction_cooldown)
            } else {
                self.rules.reproduction_cooldown
            };
            let initial_age = if organism.parent_id.is_none() {
                self.config.evolution.as_ref().map_or(0.0, |e| {
                    e.founder_age_spread
                        * seeded_position(
                            self.config.seed.wrapping_add(organism.id.0),
                            organism.id.0,
                            1.0,
                            1.0,
                        )
                        .x
                })
            } else {
                0.0
            };
            let completed_time =
                self.scheduler.last_ecology_tick() as f64 * self.rules.physics_dt + initial_age;
            if (organism.age > completed_time && !self.rules.close(organism.age, completed_time))
                || (organism.next_reproduction_age > organism.age + cooldown
                    && !self
                        .rules
                        .close(organism.next_reproduction_age, organism.age + cooldown))
            {
                return Err(AccountingError::Invariant(
                    "organism clock exceeds habitat history".into(),
                ));
            }
            if let Some(parent) = organism.parent_id
                && (parent.0 >= organism.id.0 || !self.ledger.organisms().contains_key(&parent))
            {
                return Err(AccountingError::Invariant("invalid ancestry link".into()));
            }
        }
        if self.ledger.organisms().values().filter(|o| o.alive).count()
            > self.config.ecology.max_population
        {
            return Err(AccountingError::Invariant(
                "live population exceeds configured limit".into(),
            ));
        }
        if self.config.evolution.is_some() {
            crate::evolution::audit_geometry(self)?;
        }
        light_balance(&self.rules, self.incident_light, self.escaped_light, pools)?;
        Ok(report)
    }

    pub fn audit_json(&self) -> Result<String> {
        let report = self.audit()?;
        encode(
            &serde_json::json!({"nutrient":report.nutrient,"energy":report.energy,
            "nutrient_error":report.nutrient_error,"energy_error":report.energy_error,
            "light_error":light_balance(&self.rules,self.incident_light,self.escaped_light,self.ledger.pools())?}),
        )
    }

    pub fn summary_json(&self) -> Result<String> {
        self.summary_impl(false)
    }

    pub fn summary_compact_json(&self) -> Result<String> {
        self.summary_impl(true)
    }

    fn summary_impl(&self, compact: bool) -> Result<String> {
        self.audit()?;
        let mut diagnostics = std::borrow::Cow::Borrowed(&self.diagnostics);
        if compact && let Some(diagnostic) = diagnostics.to_mut() {
            diagnostic.events.clear();
            diagnostic.behavior.clear();
            diagnostic.energy_history.clear();
            diagnostic.death_history.clear();
        }
        let mut value = self.ledger.summary("habitat")?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| AccountingError::Invariant("summary is not an object".into()))?;
        fields.extend(serde_json::json!({"schema_version":1,"tick":self.tick(),
            "elapsed_time":self.tick() as f64*self.rules.physics_dt,
            "last_ecology_tick":self.scheduler.last_ecology_tick(),"seed":self.config.seed,
            "incident_light":self.incident_light,"escaped_light":self.escaped_light,
            "available_light":self.ledger.pools().light_available,"last_update":self.last_update,"diagnostics":diagnostics,
            "initial_nutrient":self.config.initial_nutrient,"initial_energy":self.config.initial_energy
            ,"ecology":self.ecology.totals,
            "producer_count":self.ledger.organisms().values().filter(|o|o.alive&&o.genome.photosynthetic()).count(),
            "consumer_count":self.ledger.organisms().values().filter(|o|o.alive&&o.genome.can_ingest()).count()
        }).as_object().ok_or_else(|| AccountingError::Invariant("summary metadata is not an object".into()))?.clone());
        encode(&value)
    }

    pub fn diagnostics(&self) -> Option<&crate::diagnostics::Diagnostics> {
        self.diagnostics.as_ref()
    }

    pub fn subject_json(&self, id: u64) -> Result<String> {
        let organism = self
            .organisms()
            .get(&OrganismId(id))
            .ok_or_else(|| AccountingError::Invalid("unknown subject".into()))?;
        encode(
            &serde_json::json!({"organism":organism,"tick":self.tick(),"rules":self.rules,
            "configuration":self.config,"food_web":self.ecology.interactions.food_web,
            "position":self.positions[&organism.id],"heading":self.ecology.headings.get(&organism.id)}),
        )
    }

    pub fn snapshot_json(&self) -> Result<String> {
        self.audit()?;
        let organisms = self.ledger.organisms().values().map(|organism| {
            let position = self.positions[&organism.id];
            Ok(serde_json::json!({"organism":organism,"position":position,"cell_index":self.grid.cell_index(position)?}))
        }).collect::<Result<Vec<_>>>()?;
        encode(
            &serde_json::json!({"schema_version":1,"seed_algorithm":"splitmix64-counter-v1",
            "rules":self.rules,"config":self.config,"clock":self.scheduler,
            "environment":self.grid,"organisms":organisms,"pools":self.ledger.pools(),
            "incident_light":self.incident_light,"escaped_light":self.escaped_light,"last_update":self.last_update,"diagnostics":self.diagnostics,
            "ecology":{"next_id":self.ecology.next_id,"totals":self.ecology.totals,
                "headings":self.ecology.headings.iter().map(|(id,heading)|serde_json::json!({"id":id,"heading":heading})).collect::<Vec<_>>()}}),
        )
    }
}

fn process_producer(
    draft: &mut Draft<'_>,
    id: OrganismId,
    light_share: f64,
    nutrient_share: f64,
    duration: f64,
    ecology: &crate::habitat_config::EcologyConfig,
    report: &mut WindowReport,
) -> Result<()> {
    let rules = draft.rules;
    let organism = draft.get(id)?;
    let headroom = (organism.tissue.nutrient * rules.reserve_energy_capacity_per_tissue_nutrient
        - organism.reserves.energy)
        .max(0.0);
    let light = light_share
        .min(rules.max_absorbed_light_per_time * duration)
        .min(headroom / rules.photosynthesis_efficiency)
        .min(draft.pools.light_available);
    if light > 0.0 {
        physiology::photosynthesize(draft, id, light, duration)?;
        report.light_captured += light;
    }
    let organism = draft.get(id)?;
    let headroom = (organism.tissue.nutrient * rules.reserve_nutrient_capacity_per_tissue_nutrient
        - organism.reserves.nutrient)
        .max(0.0);
    let nutrient = nutrient_share
        .min(rules.max_absorption_nutrient_per_time * duration)
        .min(headroom)
        .min(draft.pools.mineral_nutrient);
    if nutrient > 0.0 {
        physiology::absorb(draft, id, nutrient, duration)?;
        report.nutrient_absorbed += nutrient;
    }
    physiology::maintain(draft, id, duration)?;
    report.producers_processed += 1;
    let organism = draft.get(id)?;
    if !organism.alive {
        report.deaths += 1;
        return Ok(());
    }
    if !reserve_for_birth(draft, id, ecology)? {
        report.tissue_grown += physiology::grow_available(draft, id, duration)?;
    }
    Ok(())
}

fn light_balance(rules: &WorldRules, incident: f64, escaped: f64, pools: Pools) -> Result<f64> {
    finite(incident, "incident light", false)?;
    finite(escaped, "escaped light", false)?;
    let mut accounted = Sum::default();
    for value in [pools.absorbed_light, pools.light_available, escaped] {
        accounted.add(value);
    }
    if !rules.close(accounted.value(), incident) {
        return Err(AccountingError::Invariant(
            "incident light does not match captured, available, and escaped light".into(),
        ));
    }
    Ok(accounted.value() - incident)
}

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|error| AccountingError::Invalid(error.to_string()))
}
