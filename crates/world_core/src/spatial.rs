use std::collections::BTreeMap;

use crate::{
    environment::{EnvironmentGrid, Position},
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Result, finite},
};

pub(crate) struct SpatialIndex {
    buckets: Vec<Vec<OrganismId>>,
}

impl SpatialIndex {
    pub fn new<'a>(
        grid: &EnvironmentGrid,
        positions: &BTreeMap<OrganismId, Position>,
        organisms: impl Iterator<Item = (&'a OrganismId, &'a Organism)>,
    ) -> Result<Self> {
        let mut result = Self {
            buckets: vec![Vec::new(); grid.cells().len()],
        };
        for (id, _) in organisms.filter(|(_, o)| o.alive) {
            let position = positions.get(id).ok_or_else(|| {
                AccountingError::Invariant("spatial identity has no position".into())
            })?;
            result.insert(grid, *id, *position)?;
        }
        Ok(result)
    }

    pub fn insert(
        &mut self,
        grid: &EnvironmentGrid,
        id: OrganismId,
        position: Position,
    ) -> Result<()> {
        self.buckets[grid.cell_index(position)?].push(id);
        Ok(())
    }

    pub fn relocate(
        &mut self,
        grid: &EnvironmentGrid,
        id: OrganismId,
        from: Position,
        to: Position,
    ) -> Result<()> {
        let old = grid.cell_index(from)?;
        self.buckets[old].retain(|item| *item != id);
        self.insert(grid, id, to)
    }

    pub fn within(
        &self,
        grid: &EnvironmentGrid,
        positions: &BTreeMap<OrganismId, Position>,
        center: Position,
        radius: f64,
    ) -> Result<Vec<OrganismId>> {
        finite(radius, "query radius", false)?;
        let (width, height, columns, _) = grid.dimensions();
        let low = grid.cell_index(Position {
            x: (center.x - radius).max(0.0),
            y: (center.y - radius).max(0.0),
        })?;
        let high = grid.cell_index(Position {
            x: (center.x + radius).min(width),
            y: (center.y + radius).min(height),
        })?;
        let mut result = Vec::new();
        for row in low / columns..=high / columns {
            for column in low % columns..=high % columns {
                for id in &self.buckets[row * columns + column] {
                    let position = positions.get(id).ok_or_else(|| {
                        AccountingError::Invariant("query identity has no position".into())
                    })?;
                    if distance(center, *position) <= radius {
                        result.push(*id);
                    }
                }
            }
        }
        result.sort_unstable();
        Ok(result)
    }
}

pub(crate) fn distance(a: Position, b: Position) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

pub(crate) fn segment_distance(point: Position, from: Position, to: Position) -> f64 {
    let length = distance(from, to);
    if length == 0.0 {
        return distance(point, from);
    }
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let fraction = (((point.x - from.x) * (dx / length) + (point.y - from.y) * (dy / length))
        / length)
        .clamp(0.0, 1.0);
    distance(
        point,
        Position {
            x: from.x + dx * fraction,
            y: from.y + dy * fraction,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::WorldRules;

    #[test]
    fn swept_distance_covers_interior_endpoints_and_stationary_paths() {
        let from = Position { x: 0.0, y: 0.0 };
        let to = Position { x: 4.0, y: 0.0 };
        assert_eq!(segment_distance(Position { x: 2.0, y: 0.0 }, from, to), 0.0);
        assert_eq!(segment_distance(Position { x: 2.0, y: 1.0 }, from, to), 1.0);
        assert_eq!(segment_distance(Position { x: 5.0, y: 0.0 }, from, to), 1.0);
        assert_eq!(segment_distance(to, from, from), 4.0);
    }

    #[test]
    fn local_queries_cross_cells_and_relocation_removes_old_membership() {
        let mut rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        rules.world_width = 4.0;
        rules.world_height = 2.0;
        rules.grid_width = 4;
        rules.grid_height = 2;
        let grid = EnvironmentGrid::new(&rules, 0.0, 0.0, 0.0, &[]).unwrap();
        let mut positions = BTreeMap::from([
            (OrganismId(1), Position { x: 0.99, y: 0.5 }),
            (OrganismId(2), Position { x: 1.01, y: 0.5 }),
            (OrganismId(3), Position { x: 4.0, y: 2.0 }),
        ]);
        let mut index = SpatialIndex {
            buckets: vec![Vec::new(); 8],
        };
        for (id, position) in &positions {
            index.insert(&grid, *id, *position).unwrap();
        }
        let center = positions[&OrganismId(1)];
        assert_eq!(
            index.within(&grid, &positions, center, 0.03).unwrap(),
            vec![OrganismId(1), OrganismId(2)]
        );
        let moved = Position { x: 3.0, y: 1.5 };
        index
            .relocate(&grid, OrganismId(2), positions[&OrganismId(2)], moved)
            .unwrap();
        positions.insert(OrganismId(2), moved);
        assert_eq!(
            index.within(&grid, &positions, center, 0.03).unwrap(),
            vec![OrganismId(1)]
        );
        assert_eq!(
            index.within(&grid, &positions, moved, 0.0).unwrap(),
            vec![OrganismId(2)]
        );
        assert_eq!(
            index
                .within(&grid, &positions, Position { x: 4.0, y: 2.0 }, 0.0)
                .unwrap(),
            vec![OrganismId(3)]
        );
    }
}
