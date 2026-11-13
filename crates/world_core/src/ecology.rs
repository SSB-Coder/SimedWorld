use std::{
    collections::BTreeMap,
    f64::consts::{PI, TAU},
};

use serde::{Deserialize, Serialize};

use crate::{
    environment::{EnvironmentGrid, Position},
    habitat_config::{EcologyConfig, seeded_position},
    ids::OrganismId,
    lifecycle, physiology,
    resources::{AccountingError, Resources, Result, finite},
    spatial::{SpatialIndex, distance, segment_distance},
    transactions::Draft,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EcologyReport {
    #[serde(default)]
    pub producer_litter_nutrient: f64,
    pub consumers_processed: u64,
    pub births: u64,
    pub deaths: u64,
    pub feeding_events: u64,
    pub ingested_nutrient: f64,
    pub tissue_grown: f64,
    pub distance_moved: f64,
    pub decomposed_nutrient: f64,
    pub decomposed_energy: f64,
}

impl EcologyReport {
    pub(crate) fn validate(&self) -> Result<()> {
        for value in [
            self.producer_litter_nutrient,
            self.ingested_nutrient,
            self.tissue_grown,
            self.distance_moved,
            self.decomposed_nutrient,
            self.decomposed_energy,
        ] {
            finite(value, "ecology receipt", false)?;
        }
        Ok(())
    }

    pub(crate) fn accumulate(&mut self, window: &Self) -> Result<()> {
        for (total, increment) in [
            (&mut self.consumers_processed, window.consumers_processed),
            (&mut self.births, window.births),
            (&mut self.deaths, window.deaths),
            (&mut self.feeding_events, window.feeding_events),
        ] {
            *total = total
                .checked_add(increment)
                .ok_or_else(|| AccountingError::Invalid("ecology counter overflow".into()))?;
        }
        self.ingested_nutrient += window.ingested_nutrient;
        self.tissue_grown += window.tissue_grown;
        self.distance_moved += window.distance_moved;
        self.decomposed_nutrient += window.decomposed_nutrient;
        self.decomposed_energy += window.decomposed_energy;
        self.producer_litter_nutrient += window.producer_litter_nutrient;
        self.validate()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EcologyState {
    pub interactions: crate::interactions::InteractionState,
    pub headings: BTreeMap<OrganismId, f64>,
    pub next_id: u64,
    pub totals: EcologyReport,
}

#[derive(Clone, Copy)]
pub(crate) struct EcologyWindow<'a> {
    pub config: &'a EcologyConfig,
    pub duration: f64,
    pub seed: u64,
    pub tick: u64,
}

impl EcologyState {
    pub fn new(next_id: u64) -> Self {
        Self {
            interactions: crate::interactions::InteractionState::default(),
            headings: BTreeMap::new(),
            next_id,
            totals: EcologyReport::default(),
        }
    }

    pub fn recycle(
        &self,
        draft: &mut Draft<'_>,
        grid: &mut EnvironmentGrid,
        config: &EcologyConfig,
        duration: f64,
        report: &mut EcologyReport,
    ) -> Result<()> {
        for cell in &mut grid.cells {
            for corpse in cell.corpses.values_mut() {
                corpse.age += duration;
            }
        }
        if !config.decomposition {
            return Ok(());
        }
        let totals = grid
            .cells
            .iter()
            .try_fold(Resources::default(), |sum, cell| {
                cell.corpses
                    .values()
                    .try_fold(sum.plus(cell.detritus)?, |sum, corpse| {
                        sum.plus(corpse.remaining)
                    })
            })?;
        if totals == Resources::default() {
            return Ok(());
        }
        let mut fraction = (config.decomposition_fraction_per_time * duration).min(1.0);
        if totals.nutrient > 0.0 {
            fraction = fraction
                .min(draft.rules.max_decomposition_nutrient_per_time * duration / totals.nutrient);
        }
        if totals.energy > 0.0 {
            fraction = fraction
                .min(draft.rules.max_decomposition_energy_per_time * duration / totals.energy);
        }
        if fraction <= 0.0 {
            return Ok(());
        }
        // One world-level budget is partitioned across cells, not granted to every cell.
        if fraction * totals.nutrient > draft.rules.max_decomposition_nutrient_per_time * duration
            || fraction * totals.energy > draft.rules.max_decomposition_energy_per_time * duration
        {
            fraction = fraction.next_down().max(0.0);
        }
        for cell in &mut grid.cells {
            for corpse in cell.corpses.values_mut() {
                let decayed = Resources::new(
                    corpse.remaining.nutrient * fraction,
                    corpse.remaining.energy * fraction,
                )?;
                corpse.remaining = corpse.remaining.minus(decayed)?;
                cell.mineral_nutrient += decayed.nutrient;
                draft.pools.heat += decayed.energy;
                report.decomposed_nutrient += decayed.nutrient;
                report.decomposed_energy += decayed.energy;
            }
            cell.corpses
                .retain(|_, corpse| corpse.remaining != Resources::default());
            if cell.detritus == Resources::default() || fraction == 0.0 {
                continue;
            }
            cell.load_pools(&mut draft.pools);
            let before = draft.pools.detritus;
            physiology::decompose(draft, fraction, duration)?;
            report.decomposed_nutrient += before.nutrient - draft.pools.detritus.nutrient;
            report.decomposed_energy += before.energy - draft.pools.detritus.energy;
            cell.store_pools(draft.pools);
        }
        Ok(())
    }

    pub fn consumers(
        &mut self,
        draft: &mut Draft<'_>,
        grid: &mut EnvironmentGrid,
        positions: &mut BTreeMap<OrganismId, Position>,
        window: EcologyWindow<'_>,
        report: &mut EcologyReport,
    ) -> Result<()> {
        let EcologyWindow {
            config,
            duration,
            seed,
            tick,
        } = window;
        let mut index = SpatialIndex::new(grid, positions, draft.organisms())?;
        let ids: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.alive && o.genome.can_ingest() && !o.genome.photosynthetic())
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let origin = positions[&id];
            let sensor_cost = draft.rules.sensor_energy_per_time * duration;
            finite(sensor_cost, "diagnostic sensing cost", true)?;
            let mut target = None;
            if draft.get(id)?.reserves.energy >= sensor_cost {
                grid.cells[grid.cell_index(origin)?].load_pools(&mut draft.pools);
                physiology::sense(draft, id, duration)?;
                let nearby = index.within(grid, positions, origin, config.perception_radius)?;
                target = nearby
                    .into_iter()
                    .filter(|other| *other != id)
                    .filter(|other| {
                        draft
                            .get(*other)
                            .is_ok_and(|o| o.alive && o.genome.photosynthetic())
                    })
                    .min_by(|a, b| {
                        distance(origin, positions[a])
                            .total_cmp(&distance(origin, positions[b]))
                            .then(a.cmp(b))
                    });
                let previous = *self.headings.get(&id).ok_or_else(|| {
                    AccountingError::Invariant("consumer heading is missing".into())
                })?;
                let heading = if let Some(other) = target {
                    let p = positions[&other];
                    (p.y - origin.y).atan2(p.x - origin.x)
                } else {
                    let turn = seeded_position(seed ^ id.0, tick, 1.0, 1.0).x * 0.6 - 0.3;
                    (previous + turn).sin().atan2((previous + turn).cos())
                };
                self.headings.insert(id, heading);
                let available =
                    draft.get(id)?.reserves.energy / config.movement_energy_per_distance;
                let requested =
                    (config.consumer_speed * duration)
                        .min(available)
                        .min(target.map_or(f64::INFINITY, |other| {
                            (distance(origin, positions[&other])
                                - (config.feeding_radius + config.minimum_spacing) * 0.5)
                                .max(0.0)
                        }));
                let (width, height, _, _) = grid.dimensions();
                let to = Position {
                    x: (origin.x + heading.cos() * requested).clamp(0.0, width),
                    y: (origin.y + heading.sin() * requested).clamp(0.0, height),
                };
                let travelled = distance(origin, to);
                if travelled > 0.0
                    && clear_path(
                        draft,
                        grid,
                        positions,
                        &index,
                        id,
                        to,
                        config.minimum_spacing,
                    )?
                {
                    let cost = travelled * config.movement_energy_per_distance;
                    finite(cost, "movement expenditure", true)?;
                    if cost <= draft.get(id)?.reserves.energy {
                        let work = cost * draft.rules.movement_efficiency;
                        physiology::actuate(draft, id, cost)?;
                        if work > 0.0 {
                            physiology::dissipate_motion(draft, work)?;
                        }
                        positions.insert(id, to);
                        index.relocate(grid, id, origin, to)?;
                        report.distance_moved += travelled;
                    }
                }
            }
            let current = positions[&id];
            let consumer_cell = grid.cell_index(current)?;
            grid.cells[consumer_cell].load_pools(&mut draft.pools);
            if let Some(other) = target
                && draft.get(other)?.alive
                && distance(current, positions[&other]) <= config.feeding_radius
            {
                let mut amount = (draft.rules.max_ingestion_nutrient_per_time * duration)
                    .min(draft.get(other)?.tissue.nutrient)
                    .min(
                        draft.get(id)?.reserves.energy
                            / draft.rules.digestion_energy_per_ingested_nutrient,
                    );
                if amount * draft.rules.digestion_energy_per_ingested_nutrient
                    > draft.get(id)?.reserves.energy
                {
                    amount = amount.next_down();
                }
                if amount > 0.0 {
                    let before = draft.get(other)?.clone();
                    physiology::feed(draft, id, other, amount, true, duration)?;
                    let target_cell = grid.cell_index(positions[&other])?;
                    if target_cell != consumer_cell {
                        let after = draft.get(other)?;
                        if !after.alive {
                            let carcass =
                                before.tissue.plus(before.reserves)?.minus(Resources::new(
                                    amount,
                                    amount * draft.rules.tissue_energy_per_nutrient,
                                )?)?;
                            draft.pools.detritus = draft.pools.detritus.minus(carcass)?;
                            grid.cells[target_cell].detritus =
                                grid.cells[target_cell].detritus.plus(carcass)?;
                        } else {
                            let released =
                                (before.reserves.nutrient - after.reserves.nutrient).max(0.0);
                            draft.pools.mineral_nutrient -= released;
                            grid.cells[target_cell].mineral_nutrient += released;
                        }
                    }
                    report.feeding_events += 1;
                    report.ingested_nutrient += amount;
                }
            }
            physiology::maintain(draft, id, duration)?;
            report.consumers_processed += 1;
            if draft.get(id)?.alive && !reserve_for_birth(draft, id, config)? {
                report.tissue_grown += physiology::grow_available(draft, id, duration)?;
            }
            grid.cells[consumer_cell].store_pools(draft.pools);
        }
        Ok(())
    }

    pub fn births(
        &mut self,
        draft: &mut Draft<'_>,
        grid: &EnvironmentGrid,
        positions: &mut BTreeMap<OrganismId, Position>,
        window: EcologyWindow<'_>,
        report: &mut EcologyReport,
    ) -> Result<()> {
        let EcologyWindow {
            config, seed, tick, ..
        } = window;
        if !config.reproduction {
            return Ok(());
        }
        let candidates: Vec<_> = draft
            .organisms()
            .filter(|(_, o)| o.mature(draft.rules) && o.age >= o.next_reproduction_age)
            .map(|(id, _)| *id)
            .collect();
        let mut alive = draft.organisms().filter(|(_, o)| o.alive).count();
        let mut index = SpatialIndex::new(grid, positions, draft.organisms())?;
        let investment = lifecycle::birth_investment(draft.rules)?;
        let (width, height, _, _) = grid.dimensions();
        for parent in candidates {
            if alive >= config.max_population {
                break;
            }
            let reserves = draft.get(parent)?.reserves;
            if reserves.nutrient < investment.nutrient || reserves.energy < investment.energy {
                continue;
            }
            let origin = positions[&parent];
            let phase = seeded_position(seed ^ parent.0, tick, 1.0, 1.0).x * TAU;
            let mut placement = None;
            for attempt in 0..8 {
                let angle = phase + attempt as f64 * TAU / 8.0;
                let candidate = Position {
                    x: origin.x + angle.cos() * config.birth_radius,
                    y: origin.y + angle.sin() * config.birth_radius,
                };
                if candidate.x < 0.0
                    || candidate.y < 0.0
                    || candidate.x > width
                    || candidate.y > height
                {
                    continue;
                }
                let mut clear = true;
                for other in index.within(grid, positions, candidate, config.minimum_spacing)? {
                    if draft.get(other)?.alive
                        && distance(candidate, positions[&other]) < config.minimum_spacing
                    {
                        clear = false;
                        break;
                    }
                }
                if clear {
                    placement = Some(candidate);
                    break;
                }
            }
            if let Some(position) = placement {
                let child = OrganismId(self.next_id);
                let next = self.next_id.checked_add(1).ok_or_else(|| {
                    AccountingError::Invalid("offspring identity overflow".into())
                })?;
                lifecycle::reproduce(draft, parent, child, true)?;
                positions.insert(child, position);
                if draft.get(child)?.genome.can_ingest() {
                    self.headings.insert(child, initial_heading(seed, child));
                }
                index.insert(grid, child, position)?;
                self.next_id = next;
                alive += 1;
                report.births += 1;
            }
        }
        Ok(())
    }
}

pub(crate) fn reserve_for_birth(
    draft: &Draft<'_>,
    id: OrganismId,
    config: &EcologyConfig,
) -> Result<bool> {
    Ok(config.reproduction
        && draft.get(id)?.tissue.nutrient >= draft.rules.maturity_tissue_nutrient)
}

pub(crate) fn initial_heading(seed: u64, id: OrganismId) -> f64 {
    seeded_position(seed, id.0, 1.0, 1.0).x * TAU - PI
}

fn clear_path(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    index: &SpatialIndex,
    id: OrganismId,
    to: Position,
    spacing: f64,
) -> Result<bool> {
    let from = positions[&id];
    let center = Position {
        x: from.x + (to.x - from.x) * 0.5,
        y: from.y + (to.y - from.y) * 0.5,
    };
    for other in index.within(grid, positions, center, distance(from, to) * 0.5 + spacing)? {
        if other != id
            && draft.get(other)?.alive
            && segment_distance(positions[&other], from, to) < spacing
        {
            return Ok(false);
        }
    }
    Ok(true)
}
