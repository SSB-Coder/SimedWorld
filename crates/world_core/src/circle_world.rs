use crate::{
    agents,
    environment::Position,
    evolution::{PopulationWindow, expend},
    habitat::WindowReport,
    ids::OrganismId,
    physiology,
    resources::{AccountingError, Result},
    spatial::distance,
    transactions::Draft,
};

impl PopulationWindow<'_> {
    pub(crate) fn growth_position(
        &self,
        draft: &Draft<'_>,
        candidate: &crate::organism::Organism,
    ) -> Result<Option<Position>> {
        let id = candidate.id;
        let origin = self.positions[&id];
        let heading = self.ecology.headings[&id];
        if crate::evolution::clear_at(
            draft,
            self.grid,
            self.positions,
            &self.ecology.headings,
            candidate,
            origin,
            heading,
        )? {
            return Ok(Some(origin));
        }
        let old = agents::circles(draft.get(id)?, origin, heading)?[0].radius;
        let radius = agents::circles(candidate, origin, heading)?[0].radius;
        let allowance = (radius - old).max(0.0);
        if allowance <= 1e-12 {
            return Ok(None);
        }
        let (width, height, _, _) = self.grid.dimensions();
        if radius * 2.0 > width || radius * 2.0 > height {
            return Ok(None);
        }
        let mut position = origin;
        for _ in 0..8 {
            position.x = position.x.clamp(radius, width - radius);
            position.y = position.y.clamp(radius, height - radius);
            for (other, o) in draft
                .organisms()
                .filter(|(other, o)| **other != id && o.alive)
            {
                for shape in
                    agents::circles(o, self.positions[other], self.ecology.headings[other])?
                {
                    let dx = position.x - shape.center.x;
                    let dy = position.y - shape.center.y;
                    let distance = dx.hypot(dy);
                    let gap = radius + shape.radius - distance;
                    if gap > 0.0 && distance > 1e-12 {
                        position.x += dx / distance * gap;
                        position.y += dy / distance * gap;
                    }
                }
            }
            if distance(origin, position) > allowance + 1e-9 {
                return Ok(None);
            }
            if crate::evolution::clear_at(
                draft,
                self.grid,
                self.positions,
                &self.ecology.headings,
                candidate,
                position,
                heading,
            )? {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }

    pub(crate) fn move_circle(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        let o = draft.get(id)?;
        if o.brain().is_none() {
            return Ok(());
        }
        let g = o
            .genome
            .body
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("circle body missing".into()))?;
        let traits = g
            .circle
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("circle traits missing".into()))?;
        let adaptation = crate::food_web::adaptations(o);
        if adaptation.is_some_and(|a| a.anchored) {
            return Ok(());
        }
        let root = g.modules[0].id;
        let mut thrust = self.action(o, root, 110);
        let mut turn = self.action(o, root, 111);
        let old_heading = self.ecology.headings[&id];
        let origin = self.positions[&id];
        let radius = agents::circles(o, origin, old_heading)?[0].radius;
        let terrain = &self.config.evolution.as_ref().expect("evolution").terrain;
        let burst = self.action(o, root, 109) * adaptation.map_or(0.0, |a| a.burst);
        let conceal = if self
            .config
            .evolution
            .as_ref()
            .expect("evolution")
            .allow_camouflage
        {
            self.action(o, root, 123) * adaptation.map_or(0.0, |a| a.camouflage)
        } else {
            0.0
        };
        let trapped = self
            .ecology
            .interactions
            .filaments
            .iter()
            .filter(|t| t.captured == Some(id) && t.strength > 0.0)
            .map(|t| t.strength)
            .sum::<f64>();
        let vigor = if self
            .config
            .evolution
            .as_ref()
            .expect("evolution")
            .energy_budgeting
        {
            o.body
                .as_ref()
                .map_or(1.0, |body| body.functional(g, root).sqrt())
        } else {
            1.0
        };
        let speed = traits.speed
            * vigor
            * self.config.ecology.consumer_speed
            * (1.0 + burst * 2.0)
            * (1.0 - conceal * 0.8)
            / crate::terrain::drag(terrain, origin)
            / (1.0 + trapped * 3.0);
        let available = if self
            .config
            .evolution
            .as_ref()
            .expect("evolution")
            .energy_budgeting
        {
            self.action_energy(o, draft.rules)
        } else {
            (o.reserves.energy
                - o.tissue.nutrient
                    * draft.rules.maintenance_per_tissue_nutrient_per_time
                    * self.duration)
                .max(0.0)
        };
        let requested_effort = (thrust + turn.abs()) * (0.002 + burst * 0.04) * self.duration;
        if requested_effort > available {
            let scale = available / requested_effort;
            thrust *= scale;
            turn *= scale;
        }
        let heading = old_heading + turn * self.config.ecology.turn_rate_per_time * self.duration;
        let requested = speed * thrust * self.duration;
        self.diagnostics
            .behavior
            .entry(id)
            .or_default()
            .requested_distance = requested;
        let effort =
            ((thrust + turn.abs()) * (0.002 + burst * 0.04) * self.duration).min(available);
        let travel = requested.min(
            (available - effort).max(0.0)
                / self.config.ecology.movement_energy_per_distance.max(1e-12),
        );
        let vector = (heading.cos() * travel, heading.sin() * travel);
        let mut fraction = 1.0_f64;
        let (width, height, _, _) = self.grid.dimensions();
        for (p, v, limit) in [(origin.x, vector.0, width), (origin.y, vector.1, height)] {
            if v > 0.0 {
                fraction = fraction.min(((limit - radius - p) / v).max(0.0));
            }
            if v < 0.0 {
                fraction = fraction.min(((radius - p) / v).max(0.0));
            }
        }
        let length2 = vector.0 * vector.0 + vector.1 * vector.1;
        if length2 > 0.0 {
            let candidates = self
                .spatial
                .as_ref()
                .expect("window spatial index")
                .within(
                    self.grid,
                    self.positions,
                    origin,
                    travel + radius + self.largest_extent,
                )?;
            for other in candidates.iter().filter(|other| **other != id) {
                let target = draft.get(*other)?;
                if !target.alive {
                    continue;
                }
                for circle in
                    agents::circles(target, self.positions[other], self.ecology.headings[other])?
                {
                    let p = (origin.x - circle.center.x, origin.y - circle.center.y);
                    let b = p.0 * vector.0 + p.1 * vector.1;
                    if b >= 0.0 {
                        continue;
                    }
                    let c = p.0 * p.0 + p.1 * p.1 - (radius + circle.radius).powi(2);
                    let discriminant = b * b - length2 * c;
                    if discriminant >= 0.0 {
                        let contact = (-b - discriminant.sqrt()) / length2;
                        if contact <= 1.0 {
                            fraction = fraction.min(contact.max(0.0));
                        }
                    }
                }
            }
        }
        for rock in terrain
            .iter()
            .filter(|p| p.kind == crate::terrain::TerrainKind::Rock)
        {
            let p = (origin.x - rock.center.x, origin.y - rock.center.y);
            let b = p.0 * vector.0 + p.1 * vector.1;
            let c = p.0 * p.0 + p.1 * p.1 - (radius + rock.radius).powi(2);
            let discriminant = b * b - length2 * c;
            if length2 > 0.0 && b < 0.0 && discriminant >= 0.0 {
                fraction = fraction.min(((-b - discriminant.sqrt()) / length2).max(0.0));
            }
        }
        fraction = fraction.clamp(0.0, 1.0);
        let destination = Position {
            x: origin.x + vector.0 * fraction,
            y: origin.y + vector.1 * fraction,
        };
        let traveled = distance(origin, destination);
        let cost = effort + traveled * self.config.ecology.movement_energy_per_distance;
        if cost > 0.0 {
            expend(draft, id, cost.min(available))?;
            self.diagnostics
                .agent_cost(id, "movement", cost.min(available))?;
        }
        self.positions.insert(id, destination);
        self.ecology
            .headings
            .insert(id, heading.sin().atan2(heading.cos()));
        let blocked = requested > 1e-8 && fraction < 0.99999;
        if blocked {
            crate::diagnostics::increment(&mut self.diagnostics.blocked_moves, 1)?;
        }
        if let Some(behavior) = self.diagnostics.behavior.get_mut(&id) {
            behavior.distance = traveled;
            behavior.executed.insert(110, traveled);
            behavior.executed.insert(
                111,
                (turn * self.config.ecology.turn_rate_per_time * self.duration).abs(),
            );
            behavior.blocked = blocked;
        }
        report.ecology.distance_moved += traveled;
        Ok(())
    }

    pub(crate) fn feed_circle(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        self.hunt_circle(draft, id)?;
        self.graze_circle(draft, id, report)
    }

    pub(crate) fn graze_circle(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
        if !draft.get(id)?.alive {
            return Ok(());
        }
        if draft.get(id)?.genome.body.as_ref().is_some_and(|b| {
            matches!(
                b.diet,
                world_genetics::body::Diet::Scavenger | world_genetics::body::Diet::Recycler
            )
        }) && self.feed_detritus(draft, id, report)?
        {
            return Ok(());
        }
        let o = draft.get(id)?;
        if !o.genome.can_ingest() {
            return Ok(());
        }
        let root = o
            .genome
            .body
            .as_ref()
            .ok_or_else(|| AccountingError::Invariant("circle body missing".into()))?
            .modules[0]
            .id;
        let effort = self.action(o, root, 120);
        if effort <= 0.0 {
            return Ok(());
        }
        if self.diagnostics.behavior.get(&id).is_some_and(|b| {
            b.ingested >= draft.rules.max_ingestion_nutrient_per_time * self.duration * effort
        }) {
            return Ok(());
        }
        let center = self.positions[&id];
        let radius = agents::circles(o, center, self.ecology.headings[&id])?[0].radius;
        if let Some((target, module, _)) = self.contact(draft, id, center, radius + 0.08, true)? {
            self.diagnostics.behavior.entry(id).or_default().target = Some(target);
            let before = draft.get(target)?.clone();
            let consumer = draft.get(id)?;
            let destination = crate::food_web::guild(consumer);
            let source = crate::food_web::guild(&before);
            let level = if before.genome.photosynthetic() {
                1.0
            } else {
                self.ecology
                    .interactions
                    .food_web
                    .trophic
                    .get(&target)
                    .copied()
                    .unwrap_or(2.0)
            };
            let residual = if before.genome.photosynthetic()
                && consumer
                    .genome
                    .body
                    .as_ref()
                    .is_some_and(|g| g.diet == world_genetics::body::Diet::Grazer)
            {
                before.body.as_ref().map_or(0.0, |body| body.developed())
                    * self.config.ecology.grazing_residual_fraction
            } else {
                0.0
            };
            let capacity = consumer.reserve_capacity(draft.rules)?;
            let energy_yield = draft.rules.tissue_energy_per_nutrient
                * draft.rules.energy_assimilation_fraction
                - draft.rules.digestion_energy_per_ingested_nutrient;
            let energy_need = if energy_yield > 0.0 {
                (capacity.energy - consumer.reserves.energy).max(0.0) / energy_yield
            } else {
                0.0
            };
            let nutrient_need = if draft.rules.nutrient_assimilation_fraction > 0.0 {
                (capacity.nutrient - consumer.reserves.nutrient).max(0.0)
                    / draft.rules.nutrient_assimilation_fraction
            } else {
                0.0
            };
            let food_funded = self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .energy_budgeting
                && energy_yield >= 0.0;
            let digestion_energy = if food_funded {
                f64::INFINITY
            } else if self
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .energy_budgeting
                && energy_yield < 0.0
            {
                self.action_energy(consumer, draft.rules)
            } else {
                consumer.reserves.energy
            };
            let already = self
                .diagnostics
                .behavior
                .get(&id)
                .map_or(0.0, |b| b.ingested);
            let mut amount = (draft.rules.max_ingestion_nutrient_per_time * self.duration * effort
                - already)
                .max(0.0)
                .min(energy_need.max(nutrient_need))
                .min((before.tissue.nutrient - residual).max(0.0))
                .min(digestion_energy / draft.rules.digestion_energy_per_ingested_nutrient);
            if !food_funded
                && amount * draft.rules.digestion_energy_per_ingested_nutrient
                    > draft.get(id)?.reserves.energy
            {
                amount = amount.next_down().max(0.0);
            }
            if amount <= 0.0 {
                self.diagnostics
                    .behavior
                    .entry(id)
                    .or_default()
                    .failures
                    .insert("feed_capacity_or_energy".into(), 1);
                return Ok(());
            }
            let cell = self.grid.cell_index(center)?;
            let target_cell = self.grid.cell_index(self.positions[&target])?;
            self.grid.cells[cell].load_pools(&mut draft.pools);
            let stored_before = draft.get(id)?.reserves.energy;
            physiology::feed_funded(
                draft,
                id,
                target,
                amount,
                self.duration,
                Some(module),
                food_funded,
            )?;
            let digestion = amount * draft.rules.digestion_energy_per_ingested_nutrient;
            let assimilated = amount
                * draft.rules.tissue_energy_per_nutrient
                * draft.rules.energy_assimilation_fraction;
            let retained = (draft.get(id)?.reserves.energy - stored_before + digestion)
                .max(0.0)
                .min(assimilated);
            let behavior = self.diagnostics.behavior.entry(id).or_default();
            behavior.assimilated_energy += assimilated;
            behavior.retained_food_energy += retained;
            let injury = self.diagnostics.behavior.entry(target).or_default();
            injury.attacker = Some(id);
            injury.attacker_tick = self.tick;
            injury.damage_received += amount;
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .damage_dealt += amount;
            if cell != target_cell && draft.get(target)?.alive {
                let released =
                    (before.reserves.nutrient - draft.get(target)?.reserves.nutrient).max(0.0);
                draft.pools.mineral_nutrient -= released;
                self.grid.cells[target_cell].mineral_nutrient += released;
            }
            self.grid.cells[cell].store_pools(draft.pools);
            crate::diagnostics::increment(&mut self.diagnostics.encounters, 1)?;
            self.diagnostics.behavior.entry(id).or_default().ingested += amount;
            *self
                .diagnostics
                .behavior
                .entry(id)
                .or_default()
                .executed
                .entry(120)
                .or_default() += amount;
            self.diagnostics.agent_cost(
                id,
                "digestion",
                amount * draft.rules.digestion_energy_per_ingested_nutrient,
            )?;
            self.ecology.interactions.food_web.record_energy(
                id,
                amount
                    * draft.rules.tissue_energy_per_nutrient
                    * draft.rules.energy_assimilation_fraction,
                amount * draft.rules.digestion_energy_per_ingested_nutrient,
                retained,
            );
            self.ecology
                .interactions
                .food_web
                .record(id, source, destination, level, amount);
            report.ecology.feeding_events += 1;
            report.ecology.ingested_nutrient += amount;
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
        } else if crate::food_web::adaptations(draft.get(id)?).is_some()
            && !self.feed_detritus(draft, id, report)?
        {
            self.diagnostics
                .behavior
                .entry(id)
                .or_default()
                .failures
                .insert("feed_no_contact".into(), 1);
        }
        Ok(())
    }
}
