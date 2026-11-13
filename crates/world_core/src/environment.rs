use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    audit::Sum,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    transactions::Pools,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellOverride {
    pub detritus: Option<Resources>,
    pub column: u64,
    pub row: u64,
    pub mineral_nutrient: Option<f64>,
    pub light_energy_per_time: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub(crate) corpses: std::collections::BTreeMap<crate::ids::OrganismId, crate::corpses::Corpse>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub(crate) filaments: std::collections::BTreeMap<crate::ids::OrganismId, Resources>,
    pub(crate) mineral_nutrient: f64,
    pub(crate) detritus: Resources,
    pub(crate) light_available: f64,
    pub(crate) light_energy_per_time: f64,
}

impl Cell {
    pub fn mineral_nutrient(&self) -> f64 {
        self.mineral_nutrient
    }
    pub fn detritus(&self) -> Resources {
        self.detritus
    }
    pub fn light_available(&self) -> f64 {
        self.light_available
    }
    pub(crate) fn load_pools(&self, pools: &mut Pools) {
        pools.mineral_nutrient = self.mineral_nutrient;
        pools.detritus = self.detritus;
        pools.light_available = self.light_available;
    }

    pub(crate) fn store_pools(&mut self, pools: Pools) {
        self.mineral_nutrient = pools.mineral_nutrient;
        self.detritus = pools.detritus;
        self.light_available = pools.light_available;
    }
}

pub(crate) struct CellTotals {
    pub mineral: f64,
    pub detritus: Resources,
    pub light: f64,
    pub power: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentGrid {
    width: f64,
    height: f64,
    columns: usize,
    rows: usize,
    pub(crate) cells: Vec<Cell>,
}

impl EnvironmentGrid {
    pub(crate) fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.width != rules.world_width
            || self.height != rules.world_height
            || self.columns as u64 != rules.grid_width
            || self.rows as u64 != rules.grid_height
            || self.columns.checked_mul(self.rows) != Some(self.cells.len())
        {
            return Err(AccountingError::Invariant(
                "grid geometry does not match rules".into(),
            ));
        }
        self.totals()?;
        Ok(())
    }

    pub fn dimensions(&self) -> (f64, f64, usize, usize) {
        (self.width, self.height, self.columns, self.rows)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub(crate) fn nearby_cells(&self, at: Position, range: f64) -> impl Iterator<Item = usize> {
        let dx = self.width / self.columns as f64;
        let dy = self.height / self.rows as f64;
        let low_x = ((at.x - range).max(0.0) / dx) as usize;
        let high_x = (((at.x + range) / dx) as usize).min(self.columns - 1);
        let low_y = ((at.y - range).max(0.0) / dy) as usize;
        let high_y = (((at.y + range) / dy) as usize).min(self.rows - 1);
        (low_y..=high_y).flat_map(move |y| (low_x..=high_x).map(move |x| y * self.columns + x))
    }

    pub(crate) fn food_position(&self, index: usize, at: Position) -> Position {
        let dx = self.width / self.columns as f64;
        let dy = self.height / self.rows as f64;
        let x = (index % self.columns) as f64 * dx;
        let y = (index / self.columns) as f64 * dy;
        Position {
            x: at.x.clamp(x, x + dx),
            y: at.y.clamp(y, y + dy),
        }
    }

    pub(crate) fn diffuse_minerals(&mut self, coefficient: f64, duration: f64) -> Result<()> {
        finite(coefficient, "mineral diffusivity", false)?;
        finite(duration, "diffusion duration", true)?;
        if coefficient == 0.0 {
            return Ok(());
        }
        let dx = self.width / self.columns as f64;
        let dy = self.height / self.rows as f64;
        let x = coefficient * duration / (dx * dx);
        let y = coefficient * duration / (dy * dy);
        let substeps = (2.0 * (x + y) / 0.9).ceil().max(1.0);
        if !substeps.is_finite() || substeps > 10000.0 {
            return Err(AccountingError::Invalid(
                "diffusion requires too many stable substeps".into(),
            ));
        }
        let mut delta = vec![0.0; self.cells.len()];
        for _ in 0..substeps as usize {
            delta.fill(0.0);
            for i in 0..self.cells.len() {
                for (neighbor, rate) in [
                    (
                        if i % self.columns + 1 < self.columns {
                            Some(i + 1)
                        } else {
                            None
                        },
                        x,
                    ),
                    (
                        if i / self.columns + 1 < self.rows {
                            Some(i + self.columns)
                        } else {
                            None
                        },
                        y,
                    ),
                ] {
                    if let Some(j) = neighbor {
                        let flux = (self.cells[i].mineral_nutrient
                            - self.cells[j].mineral_nutrient)
                            * rate
                            / substeps;
                        delta[i] -= flux;
                        delta[j] += flux;
                    }
                }
            }
            for (cell, change) in self.cells.iter_mut().zip(&delta) {
                cell.mineral_nutrient += change;
                finite(cell.mineral_nutrient, "diffused minerals", false)?;
            }
        }
        Ok(())
    }

    pub fn new(
        rules: &WorldRules,
        minerals: f64,
        light_power: f64,
        light_gradient: f64,
        overrides: &[CellOverride],
    ) -> Result<Self> {
        rules.validate()?;
        finite(minerals, "initial cell minerals", false)?;
        finite(light_power, "cell light power", false)?;
        finite(light_gradient, "light gradient", false)?;
        if light_gradient > 1.0 {
            return Err(AccountingError::Invalid(
                "light gradient must be in [0, 1]".into(),
            ));
        }
        let columns = usize::try_from(rules.grid_width)
            .map_err(|_| AccountingError::Invalid("grid width exceeds platform capacity".into()))?;
        let rows = usize::try_from(rules.grid_height).map_err(|_| {
            AccountingError::Invalid("grid height exceeds platform capacity".into())
        })?;
        let count = columns
            .checked_mul(rows)
            .ok_or_else(|| AccountingError::Invalid("grid cell count overflow".into()))?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(count)
            .map_err(|error| AccountingError::Invalid(error.to_string()))?;
        for index in 0..count {
            let fraction = ((index % columns) as f64 + 0.5) / columns as f64;
            cells.push(Cell {
                corpses: std::collections::BTreeMap::new(),
                mineral_nutrient: minerals,
                detritus: Resources::default(),
                filaments: std::collections::BTreeMap::new(),
                light_available: 0.0,
                light_energy_per_time: light_power
                    * (1.0 - light_gradient + light_gradient * fraction),
            });
        }
        let mut seen = BTreeSet::new();
        for patch in overrides {
            if patch.column >= rules.grid_width || patch.row >= rules.grid_height {
                return Err(AccountingError::Invalid(
                    "cell override is outside the grid".into(),
                ));
            }
            let index = patch.row as usize * columns + patch.column as usize;
            if !seen.insert(index)
                || (patch.mineral_nutrient.is_none()
                    && patch.light_energy_per_time.is_none()
                    && patch.detritus.is_none())
            {
                return Err(AccountingError::Invalid(
                    "cell overrides must be unique and contain a value".into(),
                ));
            }
            if let Some(value) = patch.mineral_nutrient {
                finite(value, "cell override minerals", false)?;
                cells[index].mineral_nutrient = value;
            }
            if let Some(value) = patch.detritus {
                value.validate()?;
                cells[index].detritus = value;
            }
            if let Some(value) = patch.light_energy_per_time {
                finite(value, "cell override light power", false)?;
                cells[index].light_energy_per_time = value;
            }
        }
        let grid = Self {
            width: rules.world_width,
            height: rules.world_height,
            columns,
            rows,
            cells,
        };
        grid.totals()?;
        Ok(grid)
    }

    pub fn cell_index(&self, position: Position) -> Result<usize> {
        finite(position.x, "position x", false)?;
        finite(position.y, "position y", false)?;
        if position.x > self.width || position.y > self.height {
            return Err(AccountingError::Invalid(
                "position is outside the habitat".into(),
            ));
        }
        let column =
            ((position.x / self.width * self.columns as f64) as usize).min(self.columns - 1);
        let row = ((position.y / self.height * self.rows as f64) as usize).min(self.rows - 1);
        Ok(row * self.columns + column)
    }

    pub(crate) fn totals(&self) -> Result<CellTotals> {
        let mut sums: [Sum; 5] = std::array::from_fn(|_| Sum::default());
        for cell in &self.cells {
            cell.detritus.validate()?;
            for corpse in cell.corpses.values() {
                corpse.validate()?;
                sums[1].add(corpse.remaining.nutrient);
                sums[2].add(corpse.remaining.energy);
            }
            for material in cell.filaments.values() {
                material.validate()?;
                sums[1].add(material.nutrient);
                sums[2].add(material.energy);
            }
            for (sum, value) in sums.iter_mut().zip([
                cell.mineral_nutrient,
                cell.detritus.nutrient,
                cell.detritus.energy,
                cell.light_available,
                cell.light_energy_per_time,
            ]) {
                finite(value, "cell resource", false)?;
                sum.add(value);
            }
        }
        let values = sums.map(|sum| sum.value());
        for value in values {
            finite(value, "grid total", false)?;
        }
        Ok(CellTotals {
            mineral: values[0],
            detritus: Resources::new(values[1], values[2])?,
            light: values[3],
            power: values[4],
        })
    }

    pub(crate) fn write_totals(&self, pools: &mut Pools) -> Result<()> {
        let totals = self.totals()?;
        pools.mineral_nutrient = totals.mineral;
        pools.detritus = totals.detritus;
        pools.light_available = totals.light;
        Ok(())
    }
}

#[cfg(test)]
mod diffusion_tests {
    use super::*;

    #[test]
    fn diffusion_is_conservative_positive_and_has_closed_boundaries() {
        let mut rules: WorldRules =
            serde_json::from_str(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        rules.world_width = 3.0;
        rules.world_height = 2.0;
        rules.grid_width = 3;
        rules.grid_height = 2;
        let mut grid = EnvironmentGrid::new(&rules, 0.0, 0.0, 0.0, &[]).unwrap();
        grid.cells[0].mineral_nutrient = 9.0;
        grid.diffuse_minerals(0.25, 0.1).unwrap();
        assert!(grid.cells[1].mineral_nutrient > 0.0 && grid.cells[3].mineral_nutrient > 0.0);
        assert_eq!(grid.cells[2].mineral_nutrient, 0.0);
        assert!((grid.totals().unwrap().mineral - 9.0).abs() < 1e-10);
        grid.diffuse_minerals(5.0, 10.0).unwrap();
        assert!(grid.cells.iter().all(|c| c.mineral_nutrient >= 0.0));
        assert!((grid.totals().unwrap().mineral - 9.0).abs() < 1e-10);
    }
}
