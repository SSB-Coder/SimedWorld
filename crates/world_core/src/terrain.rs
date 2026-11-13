use crate::{
    environment::Position,
    resources::{AccountingError, Result},
    spatial::{distance, segment_distance},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainKind {
    Rock,
    Vegetation,
    Water,
    Shade,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainPatch {
    pub center: Position,
    pub radius: f64,
    pub kind: TerrainKind,
    pub density: f64,
}

impl TerrainPatch {
    pub fn validate(&self, width: f64, height: f64) -> Result<()> {
        if ![self.center.x, self.center.y, self.radius, self.density]
            .iter()
            .all(|v| v.is_finite())
            || !(0.0..=width).contains(&self.center.x)
            || !(0.0..=height).contains(&self.center.y)
            || self.radius <= 0.0
            || self.radius > width.max(height)
            || !(0.0..=1.0).contains(&self.density)
        {
            return Err(AccountingError::Invalid("invalid terrain patch".into()));
        }
        Ok(())
    }
}

pub fn cover(patches: &[TerrainPatch], position: Position) -> f64 {
    patches
        .iter()
        .filter(|p| {
            matches!(p.kind, TerrainKind::Vegetation | TerrainKind::Shade)
                && distance(p.center, position) < p.radius
        })
        .map(|p| p.density)
        .fold(0.0, f64::max)
}

pub fn background(patches: &[TerrainPatch], position: Position) -> f64 {
    if patches
        .iter()
        .any(|p| p.kind == TerrainKind::Water && distance(p.center, position) < p.radius)
    {
        0.85
    } else {
        0.25 + 0.3 * cover(patches, position)
    }
}

pub fn visible(patches: &[TerrainPatch], from: Position, to: Position) -> bool {
    !patches
        .iter()
        .any(|p| p.kind == TerrainKind::Rock && segment_distance(p.center, from, to) < p.radius)
}

pub fn drag(patches: &[TerrainPatch], at: Position) -> f64 {
    1.0 + patches
        .iter()
        .filter(|p| distance(p.center, at) < p.radius)
        .map(|p| match p.kind {
            TerrainKind::Water => 2.0 * p.density,
            TerrainKind::Vegetation => 0.7 * p.density,
            _ => 0.0,
        })
        .sum::<f64>()
}
