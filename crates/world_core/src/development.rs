use crate::{
    agents,
    environment::Position,
    evolution::{PopulationWindow, damage_module, expend},
    food_web::{adaptations, guild},
    habitat::WindowReport,
    ids::OrganismId,
    interactions::{Filament, Signal},
    resources::{AccountingError, Resources, Result},
    spatial::{distance, segment_distance},
    terrain::{self, TerrainKind},
    transactions::Draft,
};
use world_genetics::{
    body::Diet,
    neural::{ConnectionGene, NodeGene, NodeKind},
};

impl PopulationWindow<'_> {
    fn retain_filaments(&mut self, keep: impl Fn(&Filament) -> bool) -> Result<()> {
        let traps = std::mem::take(&mut self.ecology.interactions.filaments);
        for trap in traps {
            if keep(&trap) {
                self.ecology.interactions.filaments.push(trap);
            } else {
                let material = self.grid.cells[trap.cell]
                    .filaments
                    .remove(&trap.owner)
                    .ok_or_else(|| {
                        AccountingError::Invariant("missing filament inventory".into())
                    })?;
                self.grid.cells[trap.cell].detritus =
                    self.grid.cells[trap.cell].detritus.plus(material)?;
            }
        }
        Ok(())
    }

    pub(crate) fn prepare_interactions(&mut self, draft: &mut Draft<'_>) -> Result<()> {
        for trap in &mut self.ecology.interactions.filaments {
            trap.remaining -= self.duration;
        }
        self.retain_filaments(|t| {
            t.remaining > 0.0 && t.strength > 0.0 && draft.get(t.owner).is_ok_and(|o| o.alive)
        })?;
        for signal in &mut self.ecology.interactions.signals {
            signal.remaining -= self.duration;
        }
        self.ecology
            .interactions
            .signals
            .retain(|s| s.remaining > 0.0 && draft.get(s.owner).is_ok_and(|o| o.alive));
        let ids: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.alive && adaptations(o).is_some())
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let o = draft.get(id)?;
            let a = adaptations(o).expect("filtered adaptations").clone();
            let root = o.genome.body.as_ref().expect("circle body").modules[0].id;
            let (communication, filaments) = {
                let config = self.config.evolution.as_ref().expect("evolution");
                (config.allow_communication, config.allow_filaments)
            };
            let signal = if communication {
                self.action(o, root, 108) * a.communication
            } else {
                0.0
            };
            let retract = self.action(o, root, 107);
            if retract > 0.5 {
                self.retain_filaments(|t| t.owner != id)?;
            }
            if signal > 0.01 {
                let cost = signal * 0.03 * self.duration;
                if cost <= self.action_energy(draft.get(id)?, draft.rules) {
                    expend(draft, id, cost)?;
                    self.diagnostics.agent_cost(id, "communication", cost)?;
                    self.ecology.interactions.signals.retain(|s| s.owner != id);
                    self.ecology.interactions.signals.push(Signal {
                        owner: id,
                        position: self.positions[&id],
                        value: signal,
                        range: 2.0 + 4.0 * a.communication,
                        remaining: 0.3,
                    });
                }
            }
            let deploy = if filaments {
                self.action(draft.get(id)?, root, 106) * a.filament
            } else {
                0.0
            };
            if deploy > 0.05
                && !self
                    .ecology
                    .interactions
                    .filaments
                    .iter()
                    .any(|t| t.owner == id)
            {
                let amount = 0.03 + 0.07 * deploy;
                let embodied = amount * draft.rules.tissue_energy_per_nutrient;
                let heat = amount * draft.rules.construction_heat_per_nutrient;
                let o = draft.get(id)?;
                if o.reserves.nutrient >= amount
                    && self.action_energy(o, draft.rules) >= embodied + heat + 2.0
                {
                    let mut o = o.clone();
                    o.reserves = o.reserves.minus(Resources::new(amount, embodied + heat)?)?;
                    draft.put(o);
                    draft.pools.heat += heat;
                    let anchor = self.positions[&id];
                    let heading = self.ecology.headings[&id];
                    let end = Position {
                        x: (anchor.x + heading.cos() * (0.5 + 2.0 * deploy))
                            .clamp(0.0, self.config_width()),
                        y: (anchor.y + heading.sin() * (0.5 + 2.0 * deploy))
                            .clamp(0.0, self.config_height()),
                    };
                    let cell = self.grid.cell_index(anchor)?;
                    let material = Resources::new(amount, embodied)?;
                    if self.grid.cells[cell]
                        .filaments
                        .insert(id, material)
                        .is_some()
                    {
                        return Err(AccountingError::Invariant(
                            "duplicate filament inventory".into(),
                        ));
                    }
                    self.ecology.interactions.filaments.push(Filament {
                        owner: id,
                        anchor,
                        end,
                        cell,
                        material,
                        strength: 0.5 + 3.0 * deploy,
                        remaining: 20.0,
                        captured: None,
                    });
                    self.diagnostics
                        .agent_cost(id, "filament_construction", embodied + heat)?;
                    self.event("filament_deployed", id, (None, None), amount, vec![], None)?;
                }
            }
        }
        let living: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.alive)
            .map(|(id, _)| *id)
            .collect();
        for trap in &mut self.ecology.interactions.filaments {
            trap.captured = None;
            for id in &living {
                if *id == trap.owner {
                    continue;
                }
                let target = draft.get(*id)?;
                if !agents::edible(draft.get(trap.owner)?, target) {
                    continue;
                }
                let radius =
                    agents::circles(target, self.positions[id], self.ecology.headings[id])?[0]
                        .radius;
                if segment_distance(self.positions[id], trap.anchor, trap.end) <= radius + 0.025 {
                    trap.captured = Some(*id);
                    break;
                }
            }
        }
        Ok(())
    }

    fn config_width(&self) -> f64 {
        self.grid.dimensions().0
    }
    fn config_height(&self) -> f64 {
        self.grid.dimensions().1
    }

    pub(crate) fn hunt_circle(&mut self, draft: &mut Draft<'_>, id: OrganismId) -> Result<()> {
        let o = draft.get(id)?;
        let Some(a) = adaptations(o).cloned() else {
            return Ok(());
        };
        let root = o.genome.body.as_ref().expect("circle body").modules[0].id;
        let escape = self.action(o, root, 122);
        if escape > 0.0 {
            let cost = escape * 0.2 * self.duration;
            if cost <= self.action_energy(o, draft.rules) {
                expend(draft, id, cost)?;
                self.diagnostics.agent_cost(id, "escape", cost)?;
                for trap in &mut self.ecology.interactions.filaments {
                    if trap.captured == Some(id) {
                        trap.strength -= (0.5 + a.armor) * escape * self.duration;
                    }
                }
                self.retain_filaments(|t| t.strength > 0.0)?;
            }
        }
        let o = draft.get(id)?;
        let attack = self.action(o, root, 121);
        if attack <= 0.0
            || (!o.genome.can_ingest()
                && !self
                    .config
                    .evolution
                    .as_ref()
                    .expect("evolution")
                    .embodied_ecology)
        {
            return Ok(());
        }
        let position = self.positions[&id];
        let radius = agents::circles(o, position, self.ecology.headings[&id])?[0].radius;
        let embodied = self
            .config
            .evolution
            .as_ref()
            .expect("evolution")
            .embodied_ecology;
        let contact = if embodied {
            self.combat_contact(draft, id, position, radius + 0.12)?
        } else {
            self.contact(draft, id, position, radius + 0.12, true)?
        };
        let trapped = self
            .ecology
            .interactions
            .filaments
            .iter()
            .find(|t| t.owner == id && t.strength > 0.0 && distance(position, t.anchor) < 0.8)
            .and_then(|t| t.captured)
            .filter(|target| draft.get(*target).is_ok_and(|o| o.alive));
        let target = contact.map(|t| (t.0, t.1)).or_else(|| {
            trapped.and_then(|t| Some((t, draft.get(t).ok()?.genome.body.as_ref()?.modules[0].id)))
        });
        let Some((target, module)) = target else {
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .failures
                .insert("attack_no_contact".into(), 1);
            return Ok(());
        };
        self.diagnostics.behavior.entry(id).or_default().target = Some(target);
        let capacity = if embodied {
            (o.tissue.nutrient / 6.5).sqrt()
                * (0.25 + 1.5 * a.offense)
                * o.body.as_ref().map_or(1.0, |b| b.integrity(root))
        } else {
            1.0
        };
        let cost = attack
            * capacity.max(0.1)
            * self.duration
            * self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .attack_energy_per_time;
        if cost > self.action_energy(draft.get(id)?, draft.rules) {
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .failures
                .insert("attack_energy".into(), 1);
            return Ok(());
        }
        expend(draft, id, cost)?;
        self.diagnostics.agent_cost(id, "attack", cost)?;
        let protection = adaptations(draft.get(target)?).map_or(0.0, |a| a.armor * 4.0);
        let amount = (attack
            * capacity
            * self.duration
            * self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .attack_nutrient_per_time
            / (1.0 + protection))
            .min(draft.get(target)?.tissue.nutrient);
        if amount > 0.0 {
            let cell = self.grid.cell_index(self.positions[&target])?;
            self.grid.cells[cell].load_pools(&mut draft.pools);
            damage_module(draft, target, module, amount)?;
            self.grid.cells[cell].store_pools(draft.pools);
            let injury = self.diagnostics.behavior.entry(target).or_default();
            injury.attacker = Some(id);
            injury.attacker_tick = self.tick;
            self.diagnostics
                .behavior
                .entry(target)
                .or_default()
                .damage_received += amount;
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .damage_dealt += amount;
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .executed
                .insert(121, amount);
            self.diagnostics.attack_hits += 1;
            self.diagnostics.damage_nutrient += amount;
            self.event(
                "damage",
                id,
                (Some(target), Some(module)),
                amount,
                vec![],
                None,
            )?;
            if !draft.get(target)?.alive {
                self.death(draft, target, Some(id))?;
            }
        }
        Ok(())
    }

    pub(crate) fn feed_detritus(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<bool> {
        let o = draft.get(id)?;
        let Some(body) = &o.genome.body else {
            return Ok(false);
        };
        if !matches!(
            body.diet,
            Diet::Scavenger | Diet::Recycler | Diet::Predator | Diet::Omnivore
        ) {
            return Ok(false);
        }
        let root = body.modules[0].id;
        let effort = self.action(o, root, 120);
        if effort <= 0.0 {
            return Ok(true);
        }
        if self.diagnostics.behavior.get(&id).is_some_and(|b| {
            b.ingested >= draft.rules.max_ingestion_nutrient_per_time * self.duration * effort
        }) {
            return Ok(true);
        }
        let position = self.positions[&id];
        let cell = self.grid.cell_index(position)?;
        let radius = agents::circles(o, position, self.ecology.headings[&id])?[0].radius;
        let recycler = body.diet == Diet::Recycler;
        let investment = adaptations(o).map_or(0.5, |a| a.digestion);
        let embodied = self
            .config
            .evolution
            .as_ref()
            .expect("evolution")
            .embodied_ecology;
        let nutrient_fraction = draft.rules.nutrient_assimilation_fraction
            * if recycler && embodied {
                0.35 + 0.3 * investment
            } else {
                0.7 + 0.3 * investment
            };
        let energy_fraction = crate::food_web::detritus_energy_fraction(o, draft.rules, embodied);
        let selected = self
            .grid
            .corpses()
            .filter(|corpse| {
                corpse.remaining.nutrient > 1e-12
                    && corpse.remaining.energy * energy_fraction
                        > corpse.remaining.nutrient
                            * draft.rules.digestion_energy_per_ingested_nutrient
                    && distance(position, corpse.position) <= radius + corpse.radius() + 0.08
                    && terrain::visible(
                        &self.config.evolution.as_ref().expect("evolution").terrain,
                        position,
                        corpse.position,
                    )
            })
            .min_by(|a, b| {
                distance(position, a.position)
                    .total_cmp(&distance(position, b.position))
                    .then(a.source.cmp(&b.source))
            })
            .map(|corpse| {
                (
                    self.grid
                        .cell_index(corpse.position)
                        .expect("validated corpse"),
                    corpse.source,
                )
            });
        let loose_cell = self
            .grid
            .nearby_cells(position, radius + 0.08)
            .filter(|index| {
                let food = self.grid.cells[*index].detritus;
                let at = self.grid.food_position(*index, position);
                food.nutrient > 1e-12
                    && food.energy * energy_fraction
                        > food.nutrient * draft.rules.digestion_energy_per_ingested_nutrient
                    && distance(position, at) <= radius + 0.08
                    && terrain::visible(
                        &self.config.evolution.as_ref().expect("evolution").terrain,
                        position,
                        at,
                    )
            })
            .min_by(|a, b| {
                distance(position, self.grid.food_position(*a, position))
                    .total_cmp(&distance(position, self.grid.food_position(*b, position)))
                    .then(a.cmp(b))
            });
        let available = selected.map_or(
            self.grid.cells[loose_cell.unwrap_or(cell)].detritus,
            |(index, source)| self.grid.cells[index].corpses[&source].remaining,
        );
        let energy_yield = available.energy / available.nutrient.max(1e-12) * energy_fraction
            - draft.rules.digestion_energy_per_ingested_nutrient;
        if available.nutrient > 1e-12 && energy_yield <= 0.0 {
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .failures
                .insert("feed_energy_poor".into(), 1);
            return Ok(true);
        }
        let capacity = o.reserve_capacity(draft.rules)?;
        let need = ((capacity.nutrient - o.reserves.nutrient) / nutrient_fraction.max(1e-12))
            .max((capacity.energy - o.reserves.energy) / energy_yield.max(1e-12));
        let already = self
            .diagnostics
            .behavior
            .get(&id)
            .map_or(0.0, |b| b.ingested);
        let amount = (draft.rules.max_ingestion_nutrient_per_time * self.duration * effort
            - already)
            .max(0.0)
            .min(available.nutrient)
            .min(need.max(0.0));
        if amount <= 1e-12 {
            let reason = if available.nutrient <= 1e-12 {
                "feed_no_detritus"
            } else if o.reserves.energy < 1e-12 {
                "digestion_energy"
            } else {
                "feed_capacity"
            };
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .failures
                .insert(reason.into(), 1);
            return Ok(true);
        }
        let bite = Resources::new(
            amount,
            available.energy * (amount / available.nutrient).min(1.0),
        )?;
        let cost = amount * draft.rules.digestion_energy_per_ingested_nutrient;
        let assimilated =
            Resources::new(amount * nutrient_fraction, bite.energy * energy_fraction)?;
        let mut consumer = o.clone();
        let stored_before = consumer.reserves.energy;
        consumer.reserves = consumer
            .reserves
            .plus(assimilated)?
            .minus(Resources::new(0.0, cost)?)?;
        if let Some((index, source)) = selected {
            let corpse = self.grid.cells[index]
                .corpses
                .get_mut(&source)
                .expect("selected corpse");
            corpse.remaining = corpse.remaining.minus(bite)?;
            if corpse.remaining == Resources::default() {
                self.grid.cells[index].corpses.remove(&source);
            }
        } else {
            let index = loose_cell.unwrap_or(cell);
            self.grid.cells[index].detritus = self.grid.cells[index].detritus.minus(bite)?;
        }
        self.grid.cells[cell].load_pools(&mut draft.pools);
        if recycler {
            draft.pools.mineral_nutrient += bite.nutrient - assimilated.nutrient;
        } else {
            draft.pools.detritus.nutrient += bite.nutrient - assimilated.nutrient;
        }
        draft.pools.heat += cost + bite.energy - assimilated.energy;
        let reserves = consumer.reserves;
        let destination = guild(&consumer);
        draft.set_reserves(consumer, reserves)?;
        let retained = (draft.get(id)?.reserves.energy - stored_before + cost)
            .max(0.0)
            .min(assimilated.energy);
        let behavior = self.diagnostics.behavior.entry(id).or_default();
        behavior.assimilated_energy += assimilated.energy;
        behavior.retained_food_energy += retained;
        self.grid.cells[cell].store_pools(draft.pools);
        self.ecology.interactions.food_web.record(
            id,
            if selected.is_some() {
                "corpse"
            } else {
                "detritus"
            },
            destination,
            1.0,
            amount,
        );
        self.ecology
            .interactions
            .food_web
            .record_energy(id, assimilated.energy, cost, retained);
        self.diagnostics.agent_cost(id, "digestion", cost)?;
        self.diagnostics.behavior.entry(id).or_default().ingested += amount;
        *self
            .diagnostics
            .behavior
            .entry(id)
            .or_default()
            .executed
            .entry(120)
            .or_default() += amount;
        report.ecology.feeding_events += 1;
        report.ecology.ingested_nutrient += amount;
        self.event(
            if selected.is_some() {
                "corpse_feeding"
            } else {
                "detritus_feeding"
            },
            id,
            (selected.map(|(_, source)| source), None),
            amount,
            vec![],
            Some(destination.into()),
        )?;
        Ok(true)
    }

    pub(crate) fn neural_development(&mut self, draft: &mut Draft<'_>) -> Result<()> {
        let config = self.config.evolution.as_ref().expect("evolution");
        if !config.neural_development
            || !self.tick.is_multiple_of(config.development_interval_ticks)
        {
            return Ok(());
        }
        let ids: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| {
                o.alive && o.brain().is_some() && adaptations(o).is_some_and(|a| a.agency)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let mut o = draft.get(id)?.clone();
            let a = adaptations(&o).expect("adaptations").clone();
            let root = o.genome.body.as_ref().expect("body").modules[0].id;
            let grow = self.action(&o, root, 104) * a.neural_growth;
            let prune = self.action(&o, root, 105) * a.neural_pruning;
            let mut brain = o.brain().expect("brain").clone();
            let mut event = None;
            if grow > 0.1
                && brain.nodes.len() < config.mutation.max_nodes
                && brain.connections.len() + 2 <= config.mutation.max_connections
                && (brain.nodes.len() as f64) < o.tissue.nutrient * 200.0
            {
                let material = if config.embodied_ecology {
                    config.offspring_neuron_nutrient + 2.0 * config.offspring_connection_nutrient
                } else {
                    0.0
                };
                let material_cost = material
                    * (draft.rules.tissue_energy_per_nutrient
                        + draft.rules.construction_heat_per_nutrient);
                let cost = 0.05 + 0.1 * grow;
                let safety = if config.energy_budgeting {
                    crate::evolution::reserve_floor(&o, draft.rules, self.config)
                } else {
                    2.0
                };
                if o.reserves.energy > cost + material_cost + safety
                    && o.reserves.nutrient >= material
                {
                    let edge = brain
                        .connections
                        .get(
                            (self.tick as usize / config.development_interval_ticks as usize
                                + id.0 as usize)
                                % brain.connections.len().max(1),
                        )
                        .cloned();
                    if let Some(edge) = edge {
                        let node = brain.next_node;
                        brain.next_node = node
                            .checked_add(1)
                            .ok_or_else(|| AccountingError::Invalid("neuron id overflow".into()))?;
                        brain.nodes.push(NodeGene {
                            id: node,
                            kind: NodeKind::Hidden,
                            port: None,
                            bias: 0.0,
                            leak: 0.6,
                        });
                        for (source, target, weight) in [
                            (edge.source, node, 1.0),
                            (
                                node,
                                edge.target,
                                if config.embodied_ecology { 0.0 } else { 0.05 },
                            ),
                        ] {
                            let identity = brain.next_connection;
                            brain.next_connection = identity.checked_add(1).ok_or_else(|| {
                                AccountingError::Invalid("connection id overflow".into())
                            })?;
                            brain.connections.push(ConnectionGene {
                                id: identity,
                                source,
                                target,
                                weight,
                            });
                        }
                        if material > 0.0 {
                            o.reserves =
                                o.reserves.minus(Resources::new(material, material_cost)?)?;
                            o.tissue = o.tissue.plus(Resources::new(
                                material,
                                material * draft.rules.tissue_energy_per_nutrient,
                            )?)?;
                            o.body
                                .as_mut()
                                .expect("body")
                                .grow(o.genome.body.as_ref().expect("body"), material)?;
                            let Some(position) = self.growth_position(draft, &o)? else {
                                continue;
                            };
                            let work = distance(self.positions[&id], position)
                                * self.config.ecology.movement_energy_per_distance;
                            let safety = if config.energy_budgeting {
                                let mut candidate = o.clone();
                                candidate.expressed_brain = Some(brain.clone());
                                crate::evolution::reserve_floor(
                                    &candidate,
                                    draft.rules,
                                    self.config,
                                )
                            } else {
                                2.0
                            };
                            if o.reserves.energy < cost + work + safety {
                                continue;
                            }
                            o.reserves.energy -= work;
                            draft.pools.heat +=
                                work + material * draft.rules.construction_heat_per_nutrient;
                            self.positions.insert(id, position);
                            self.diagnostics.agent_cost(
                                id,
                                "neural_material",
                                material_cost + work,
                            )?;
                        }
                        o.reserves.energy -= cost;
                        draft.pools.heat += cost;
                        self.diagnostics
                            .agent_cost(id, "neural_development", cost)?;
                        event = Some("neural_growth");
                    }
                }
            } else if prune > 0.1
                && let Some(node) = brain
                    .nodes
                    .iter()
                    .filter(|n| n.kind == NodeKind::Hidden)
                    .min_by(|a, b| {
                        if !config.embodied_ecology {
                            return b.id.cmp(&a.id);
                        }
                        let strength = |id| {
                            brain
                                .connections
                                .iter()
                                .filter(|e| e.source == id || e.target == id)
                                .map(|e| e.weight.abs())
                                .sum::<f64>()
                        };
                        let score = |id| {
                            self.controls
                                .activity
                                .get(&o.id)
                                .and_then(|u| u.get(&id))
                                .map_or_else(|| strength(id), |v| *v * (0.01 + strength(id)))
                        };
                        score(a.id).total_cmp(&score(b.id)).then(a.id.cmp(&b.id))
                    })
                    .map(|n| n.id)
            {
                brain.nodes.retain(|n| n.id != node);
                brain
                    .connections
                    .retain(|e| e.source != node && e.target != node);
                event = Some("neural_pruning");
            }
            if let Some(kind) = event {
                brain.version = brain
                    .version
                    .checked_add(1)
                    .ok_or_else(|| AccountingError::Invalid("topology version overflow".into()))?;
                o.expressed_brain = Some(brain);
                draft.put(o);
                self.event(kind, id, (None, None), 1.0, vec![], None)?;
            }
        }
        Ok(())
    }
}

pub fn detected(
    habitat: &crate::habitat::Habitat,
    observer: &crate::organism::Organism,
    target: &crate::organism::Organism,
    range: f64,
) -> bool {
    let Some(config) = &habitat.config.evolution else {
        return true;
    };
    let from = habitat.positions[&observer.id];
    let to = habitat.positions[&target.id];
    if !terrain::visible(&config.terrain, from, to) {
        return false;
    }
    let Some(a) = adaptations(target).filter(|_| config.allow_camouflage) else {
        return true;
    };
    let movement = habitat
        .diagnostics
        .as_ref()
        .and_then(|d| d.behavior.get(&target.id))
        .map_or(0.0, |b| (b.distance * 10.0).min(1.0));
    let matching = 1.0 - (a.pigment - terrain::background(&config.terrain, to)).abs();
    let conceal = a.camouflage
        * matching
        * (0.3 + 0.7 * terrain::cover(&config.terrain, to))
        * (1.0 - movement);
    distance(from, to) <= range * (1.0 - 0.85 * conceal)
}

pub fn rock_collision(patches: &[terrain::TerrainPatch], at: Position, radius: f64) -> bool {
    patches
        .iter()
        .any(|p| p.kind == TerrainKind::Rock && distance(at, p.center) < radius + p.radius - 1e-10)
}
