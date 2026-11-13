use crate::{
    agents::{self, Controls},
    anatomy::BodyState,
    diagnostics::{Diagnostics, EvolutionEvent},
    ecology::{EcologyState, initial_heading},
    environment::{EnvironmentGrid, Position},
    habitat::WindowReport,
    habitat_config::{HabitatConfig, seeded_position},
    ids::OrganismId,
    inherited::InheritedGenome,
    lifecycle,
    organism::{DeathCause, Organism},
    physiology,
    resources::{AccountingError, Resources, Result, finite},
    spatial::{distance, segment_distance},
    transactions::Draft,
};
use std::collections::BTreeMap;
use world_genetics::{body::ModuleKind, genome::Genome, mutation::mutate};

pub(crate) struct PopulationWindow<'a> {
    pub blocked_controls: std::collections::BTreeSet<OrganismId>,
    pub spatial: Option<crate::spatial::SpatialIndex>,
    pub largest_extent: f64,
    pub grid: &'a mut EnvironmentGrid,
    pub positions: &'a mut BTreeMap<OrganismId, Position>,
    pub ecology: &'a mut EcologyState,
    pub diagnostics: &'a mut Diagnostics,
    pub config: &'a HabitatConfig,
    pub tick: u64,
    pub duration: f64,
    pub controls: &'a Controls,
}

impl PopulationWindow<'_> {
    pub fn run(&mut self, draft: &mut Draft<'_>) -> Result<WindowReport> {
        draft.preserve_corpses = true;
        let mut report = WindowReport {
            tick: self.tick,
            duration: self.duration,
            ..WindowReport::default()
        };
        let mut ids: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.alive)
            .map(|(id, _)| *id)
            .collect();
        if !ids.is_empty() {
            let offset = (self.tick / draft.rules.ecology_interval_ticks) as usize % ids.len();
            ids.rotate_left(offset);
        }
        let config =
            self.config.evolution.as_ref().ok_or_else(|| {
                AccountingError::Invariant("evolution configuration missing".into())
            })?;
        self.spatial = Some(crate::spatial::SpatialIndex::new(
            self.grid,
            self.positions,
            draft.organisms(),
        )?);
        for id in &ids {
            let position = self.positions[id];
            for shape in agents::circles(draft.get(*id)?, position, self.ecology.headings[id])? {
                self.largest_extent = self
                    .largest_extent
                    .max(distance(position, shape.center) + shape.radius);
            }
        }
        let initial_energy: BTreeMap<_, _> = ids
            .iter()
            .filter_map(|id| {
                let o = draft.get(*id).ok()?;
                o.genome.body.as_ref()?.circle.as_ref()?;
                Some((*id, o.reserves.energy))
            })
            .collect();
        self.diagnostics
            .behavior
            .retain(|id, _| initial_energy.contains_key(id));
        for id in initial_energy.keys() {
            let recent = self
                .diagnostics
                .behavior
                .get(id)
                .filter(|b| self.tick.saturating_sub(b.attacker_tick) <= 120);
            let attacker = recent.and_then(|b| b.attacker);
            let attacker_tick = recent.map_or(0, |b| b.attacker_tick);
            let requested = self
                .controls
                .values
                .get(id)
                .map(|v| v.iter().map(|(p, v)| ((*p % 256) as u8, *v)).collect())
                .unwrap_or_default();
            self.diagnostics.behavior.insert(
                *id,
                crate::diagnostics::Behavior {
                    upkeep_per_time: crate::metabolism::upkeep(
                        draft.get(*id)?,
                        draft.rules,
                        config,
                        1.0,
                    )
                    .total(),
                    attacker,
                    attacker_tick,
                    requested,
                    energy_before: draft.get(*id)?.reserves.energy,
                    ..Default::default()
                },
            );
        }
        for id in &ids {
            if !draft.get(*id)?.alive {
                continue;
            }
            if config.energy_budgeting {
                let o = draft.get(*id)?;
                let costs = crate::metabolism::upkeep(o, draft.rules, config, self.duration);
                let needed = if self.controls.authorized.contains(id) {
                    let forage = o
                        .genome
                        .body
                        .as_ref()
                        .and_then(|b| b.circle.as_ref())
                        .map_or(0.0, |circle| {
                            let root = o.genome.body.as_ref().expect("body").modules[0].id;
                            let feed = self.action(o, root, 120);
                            let thrust = self.action(o, root, 110);
                            let turn = self.action(o, root, 111).abs();
                            (feed
                                * draft.rules.max_ingestion_nutrient_per_time
                                * draft.rules.digestion_energy_per_ingested_nutrient
                                + thrust
                                    * circle.speed
                                    * self.config.ecology.consumer_speed
                                    * self.config.ecology.movement_energy_per_distance
                                + (thrust + turn) * 0.002)
                                * self.duration
                        });
                    costs.total() + forage
                } else {
                    costs.body_total()
                };
                let cell = self.grid.cell_index(self.positions[id])?;
                self.grid.cells[cell].load_pools(&mut draft.pools);
                let (amount, fuel) =
                    crate::metabolism::mobilize(draft, *id, config, self.duration, needed)?;
                self.grid.cells[cell].store_pools(draft.pools);
                let behavior = self.diagnostics.behavior.entry(*id).or_default();
                behavior.catabolized_nutrient = amount;
                behavior.catabolic_energy = fuel;
                self.diagnostics.catabolized_nutrient += amount;
                self.diagnostics.catabolic_energy += fuel;
                if self.controls.authorized.contains(id)
                    && draft.get(*id)?.reserves.energy < costs.total()
                    && draft
                        .get(*id)?
                        .genome
                        .body
                        .as_ref()
                        .is_some_and(|b| b.circle.is_some())
                {
                    self.graze_circle(draft, *id, &mut report)?;
                }
            }
            if self.controls.authorized.contains(id) {
                let neural = agents::neural_cost(draft.get(*id)?, config, self.duration);
                let sensing = agents::sensor_cost(
                    draft.get(*id)?,
                    draft.rules.sensor_energy_per_time,
                    self.duration,
                );
                let available = draft.get(*id)?.reserves.energy;
                let requested = neural + sensing;
                let paid = available.min(requested);
                if paid < requested {
                    self.blocked_controls.insert(*id);
                    self.diagnostics
                        .behavior
                        .entry(*id)
                        .or_default()
                        .failures
                        .insert("controller_unfunded".into(), 1);
                }
                expend(draft, *id, paid)?;
                let fraction = if requested > 0.0 {
                    paid / requested
                } else {
                    1.0
                };
                self.diagnostics
                    .agent_cost(*id, "neural", neural * fraction)?;
                self.diagnostics
                    .agent_cost(*id, "sensing", sensing * fraction)?;
            }
        }
        let mut producers = vec![Vec::new(); self.grid.cells.len()];
        for id in &ids {
            if draft.get(*id)?.alive && draft.get(*id)?.genome.photosynthetic() {
                producers[self.grid.cell_index(self.positions[id])?].push(*id);
            }
        }
        for (cell_index, residents) in producers.into_iter().enumerate() {
            if residents.is_empty() {
                continue;
            }
            let area = |o: &Organism| {
                capability(o, ModuleKind::Photosynthetic)
                    / o.genome.body.as_ref().map_or(12.0, |b| 12.0 * b.density)
            };
            let total_area: f64 = residents
                .iter()
                .map(|id| area(draft.get(*id).expect("resident")))
                .sum();
            let cell_area =
                draft.rules.world_width * draft.rules.world_height / self.grid.cells.len() as f64;
            let light_share = self.grid.cells[cell_index].light_available / residents.len() as f64;
            let mineral_share =
                self.grid.cells[cell_index].mineral_nutrient / residents.len() as f64;
            self.grid.cells[cell_index].load_pools(&mut draft.pools);
            for id in residents {
                let o = draft.get(id)?;
                let embodied = config.embodied_ecology
                    && o.genome.body.as_ref().is_some_and(|b| b.circle.is_some());
                let exposed = if embodied {
                    (area(o) / cell_area).min(1.0)
                } else {
                    capability(o, ModuleKind::Photosynthetic).min(1.0)
                };
                let light_share = if embodied {
                    self.grid.cells[cell_index].light_available * area(o)
                        / total_area.max(cell_area)
                } else {
                    light_share
                };
                let capture = exposed
                    * (1.0 - 0.7 * crate::terrain::cover(&config.terrain, self.positions[&id]))
                    * crate::food_web::adaptations(o).map_or(1.0, |a| 0.1 + 0.9 * a.capture);
                let light = light_share
                    .min(draft.rules.max_absorbed_light_per_time * self.duration * capture)
                    .min(
                        ((o.reserve_capacity(draft.rules)?.energy - o.reserves.energy).max(0.0))
                            / draft.rules.photosynthesis_efficiency,
                    )
                    .min(draft.pools.light_available);
                if light > 0.0 {
                    let before = draft.get(id)?.reserves.energy;
                    physiology::photosynthesize(draft, id, light, self.duration)?;
                    self.diagnostics
                        .behavior
                        .entry(id)
                        .or_default()
                        .light_energy += draft.get(id)?.reserves.energy - before;
                    report.light_captured += light;
                }
                let o = draft.get(id)?;
                let nutrient = mineral_share
                    .min(draft.rules.max_absorption_nutrient_per_time * self.duration * capture)
                    .min((o.reserve_capacity(draft.rules)?.nutrient - o.reserves.nutrient).max(0.0))
                    .min(draft.pools.mineral_nutrient);
                if nutrient > 0.0 {
                    physiology::absorb(draft, id, nutrient, self.duration)?;
                    report.nutrient_absorbed += nutrient;
                }
                report.producers_processed += 1;
            }
            self.grid.cells[cell_index].store_pools(draft.pools);
        }
        self.prepare_interactions(draft)?;
        for id in &ids {
            let from = self.positions[id];
            self.move_one(draft, *id, &mut report)?;
            self.spatial
                .as_mut()
                .expect("window spatial index")
                .relocate(self.grid, *id, from, self.positions[id])?;
        }
        for id in &ids {
            if !draft.get(*id)?.alive {
                continue;
            }
            self.interact(draft, *id, &mut report)?;
            if !draft.get(*id)?.alive {
                continue;
            }
            let development_before = draft.get(*id)?.reserves.energy;
            let from = self.positions[id];
            self.develop(draft, *id, &mut report)?;
            self.spatial
                .as_mut()
                .expect("window spatial index")
                .relocate(self.grid, *id, from, self.positions[id])?;
            for shape in agents::circles(
                draft.get(*id)?,
                self.positions[id],
                self.ecology.headings[id],
            )? {
                self.largest_extent = self
                    .largest_extent
                    .max(distance(self.positions[id], shape.center) + shape.radius);
            }
            if let Some(behavior) = self.diagnostics.behavior.get_mut(id) {
                behavior.reward += development_before - draft.get(*id)?.reserves.energy;
            }
            let cell = self.grid.cell_index(self.positions[id])?;
            self.grid.cells[cell].load_pools(&mut draft.pools);
            let producer = draft.get(*id)?;
            if producer.genome.photosynthetic()
                && self.config.ecology.producer_litter_fraction_per_time > 0.0
            {
                let amount = (producer.tissue.nutrient
                    * self.config.ecology.producer_litter_fraction_per_time
                    * self.duration)
                    .min(
                        (producer.tissue.nutrient - draft.rules.minimum_viable_tissue_nutrient)
                            .max(0.0),
                    );
                if amount > 0.0 {
                    lifecycle::damage(draft, *id, amount)?;
                    report.ecology.producer_litter_nutrient += amount;
                    if !draft.get(*id)?.alive {
                        self.grid.cells[cell].store_pools(draft.pools);
                        self.death(draft, *id, None)?;
                        continue;
                    }
                }
            }
            if config.energy_budgeting {
                let (receipt, shortage) =
                    crate::metabolism::pay_upkeep(draft, *id, config, self.duration)?;
                self.diagnostics
                    .agent_cost(*id, "body_upkeep", receipt.body)?;
                self.diagnostics
                    .agent_cost(*id, "maintenance", receipt.maintenance)?;
                self.diagnostics
                    .behavior
                    .entry(*id)
                    .or_default()
                    .maintenance_shortfall = shortage;
            } else {
                let before = draft.get(*id)?.reserves.energy;
                let upkeep = (body_upkeep(draft.get(*id)?, config.module_upkeep_per_time)
                    + crate::food_web::adaptations(draft.get(*id)?).map_or(0.0, |a| {
                        a.investments() * 0.008 * draft.get(*id).expect("organism").tissue.nutrient
                    }))
                    * self.duration;
                let paid = upkeep.min(before);
                if paid > 0.0 {
                    expend(draft, *id, paid)?;
                }
                self.diagnostics.agent_cost(*id, "body_upkeep", paid)?;
                let before = draft.get(*id)?.reserves.energy;
                physiology::maintain(draft, *id, self.duration)?;
                self.diagnostics.agent_cost(
                    *id,
                    "maintenance",
                    before - draft.get(*id)?.reserves.energy,
                )?;
            }
            self.grid.cells[cell].store_pools(draft.pools);
            if !draft.get(*id)?.alive {
                self.death(draft, *id, None)?;
            }
            if draft.get(*id)?.genome.can_ingest() {
                report.ecology.consumers_processed += 1;
            }
        }
        for (id, initial) in initial_energy {
            if let Some(behavior) = self.diagnostics.behavior.get_mut(&id) {
                behavior.energy_after = draft.get(id)?.reserves.energy;
                behavior.reward += draft.get(id)?.reserves.energy
                    - initial
                    - behavior.catabolic_energy
                    - config.injury_feedback * behavior.damage_received;
            }
        }
        self.births(draft, &mut report)?;
        self.neural_development(draft)?;
        for (id, behavior) in self.diagnostics.behavior.iter_mut() {
            behavior.energy_after = draft.get(*id)?.reserves.energy;
        }
        if config.energy_history_samples > 0 {
            for id in &ids {
                let organism = draft.get(*id)?;
                let behavior = self
                    .diagnostics
                    .behavior
                    .get(id)
                    .cloned()
                    .unwrap_or_default();
                if !organism.alive
                    || self
                        .diagnostics
                        .energy_history
                        .get(id)
                        .and_then(|h| h.back())
                        .is_none_or(|last| {
                            self.tick.saturating_sub(last.tick)
                                >= config.energy_history_interval_ticks
                        })
                    || behavior.costs.get("offspring").is_some_and(|v| *v > 0.0)
                {
                    self.diagnostics.record_energy(
                        *id,
                        crate::diagnostics::EnergySample {
                            tick: self.tick,
                            tissue: organism.tissue.nutrient,
                            upkeep_per_time: behavior.upkeep_per_time,
                            behavior,
                        },
                        config.energy_history_samples,
                        organism.death_cause,
                    );
                }
            }
        }
        let alive = draft.organisms().filter(|(_, o)| o.alive).count();
        report.ecology.deaths = (ids.len() as u64)
            .checked_add(report.ecology.births)
            .and_then(|n| n.checked_sub(alive as u64))
            .ok_or_else(|| AccountingError::Invariant("population counters disagree".into()))?;
        report.deaths = report.ecology.deaths as usize;
        report.ecology.tissue_grown = report.tissue_grown;
        if !draft.death_remains.is_empty() {
            return Err(AccountingError::Invariant(
                "unassigned death remains".into(),
            ));
        }
        draft.preserve_corpses = false;
        Ok(report)
    }

    pub(crate) fn action(&self, o: &Organism, module: u64, channel: u8) -> f64 {
        if self.blocked_controls.contains(&o.id) {
            return 0.0;
        }
        if o.brain().is_none() {
            return if matches!(channel, 100 | 101 | 103 | 120) {
                1.0
            } else {
                0.0
            };
        }
        self.controls
            .values
            .get(&o.id)
            .and_then(|p| p.get(&(module * 256 + channel as u64)))
            .copied()
            .unwrap_or(0.0)
    }

    pub(crate) fn action_energy(&self, o: &Organism, rules: &crate::rules::WorldRules) -> f64 {
        let config = self.config.evolution.as_ref().expect("evolution");
        let reserve = if config.energy_budgeting {
            crate::metabolism::upkeep(o, rules, config, self.duration).body_total()
        } else {
            0.0
        };
        (o.reserves.energy - reserve).max(0.0)
    }

    fn move_one(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        let o = draft.get(id)?;
        if !o.alive {
            return Ok(());
        }
        if o.genome.body.as_ref().is_some_and(|g| g.circle.is_some()) {
            return self.move_circle(draft, id, report);
        }
        let g = o
            .genome
            .body
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("body blueprint missing".into()))?;
        let b = o
            .body
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("body state missing".into()))?;
        let heading = self.ecology.headings[&id];
        let origin = self.positions[&id];
        let layout = g.layout().map_err(|e| AccountingError::Invalid(e.into()))?;
        let mut force = (0.0, 0.0);
        let mut torque = 0.0;
        let mut effort = 0.0;
        for module in g
            .modules
            .iter()
            .filter(|m| m.kind == ModuleKind::Propulsion)
        {
            let tissue = b
                .modules
                .iter()
                .find(|m| m.id == module.id)
                .map_or(0.0, |m| m.nutrient);
            let strength = tissue * module.investment * b.functional(g, module.id);
            let thrust = self.action(o, module.id, 110);
            let turn = self.action(o, module.id, 111);
            let direction = heading + module.orientation;
            force.0 += direction.cos() * strength * thrust * 15.0;
            force.1 += direction.sin() * strength * thrust * 15.0;
            let attachment = layout[&module.id];
            let scale = (b.developed() / g.target_nutrient()).sqrt();
            let arm = (
                scale * (attachment.0 * heading.cos() - attachment.1 * heading.sin()),
                scale * (attachment.0 * heading.sin() + attachment.1 * heading.cos()),
            );
            torque += strength * turn * 1.5
                + (arm.0 * direction.sin() - arm.1 * direction.cos()) * strength * thrust * 15.0;
            effort += strength * (thrust + turn.abs()) * 0.08 * self.duration;
        }
        if effort == 0.0 {
            return Ok(());
        }
        let drag = self
            .config
            .evolution
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("evolution configuration missing".into()))?
            .body_drag;
        let change = (torque / (b.inertia(g)? * drag) * self.duration).clamp(-0.4, 0.4);
        let angle = (heading + change).sin().atan2((heading + change).cos());
        let mut destination = Position {
            x: origin.x + force.0 / (o.tissue.nutrient * drag) * self.duration,
            y: origin.y + force.1 / (o.tissue.nutrient * drag) * self.duration,
        };
        let mut rotation = angle;
        if !clear_motion(
            draft,
            self.grid,
            self.positions,
            &self.ecology.headings,
            id,
            destination,
            rotation,
        )? {
            destination = origin;
            self.diagnostics.blocked_moves = self
                .diagnostics
                .blocked_moves
                .checked_add(1)
                .ok_or_else(|| AccountingError::Invalid("movement counter overflow".into()))?;
            if !clear_motion(
                draft,
                self.grid,
                self.positions,
                &self.ecology.headings,
                id,
                origin,
                rotation,
            )? {
                rotation = heading;
            }
        }
        let traveled = distance(origin, destination);
        let actual_turn = (rotation - heading).sin().atan2((rotation - heading).cos());
        let translation_work =
            (force.0 * (destination.x - origin.x) + force.1 * (destination.y - origin.y)).max(0.0);
        let rotation_work = (torque * actual_turn).max(0.0);
        let work_scale = self
            .config
            .evolution
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("evolution configuration missing".into()))?
            .work_energy_scale;
        let expenditure = effort
            + (translation_work + rotation_work) * work_scale / draft.rules.movement_efficiency;
        finite(expenditure, "body motion expenditure", false)?;
        if expenditure <= self.action_energy(o, draft.rules) {
            expend(draft, id, expenditure)?;
            self.diagnostics.agent_cost(id, "movement", expenditure)?;
            self.positions.insert(id, destination);
            self.ecology.headings.insert(id, rotation);
            report.ecology.distance_moved += traveled;
        }
        Ok(())
    }

    fn interact(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        let o = draft.get(id)?.clone();
        if o.genome.body.as_ref().is_some_and(|g| g.circle.is_some()) {
            return self.feed_circle(draft, id, report);
        }
        let Some(genome) = &o.genome.body else {
            return Ok(());
        };
        let Some(body) = &o.body else {
            return Ok(());
        };
        let shapes = body.circles(genome, self.positions[&id], self.ecology.headings[&id])?;
        let mut ingestion_left = draft.rules.max_ingestion_nutrient_per_time * self.duration;
        for mouth in shapes
            .iter()
            .filter(|c| c.kind == ModuleKind::Mouth && c.nutrient > 0.015)
        {
            let gene = genome
                .modules
                .iter()
                .find(|m| m.id == mouth.id)
                .ok_or_else(|| AccountingError::Invariant("mouth gene missing".into()))?;
            let functionality = body.functional(genome, mouth.id);
            if functionality <= 0.05 {
                continue;
            }
            let reach = 0.08 * gene.investment * functionality;
            let attack = self.action(&o, mouth.id, 121);
            if attack > 0.0
                && let Some((target, module, _)) =
                    self.contact(draft, id, mouth.center, mouth.radius + reach, false)?
            {
                let econfig = self.config.evolution.as_ref().ok_or_else(|| {
                    AccountingError::Invariant("evolution configuration missing".into())
                })?;
                let cost = econfig.attack_energy_per_time
                    * self.duration
                    * attack
                    * gene.investment
                    * functionality;
                if cost <= self.action_energy(draft.get(id)?, draft.rules) {
                    expend(draft, id, cost)?;
                    self.diagnostics.agent_cost(id, "attack", cost)?;
                    let protection = capability(draft.get(target)?, ModuleKind::Armor);
                    let available = draft
                        .get(target)?
                        .body
                        .as_ref()
                        .and_then(|b| b.modules.iter().find(|m| m.id == module))
                        .map_or(0.0, |m| m.nutrient);
                    let amount = (econfig.attack_nutrient_per_time
                        * self.duration
                        * attack
                        * gene.investment
                        * functionality
                        / (1.0 + protection))
                        .min(available);
                    if amount > 0.0 {
                        let cell = self.grid.cell_index(self.positions[&target])?;
                        self.grid.cells[cell].load_pools(&mut draft.pools);
                        damage_module(draft, target, module, amount)?;
                        self.grid.cells[cell].store_pools(draft.pools);
                        crate::diagnostics::increment(&mut self.diagnostics.attack_hits, 1)?;
                        self.diagnostics.damage_nutrient += amount;
                        self.event(
                            "damage",
                            id,
                            (Some(target), Some(module)),
                            amount,
                            Vec::new(),
                            None,
                        )?;
                        if !draft.get(target)?.alive {
                            self.death(draft, target, Some(id))?;
                        }
                    }
                }
            }
            let feed = self.action(&o, mouth.id, 120);
            if feed <= 0.0 || ingestion_left <= 0.0 {
                continue;
            }
            if let Some((target, module, _)) =
                self.contact(draft, id, mouth.center, mouth.radius + reach, true)?
            {
                crate::diagnostics::increment(&mut self.diagnostics.encounters, 1)?;
                let before = draft.get(target)?.clone();
                let available = before
                    .body
                    .as_ref()
                    .and_then(|b| b.modules.iter().find(|m| m.id == module))
                    .map_or(before.tissue.nutrient, |m| m.nutrient);
                let digestion_energy = if self
                    .config
                    .evolution
                    .as_ref()
                    .expect("evolution")
                    .energy_budgeting
                    && draft.rules.tissue_energy_per_nutrient
                        * draft.rules.energy_assimilation_fraction
                        < draft.rules.digestion_energy_per_ingested_nutrient
                {
                    self.action_energy(draft.get(id)?, draft.rules)
                } else {
                    draft.get(id)?.reserves.energy
                };
                let mut amount = ingestion_left
                    .min(
                        draft.rules.max_ingestion_nutrient_per_time
                            * self.duration
                            * feed
                            * functionality.min(1.0),
                    )
                    .min(available)
                    .min(digestion_energy / draft.rules.digestion_energy_per_ingested_nutrient);
                if amount * draft.rules.digestion_energy_per_ingested_nutrient
                    > draft.get(id)?.reserves.energy
                {
                    amount = amount.next_down().max(0.0);
                }
                if amount <= 0.0 {
                    continue;
                }
                let cell = self.grid.cell_index(self.positions[&id])?;
                self.grid.cells[cell].load_pools(&mut draft.pools);
                physiology::feed_from_module(
                    draft,
                    id,
                    target,
                    amount,
                    self.duration,
                    Some(module),
                )?;
                let target_cell = self.grid.cell_index(self.positions[&target])?;
                if target_cell != cell {
                    let after = draft.get(target)?;
                    if after.alive {
                        let released =
                            (before.reserves.nutrient - after.reserves.nutrient).max(0.0);
                        draft.pools.mineral_nutrient -= released;
                        self.grid.cells[target_cell].mineral_nutrient += released;
                    }
                }
                self.grid.cells[cell].store_pools(draft.pools);
                self.diagnostics.agent_cost(
                    id,
                    "digestion",
                    amount * draft.rules.digestion_energy_per_ingested_nutrient,
                )?;
                report.ecology.feeding_events += 1;
                report.ecology.ingested_nutrient += amount;
                ingestion_left -= amount;
                self.event(
                    "feeding",
                    id,
                    (Some(target), Some(module)),
                    amount,
                    Vec::new(),
                    None,
                )?;
                if !draft.get(target)?.alive {
                    self.death(draft, target, Some(id))?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn combat_contact(
        &self,
        draft: &Draft<'_>,
        id: OrganismId,
        center: Position,
        reach: f64,
    ) -> Result<Option<(OrganismId, u64, f64)>> {
        let heading = self.ecology.headings[&id];
        let mut best: Option<(OrganismId, u64, u8, f64)> = None;
        for other in self.spatial.as_ref().expect("spatial index").within(
            self.grid,
            self.positions,
            center,
            reach + self.largest_extent,
        )? {
            if other == id || !draft.get(other)?.alive {
                continue;
            }
            let actor = draft.get(id)?;
            let target = draft.get(other)?;
            let retaliate = self.diagnostics.behavior.get(&id).is_some_and(|b| {
                b.attacker == Some(other) && self.tick.saturating_sub(b.attacker_tick) <= 120
            });
            let priority = u8::from(
                !agents::edible(actor, target) && !agents::edible(target, actor) && !retaliate,
            );
            for shape in agents::circles(
                draft.get(other)?,
                self.positions[&other],
                self.ecology.headings[&other],
            )? {
                let dx = shape.center.x - center.x;
                let dy = shape.center.y - center.y;
                let d = dx.hypot(dy);
                let alignment = (dx * heading.cos() + dy * heading.sin()) / d.max(1e-12);
                let gap = d - shape.radius;
                let score = gap + 0.3 * (1.0 - alignment);
                if shape.nutrient > 0.0
                    && gap <= reach
                    && alignment >= 0.0
                    && best.is_none_or(|(old, _, p, s)| (priority, score, other) < (p, s, old))
                {
                    best = Some((other, shape.id, priority, score));
                }
            }
        }
        Ok(best.map(|(id, module, _, score)| (id, module, score)))
    }

    pub(crate) fn contact(
        &self,
        draft: &Draft<'_>,
        id: OrganismId,
        center: Position,
        reach: f64,
        food: bool,
    ) -> Result<Option<(OrganismId, u64, f64)>> {
        let mut best = None;
        let candidates = self
            .spatial
            .as_ref()
            .expect("window spatial index")
            .within(
                self.grid,
                self.positions,
                center,
                reach + self.largest_extent,
            )?;
        for other in candidates.iter().filter(|other| **other != id) {
            let o = draft.get(*other)?;
            if !o.alive {
                continue;
            }
            if food
                && agents::available_food(
                    draft.get(id)?,
                    o,
                    self.config.ecology.grazing_residual_fraction,
                ) <= 1e-8
            {
                continue;
            }
            for shape in agents::circles(o, self.positions[other], self.ecology.headings[other])? {
                if shape.nutrient <= 0.0 {
                    continue;
                }
                let gap = distance(center, shape.center) - shape.radius;
                if gap <= reach
                    && best
                        .as_ref()
                        .is_none_or(|(b, m, d): &(OrganismId, u64, f64)| {
                            gap.total_cmp(d)
                                .then(other.cmp(b))
                                .then(shape.id.cmp(m))
                                .is_lt()
                        })
                {
                    best = Some((*other, shape.id, gap));
                }
            }
        }
        Ok(best)
    }

    fn develop(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        let o = draft.get(id)?;
        let Some(g) = &o.genome.body else {
            return Ok(());
        };
        let root = g
            .modules
            .iter()
            .find(|m| m.parent.is_none())
            .ok_or_else(|| AccountingError::Invariant("body root missing".into()))?
            .id;
        let circle = g.circle.is_some();
        let growth_target = draft
            .rules
            .maturity_tissue_nutrient
            .max(g.target_nutrient());
        let agency = crate::food_web::adaptations(o).is_some_and(|a| a.agency);
        let repair = if circle && !agency {
            1.0
        } else {
            self.action(o, root, 101)
        };
        let mut growth = self.action(o, root, 100);
        if circle && !agency {
            growth = if o
                .body
                .as_ref()
                .map_or(o.tissue.nutrient, |body| body.developed())
                < draft
                    .rules
                    .maturity_tissue_nutrient
                    .max(g.target_nutrient())
            {
                1.0
            } else {
                0.0
            };
        }
        if o.brain().is_none() && o.mature(draft.rules) {
            growth = 0.0;
        }
        let repair_rate = self
            .config
            .evolution
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("evolution configuration missing".into()))?
            .repair_nutrient_per_time;
        let unit_cost =
            draft.rules.tissue_energy_per_nutrient + draft.rules.construction_heat_per_nutrient;
        let reserve = if circle
            || self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .energy_budgeting
        {
            reserve_floor(o, draft.rules, self.config)
        } else {
            0.0
        };
        let mut repair_amount = (repair_rate * self.duration * repair)
            .min(o.body.as_ref().map_or(0.0, |b| b.missing()))
            .min(o.reserves.nutrient)
            .min((o.reserves.energy - reserve).max(0.0) / unit_cost);
        if repair_amount * unit_cost > o.reserves.energy {
            repair_amount = repair_amount.next_down().max(0.0);
        }
        if repair_amount > 0.0 {
            let mut candidate = o.clone();
            if let Some(body) = &mut candidate.body {
                body.repair(repair_amount);
            }
            candidate.tissue = candidate.tissue.plus(Resources::new(
                repair_amount,
                repair_amount * draft.rules.tissue_energy_per_nutrient,
            )?)?;
            let affordable = !self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .energy_budgeting
                || candidate.reserves.energy
                    >= repair_amount * unit_cost
                        + reserve_floor(&candidate, draft.rules, self.config);
            if affordable
                && clear_growth(
                    draft,
                    self.grid,
                    self.positions,
                    &self.ecology.headings,
                    &candidate,
                )?
            {
                candidate.reserves = candidate
                    .reserves
                    .minus(Resources::new(repair_amount, repair_amount * unit_cost)?)?;
                draft.pools.heat += repair_amount * draft.rules.construction_heat_per_nutrient;
                draft.put(candidate);
                self.diagnostics
                    .agent_cost(id, "repair", repair_amount * unit_cost)?;
                self.diagnostics
                    .behavior
                    .entry(id)
                    .or_default()
                    .executed
                    .insert(101, repair_amount);
                self.diagnostics.repaired_nutrient += repair_amount;
                report.tissue_grown += repair_amount;
                self.event("repair", id, (None, None), repair_amount, Vec::new(), None)?;
            }
        }
        let o = draft.get(id)?;
        let mut amount = (draft.rules.max_growth_nutrient_per_time * self.duration * growth)
            .min(o.reserves.nutrient)
            .min((o.reserves.energy - reserve).max(0.0) / unit_cost);
        if circle {
            amount = amount.min(
                (growth_target
                    - o.body
                        .as_ref()
                        .map_or(o.tissue.nutrient, |body| body.developed()))
                .max(0.0),
            );
        }
        if amount * draft.rules.tissue_energy_per_nutrient
            + amount * draft.rules.construction_heat_per_nutrient
            > o.reserves.energy
        {
            amount = amount.next_down().max(0.0);
        }
        if amount > 0.0 {
            let mut candidate = o.clone();
            if let (Some(body), Some(genome)) = (&mut candidate.body, &candidate.genome.body) {
                body.grow(genome, amount)?;
            }
            candidate.tissue = candidate.tissue.plus(Resources::new(
                amount,
                amount * draft.rules.tissue_energy_per_nutrient,
            )?)?;
            let placement = if circle {
                self.growth_position(draft, &candidate)?
            } else if clear_growth(
                draft,
                self.grid,
                self.positions,
                &self.ecology.headings,
                &candidate,
            )? {
                Some(self.positions[&id])
            } else {
                None
            };
            let placement = placement.filter(|p| {
                !crate::development::rock_collision(
                    &self.config.evolution.as_ref().expect("evolution").terrain,
                    *p,
                    agents::circles(&candidate, *p, self.ecology.headings[&id])
                        .expect("candidate geometry")[0]
                        .radius,
                )
            });
            if let Some(position) = placement {
                let work = distance(self.positions[&id], position)
                    * self.config.ecology.movement_energy_per_distance;
                let reserve = if self
                    .config
                    .evolution
                    .as_ref()
                    .expect("evolution")
                    .energy_budgeting
                {
                    reserve_floor(&candidate, draft.rules, self.config)
                } else {
                    reserve
                };
                if o.reserves.energy >= amount * unit_cost + work + reserve {
                    physiology::grow(draft, id, amount, self.duration)?;
                    if work > 0.0 {
                        expend(draft, id, work)?;
                        self.diagnostics
                            .agent_cost(id, "growth_displacement", work)?;
                    }
                    self.positions.insert(id, position);
                    self.diagnostics
                        .agent_cost(id, "growth", amount * unit_cost)?;
                    self.diagnostics
                        .behavior
                        .entry(id)
                        .or_default()
                        .executed
                        .insert(100, amount);
                    report.tissue_grown += amount;
                } else {
                    crate::diagnostics::increment(
                        self.diagnostics
                            .growth_blocks
                            .entry("displacement_energy".into())
                            .or_default(),
                        1,
                    )?;
                }
            } else {
                crate::diagnostics::increment(
                    self.diagnostics
                        .growth_blocks
                        .entry(
                            if o.genome.photosynthetic() {
                                "producer_space"
                            } else {
                                "consumer_space"
                            }
                            .into(),
                        )
                        .or_default(),
                    1,
                )?;
            }
        }
        Ok(())
    }

    fn births(&mut self, draft: &mut Draft<'_>, report: &mut WindowReport) -> Result<()> {
        if !self.config.ecology.reproduction {
            return Ok(());
        }
        let mut parents: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.mature(draft.rules) && o.age >= o.next_reproduction_age)
            .map(|(id, _)| *id)
            .collect();
        let mut live = draft.organisms().filter(|(_, o)| o.alive).count();
        let config =
            self.config.evolution.as_ref().ok_or_else(|| {
                AccountingError::Invariant("evolution configuration missing".into())
            })?;
        let circle_mode = parents.iter().any(|id| {
            draft
                .get(*id)
                .is_ok_and(|o| o.genome.body.as_ref().is_some_and(|b| b.circle.is_some()))
        });
        if circle_mode && !parents.is_empty() {
            let offset = (self.tick / draft.rules.ecology_interval_ticks) as usize % parents.len();
            parents.rotate_left(offset);
        }
        for parent in parents {
            let parent_organism = draft.get(parent)?;
            if parent_organism.genome.photosynthetic()
                && parent_organism.age < self.config.ecology.producer_minimum_reproduction_age
            {
                self.diagnostics.birth_block("producer", "producer_age")?;
                continue;
            }
            if crate::food_web::adaptations(parent_organism).is_some_and(|a| a.agency) {
                let root = parent_organism.genome.body.as_ref().expect("body").modules[0].id;
                if self.action(parent_organism, root, 103) <= 0.1 {
                    self.diagnostics.birth_block(
                        crate::food_web::guild(parent_organism),
                        "neural_reproduction",
                    )?;
                    continue;
                }
            }
            let role = if parent_organism.genome.photosynthetic() {
                "producer"
            } else {
                "consumer"
            };
            let circle = parent_organism
                .genome
                .body
                .as_ref()
                .is_some_and(|b| b.circle.is_some());
            if circle
                && config.inherit_learning
                && config.online_learning
                && parent_organism.brain().is_some()
                && !self.controls.learned.contains_key(&parent)
            {
                self.diagnostics.birth_block(role, "parameter_sync")?;
                continue;
            }
            if live >= self.config.ecology.max_population {
                self.diagnostics.birth_block(role, "population_limit")?;
                continue;
            }
            let minimum = if config.embodied_ecology {
                let Some((tissue, reserves)) = offspring_funding(
                    draft.rules,
                    &draft.get(parent)?.genome,
                    config,
                    &self.config.ecology,
                )?
                else {
                    self.diagnostics.birth_block(role, "juvenile_capacity")?;
                    continue;
                };
                reserves.plus(Resources::new(
                    tissue,
                    tissue
                        * (draft.rules.tissue_energy_per_nutrient
                            + draft.rules.construction_heat_per_nutrient),
                )?)?
            } else {
                lifecycle::birth_investment(draft.rules)?
            };
            let reserves = draft.get(parent)?.reserves;
            if reserves.nutrient < minimum.nutrient || reserves.energy < minimum.energy {
                self.diagnostics.birth_block(role, "base_investment")?;
                continue;
            }
            let child = OrganismId(self.ecology.next_id);
            let mut inherited = draft.get(parent)?.genome.blueprint();
            if crate::food_web::adaptations(draft.get(parent)?).is_some_and(|a| a.inherit_structure)
            {
                inherited.brain = draft.get(parent)?.brain().cloned();
            }
            if let Some(parameters) = self.controls.learned.get(&parent) {
                let weights: BTreeMap<_, _> = parameters.weights.iter().copied().collect();
                let biases: BTreeMap<_, _> = parameters.biases.iter().copied().collect();
                let brain = inherited.brain.as_mut().ok_or_else(|| {
                    AccountingError::Invariant("learned inheritance has no brain".into())
                })?;
                for edge in &mut brain.connections {
                    edge.weight = weights.get(&edge.id).copied().unwrap_or(edge.weight);
                }
                for node in &mut brain.nodes {
                    node.bias = biases.get(&node.id).copied().unwrap_or(node.bias);
                }
            }
            let (genome, edits) = mutate(
                &inherited,
                &config.mutation,
                self.config.seed,
                parent.0,
                child.0,
                self.tick,
            )
            .map_err(|e| AccountingError::Invalid(e.into()))?;
            let genome = InheritedGenome::from(genome);
            let Some((tissue, child_reserves)) =
                offspring_funding(draft.rules, &genome, config, &self.config.ecology)?
            else {
                self.diagnostics.birth_block(role, "juvenile_capacity")?;
                continue;
            };
            let needed = child_reserves.plus(Resources::new(
                tissue,
                tissue
                    * (draft.rules.tissue_energy_per_nutrient
                        + draft.rules.construction_heat_per_nutrient),
            )?)?;
            let mut placement = None;
            let mut reserve = if circle || config.energy_budgeting {
                reserve_floor(draft.get(parent)?, draft.rules, self.config)
            } else {
                0.0
            };
            if parent_organism.genome.photosynthetic() {
                reserve = reserve.max(
                    parent_organism.reserve_capacity(draft.rules)?.energy
                        * self.config.ecology.producer_reproduction_reserve_fraction,
                );
            }
            if reserves.nutrient >= needed.nutrient && reserves.energy >= needed.energy + reserve {
                let prototype = Organism {
                    expressed_brain: None,
                    id: child,
                    genome: genome.clone(),
                    tissue: Resources::new(
                        tissue,
                        tissue * draft.rules.tissue_energy_per_nutrient,
                    )?,
                    reserves: Resources::default(),
                    age: 0.0,
                    alive: true,
                    parent_id: Some(parent),
                    next_reproduction_age: 0.0,
                    death_cause: None,
                    body: genome
                        .body
                        .as_ref()
                        .map(|g| BodyState::new(g, tissue))
                        .transpose()?,
                };
                let origin = self.positions[&parent];
                let phase = seeded_position(self.config.seed ^ parent.0, self.tick, 1.0, 1.0).x
                    * std::f64::consts::TAU;
                for attempt in 0..8 {
                    let angle = phase + attempt as f64 * std::f64::consts::TAU / 8.0;
                    let p = Position {
                        x: origin.x + angle.cos() * self.config.ecology.birth_radius,
                        y: origin.y + angle.sin() * self.config.ecology.birth_radius,
                    };
                    if !crate::development::rock_collision(
                        &config.terrain,
                        p,
                        agents::circles(&prototype, p, initial_heading(self.config.seed, child))?
                            [0]
                        .radius,
                    ) && clear_at(
                        draft,
                        self.grid,
                        self.positions,
                        &self.ecology.headings,
                        &prototype,
                        p,
                        initial_heading(self.config.seed, child),
                    )? {
                        placement = Some(p);
                        break;
                    }
                }
            }
            if let Some(position) = placement {
                let next = child.0.checked_add(1).ok_or_else(|| {
                    AccountingError::Invalid("offspring identity overflow".into())
                })?;
                lifecycle::reproduce_funded(
                    draft,
                    parent,
                    child,
                    true,
                    genome.blueprint(),
                    tissue,
                    child_reserves,
                )?;
                if draft.get(parent)?.genome.photosynthetic() {
                    let mut organism = draft.get(parent)?.clone();
                    organism.next_reproduction_age = organism.age
                        + draft
                            .rules
                            .reproduction_cooldown
                            .max(self.config.ecology.producer_reproduction_cooldown);
                    draft.put(organism);
                }
                self.positions.insert(child, position);
                self.ecology
                    .headings
                    .insert(child, initial_heading(self.config.seed, child));
                self.ecology.next_id = next;
                self.diagnostics.accepted_mutations +=
                    edits.iter().filter(|e| e.accepted).count() as u64;
                self.diagnostics.rejected_mutations +=
                    edits.iter().filter(|e| !e.accepted).count() as u64;
                self.diagnostics
                    .agent_cost(parent, "offspring", needed.energy)?;
                let behavior = self.diagnostics.behavior.entry(parent).or_default();
                behavior.executed.insert(103, 1.0);
                behavior.reward += config.reproduction_feedback;
                crate::diagnostics::increment(
                    self.diagnostics
                        .births_by_role
                        .entry(role.into())
                        .or_default(),
                    1,
                )?;
                self.diagnostics
                    .last_birth_tick
                    .insert(role.into(), self.tick);
                self.event("birth", parent, (Some(child), None), tissue, edits, None)?;
                report.ecology.births += 1;
                live += 1;
            } else {
                let reason =
                    if reserves.nutrient < needed.nutrient || reserves.energy < needed.energy {
                        "offspring_investment"
                    } else if reserves.energy < needed.energy + reserve {
                        "maintenance_reserve"
                    } else {
                        "placement"
                    };
                self.diagnostics.birth_block(role, reason)?;
                self.event(
                    "birth_blocked",
                    parent,
                    (None, None),
                    0.0,
                    edits,
                    Some(reason.into()),
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn event(
        &mut self,
        kind: &str,
        actor: OrganismId,
        affected: (Option<OrganismId>, Option<u64>),
        amount: f64,
        edits: Vec<world_genetics::mutation::Edit>,
        note: Option<String>,
    ) -> Result<()> {
        let (target, module) = affected;
        let limit = self
            .config
            .evolution
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("evolution configuration missing".into()))?
            .history_limit;
        self.diagnostics.event(
            EvolutionEvent {
                sequence: 0,
                tick: self.tick,
                kind: kind.into(),
                actor,
                target,
                module,
                amount,
                edits,
                note,
            },
            limit,
        )
    }

    pub(crate) fn death(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        source: Option<OrganismId>,
    ) -> Result<()> {
        let cause = match draft.get(id)?.death_cause {
            Some(DeathCause::Starvation) => "starvation",
            Some(DeathCause::Consumption) => "consumption",
            Some(DeathCause::Damage) => "damage",
            None => return Ok(()),
        };
        if let Some(remaining) = draft.death_remains.remove(&id)
            && remaining != Resources::default()
        {
            let organism = draft.get(id)?;
            let position = self.positions[&id];
            let cell = self.grid.cell_index(position)?;
            let corpse = crate::corpses::Corpse {
                source: id,
                position,
                age: 0.0,
                density: organism.genome.body.as_ref().map_or(1.0, |b| b.density),
                remaining,
                cause: organism.death_cause.expect("death cause"),
            };
            if self.grid.cells[cell].corpses.insert(id, corpse).is_some() {
                return Err(AccountingError::Invariant(
                    "duplicate corpse identity".into(),
                ));
            }
        }
        crate::diagnostics::increment(self.diagnostics.deaths.entry(cause.into()).or_default(), 1)?;
        let role = if draft.get(id)?.genome.photosynthetic() {
            "producer"
        } else {
            "consumer"
        };
        crate::diagnostics::increment(
            self.diagnostics
                .deaths_by_role
                .entry(role.into())
                .or_default()
                .entry(cause.into())
                .or_default(),
            1,
        )?;
        self.event(
            "death",
            id,
            (source, None),
            0.0,
            Vec::new(),
            Some(cause.into()),
        )
    }
}

pub fn capability(o: &Organism, kind: ModuleKind) -> f64 {
    if let Some(circle) = o.genome.body.as_ref().and_then(|b| b.circle.as_ref()) {
        return match kind {
            ModuleKind::Photosynthetic if circle.photosynthetic => o.tissue.nutrient,
            _ => 0.0,
        };
    }
    o.genome
        .body
        .as_ref()
        .zip(o.body.as_ref())
        .map_or(0.0, |(g, b)| {
            g.modules
                .iter()
                .filter(|m| m.kind == kind)
                .map(|m| {
                    b.modules
                        .iter()
                        .find(|s| s.id == m.id)
                        .map_or(0.0, |s| s.nutrient)
                        * m.investment
                        * b.functional(g, m.id)
                })
                .sum()
        })
}

pub(crate) fn body_upkeep(o: &Organism, rate: f64) -> f64 {
    o.genome
        .body
        .as_ref()
        .zip(o.body.as_ref())
        .map_or(0.0, |(g, b)| {
            g.modules
                .iter()
                .map(|m| {
                    b.modules
                        .iter()
                        .find(|s| s.id == m.id)
                        .map_or(0.0, |s| s.nutrient)
                        * m.investment
                        * rate
                })
                .sum()
        })
}

pub(crate) fn expend(draft: &mut Draft<'_>, id: OrganismId, energy: f64) -> Result<()> {
    if energy == 0.0 {
        return Ok(());
    }
    let mut o = draft.living(id)?;
    o.reserves = o.reserves.minus(Resources::new(0.0, energy)?)?;
    draft.pools.heat += energy;
    draft.put(o);
    Ok(())
}

pub(crate) fn reserve_floor(
    o: &Organism,
    rules: &crate::rules::WorldRules,
    config: &HabitatConfig,
) -> f64 {
    config.evolution.as_ref().map_or(0.0, |e| {
        if e.energy_budgeting {
            return (e.maintenance_reserve_time + e.foraging_reserve_time)
                * crate::metabolism::upkeep(o, rules, e, 1.0).total();
        }
        e.maintenance_reserve_time
            * (o.tissue.nutrient
                * (rules.maintenance_per_tissue_nutrient_per_time + e.module_upkeep_per_time)
                + agents::neural_cost(o, e, 1.0)
                + agents::sensor_cost(o, rules.sensor_energy_per_time, 1.0))
    })
}

pub(crate) fn damage_module(
    draft: &mut Draft<'_>,
    id: OrganismId,
    module: u64,
    amount: f64,
) -> Result<()> {
    let mut o = draft.living(id)?;
    let removed = o
        .body
        .as_mut()
        .ok_or_else(|| AccountingError::Invariant("damage body missing".into()))?
        .remove(amount, Some(module))?;
    let lost = Resources::new(removed, removed * draft.rules.tissue_energy_per_nutrient)?;
    o.tissue = o.tissue.minus(lost)?;
    draft.pools.detritus = draft.pools.detritus.plus(lost)?;
    lifecycle::after_tissue_loss(draft, o, DeathCause::Damage)
}

fn offspring_funding(
    rules: &crate::rules::WorldRules,
    g: &InheritedGenome,
    config: &crate::evolution_config::EvolutionConfig,
    ecology: &crate::habitat_config::EcologyConfig,
) -> Result<Option<(f64, Resources)>> {
    let scale = if config.embodied_ecology {
        g.body.as_ref().map_or(1.0, |b| {
            (b.target_nutrient().max(rules.maturity_tissue_nutrient) / 6.0).min(1.0)
        })
    } else {
        1.0
    };
    let base = (rules.offspring_tissue_nutrient * scale).max(rules.minimum_viable_tissue_nutrient);
    let tissue = offspring_tissue(base, g, config);
    let original_energy = (rules.offspring_reserve_energy * scale)
        .min(tissue * rules.reserve_energy_capacity_per_tissue_nutrient);
    let target = g
        .body
        .as_ref()
        .map_or(tissue, |b| b.target_nutrient().max(tissue));
    let funding = |tissue: f64| -> Result<Option<Resources>> {
        let prototype = Organism {
            expressed_brain: None,
            id: OrganismId(0),
            genome: g.clone(),
            tissue: Resources::new(tissue, tissue * rules.tissue_energy_per_nutrient)?,
            reserves: Resources::default(),
            age: 0.0,
            alive: true,
            parent_id: None,
            next_reproduction_age: 0.0,
            death_cause: None,
            body: g
                .body
                .as_ref()
                .map(|body| BodyState::new(body, tissue))
                .transpose()?,
        };
        let capacity = prototype.reserve_capacity(rules)?.energy;
        let required = if config.energy_budgeting && prototype.brain().is_some() {
            crate::metabolism::upkeep(&prototype, rules, config, 1.0).total()
                * config.juvenile_reserve_time
                + ecology.movement_energy_per_distance
                    * ecology.perception_radius
                    * g.body
                        .as_ref()
                        .and_then(|b| b.circle.as_ref())
                        .map_or(1.0, |c| c.perception)
        } else {
            0.0
        };
        let energy = original_energy.max(required);
        if energy <= capacity {
            return Ok(Some(Resources::new(
                rules.offspring_reserve_nutrient * scale,
                energy,
            )?));
        }
        Ok(None)
    };
    if let Some(reserves) = funding(tissue)? {
        return Ok(Some((tissue, reserves)));
    }
    let Some(mut reserves) = funding(target)? else {
        return Ok(None);
    };
    let (mut lower, mut upper) = (tissue, target);
    for _ in 0..40 {
        let middle = lower + (upper - lower) * 0.5;
        if let Some(available) = funding(middle)? {
            upper = middle;
            reserves = available;
        } else {
            lower = middle;
        }
    }
    Ok(Some((upper, reserves)))
}

fn offspring_tissue(
    base: f64,
    g: &Genome,
    config: &crate::evolution_config::EvolutionConfig,
) -> f64 {
    base + g.body.as_ref().map_or(0.0, |b| {
        b.modules.len().saturating_sub(1) as f64 * config.offspring_module_nutrient
    }) + g.brain.as_ref().map_or(0.0, |b| {
        b.nodes.len() as f64 * config.offspring_neuron_nutrient
            + b.connections.len() as f64 * config.offspring_connection_nutrient
    })
}

pub(crate) fn clear_at(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    o: &Organism,
    position: Position,
    heading: f64,
) -> Result<bool> {
    let shapes = agents::circles(o, position, heading)?;
    let (width, height, _, _) = grid.dimensions();
    for shape in shapes.iter().filter(|s| s.radius > 0.0) {
        if shape.center.x - shape.radius < 0.0
            || shape.center.y - shape.radius < 0.0
            || shape.center.x + shape.radius > width
            || shape.center.y + shape.radius > height
        {
            return Ok(false);
        }
    }
    for (id, other) in draft
        .organisms()
        .filter(|(id, other)| **id != o.id && other.alive)
    {
        for b in agents::circles(other, positions[id], headings[id])? {
            for a in &shapes {
                if a.radius > 0.0
                    && b.radius > 0.0
                    && distance(a.center, b.center) + 1e-10 < a.radius + b.radius
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

fn clear_growth(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    o: &Organism,
) -> Result<bool> {
    clear_at(
        draft,
        grid,
        positions,
        headings,
        o,
        positions[&o.id],
        headings[&o.id],
    )
}

fn clear_motion(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    id: OrganismId,
    to: Position,
    heading: f64,
) -> Result<bool> {
    let o = draft.get(id)?;
    if !clear_at(draft, grid, positions, headings, o, to, heading)? {
        return Ok(false);
    }
    let old = agents::circles(o, positions[&id], headings[&id])?;
    let new = agents::circles(o, to, heading)?;
    let angle = (heading - headings[&id])
        .sin()
        .atan2((heading - headings[&id]).cos())
        .abs();
    for (other, target) in draft
        .organisms()
        .filter(|(other, target)| **other != id && target.alive)
    {
        for t in agents::circles(target, positions[other], headings[other])?
            .into_iter()
            .filter(|s| s.radius > 0.0)
        {
            for (a, b) in old.iter().zip(&new).filter(|(a, _)| a.radius > 0.0) {
                let arc = distance(a.center, positions[&id]) * (1.0 - (angle * 0.5).cos());
                if segment_distance(t.center, a.center, b.center) + 1e-10
                    < a.radius + t.radius + arc
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

pub(crate) fn founder_position(
    grid: &EnvironmentGrid,
    organisms: &[Organism],
    positions: &BTreeMap<OrganismId, Position>,
    o: &Organism,
    seed: u64,
    explicit: Option<Position>,
) -> Result<Position> {
    let (width, height, _, _) = grid.dimensions();
    for attempt in 0..128u64 {
        let p = explicit.unwrap_or_else(|| seeded_position(seed ^ o.id.0, attempt, width, height));
        let shapes = agents::circles(o, p, initial_heading(seed, o.id))?;
        let mut clear = shapes.iter().all(|s| {
            s.center.x - s.radius >= 0.0
                && s.center.y - s.radius >= 0.0
                && s.center.x + s.radius <= width
                && s.center.y + s.radius <= height
        });
        for other in organisms {
            let other_shapes =
                agents::circles(other, positions[&other.id], initial_heading(seed, other.id))?;
            if shapes.iter().any(|a| {
                other_shapes
                    .iter()
                    .any(|b| distance(a.center, b.center) + 1e-10 < a.radius + b.radius)
            }) {
                clear = false;
                break;
            }
        }
        if clear {
            return Ok(p);
        }
        if explicit.is_some() {
            break;
        }
    }
    Err(AccountingError::Invalid(
        "no valid non-overlapping founder placement".into(),
    ))
}

pub(crate) fn audit_geometry(world: &crate::habitat::Habitat) -> Result<()> {
    let (width, height, _, _) = world.grid().dimensions();
    let mut shapes = BTreeMap::new();
    let mut extents = BTreeMap::new();
    let mut largest = 0.0_f64;
    for (id, o) in world.organisms().iter().filter(|(_, o)| o.alive) {
        let heading = world
            .heading(*id)
            .ok_or_else(|| AccountingError::Invariant("body heading missing".into()))?;
        let circles = agents::circles(o, world.positions()[id], heading)?;
        let mut extent = 0.0_f64;
        for circle in circles.iter().filter(|c| c.radius > 0.0) {
            if circle.center.x - circle.radius < -1e-8
                || circle.center.y - circle.radius < -1e-8
                || circle.center.x + circle.radius > width + 1e-8
                || circle.center.y + circle.radius > height + 1e-8
            {
                return Err(AccountingError::Invariant(
                    "developed anatomy crosses a habitat wall".into(),
                ));
            }
            extent = extent.max(distance(circle.center, world.positions()[id]) + circle.radius);
        }
        extents.insert(*id, extent);
        largest = largest.max(extent);
        shapes.insert(*id, circles);
    }
    let index = crate::spatial::SpatialIndex::new(
        world.grid(),
        world.positions(),
        world.organisms().iter(),
    )?;
    for (id, circles) in &shapes {
        for other in index.within(
            world.grid(),
            world.positions(),
            world.positions()[id],
            extents[id] + largest,
        )? {
            if other <= *id {
                continue;
            }
            if let Some(other_circles) = shapes.get(&other) {
                for a in circles {
                    for b in other_circles {
                        if a.radius > 0.0
                            && b.radius > 0.0
                            && distance(a.center, b.center) + 1e-8 < a.radius + b.radius
                        {
                            return Err(AccountingError::Invariant(
                                "living compound bodies overlap".into(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
