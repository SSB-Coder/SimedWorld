use crate::{
    environment::Position,
    resources::{AccountingError, Result, finite},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use world_genetics::body::{BodyGenome, ModuleKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleState {
    pub id: u64,
    pub nutrient: f64,
    pub developed: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyState {
    pub modules: Vec<ModuleState>,
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub id: u64,
    pub kind: ModuleKind,
    pub center: Position,
    pub radius: f64,
    pub nutrient: f64,
    pub integrity: f64,
}

pub fn reserve_capacity(
    nutrient: f64,
    genome: Option<&BodyGenome>,
    body: Option<&BodyState>,
    rules: &crate::rules::WorldRules,
) -> Result<crate::resources::Resources> {
    let storage = genome.zip(body).map_or(0.0, |(g, b)| {
        g.modules
            .iter()
            .filter(|m| m.kind == ModuleKind::Storage)
            .map(|m| {
                b.modules
                    .iter()
                    .find(|s| s.id == m.id)
                    .map_or(0.0, |s| s.nutrient)
                    * m.investment
                    * b.functional(g, m.id)
            })
            .sum::<f64>()
    });
    crate::resources::Resources::new(
        nutrient * rules.reserve_nutrient_capacity_per_tissue_nutrient + storage * 1.5,
        nutrient * rules.reserve_energy_capacity_per_tissue_nutrient + storage * 5.0,
    )
}

impl BodyState {
    pub fn new(genome: &BodyGenome, nutrient: f64) -> Result<Self> {
        finite(nutrient, "developed body tissue", false)?;
        let total = genome.target_nutrient();
        finite(total, "body target tissue", true)?;
        let mut remaining = nutrient;
        let mut modules = Vec::new();
        for (i, m) in genome.modules.iter().enumerate() {
            let share = if i + 1 == genome.modules.len() {
                remaining
            } else {
                (nutrient
                    * (std::f64::consts::PI * m.radius * m.radius * genome.density * 12.0 / total))
                    .min(remaining)
            };
            modules.push(ModuleState {
                id: m.id,
                nutrient: share,
                developed: share,
            });
            remaining -= share;
        }
        Ok(Self { modules })
    }

    pub fn validate(&self, genome: &BodyGenome, nutrient: f64) -> Result<()> {
        if self.modules.len() != genome.modules.len() {
            return Err(AccountingError::Invariant(
                "body module inventory mismatch".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for m in &self.modules {
            finite(m.nutrient, "module tissue", false)?;
            finite(m.developed, "developed tissue", false)?;
            if !ids.insert(m.id)
                || !genome.modules.iter().any(|g| g.id == m.id)
                || m.nutrient > m.developed + 1e-10
            {
                return Err(AccountingError::Invariant("invalid module state".into()));
            }
        }
        if (self.total() - nutrient).abs() > 1e-8 * (1.0 + nutrient.abs()) {
            return Err(AccountingError::Invariant(
                "module totals disagree with organism tissue".into(),
            ));
        }
        let expected = Self::new(genome, self.developed())?;
        for module in &self.modules {
            let target = expected
                .modules
                .iter()
                .find(|m| m.id == module.id)
                .ok_or_else(|| AccountingError::Invariant("developed module missing".into()))?;
            if (module.developed - target.developed).abs() > 1e-8 * (1.0 + self.developed()) {
                return Err(AccountingError::Invariant(
                    "developed geometry and module tissue allocation disagree".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn total(&self) -> f64 {
        self.modules.iter().map(|m| m.nutrient).sum()
    }
    pub fn developed(&self) -> f64 {
        self.modules.iter().map(|m| m.developed).sum()
    }
    pub fn missing(&self) -> f64 {
        self.modules
            .iter()
            .map(|m| (m.developed - m.nutrient).max(0.0))
            .sum()
    }
    pub fn integrity(&self, id: u64) -> f64 {
        self.modules.iter().find(|m| m.id == id).map_or(0.0, |m| {
            if m.developed > 0.0 {
                (m.nutrient / m.developed).clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
    }

    pub fn functional(&self, genome: &BodyGenome, id: u64) -> f64 {
        let mut current = Some(id);
        let mut value = 1.0_f64;
        while let Some(id) = current {
            let Some(gene) = genome.modules.iter().find(|g| g.id == id) else {
                return 0.0;
            };
            let Some(state) = self.modules.iter().find(|m| m.id == id) else {
                return 0.0;
            };
            value = value.min(self.integrity(id));
            if state.nutrient < 0.015 {
                return 0.0;
            }
            current = gene.parent;
        }
        value
    }

    pub fn grow(&mut self, genome: &BodyGenome, amount: f64) -> Result<()> {
        let addition = Self::new(genome, amount)?;
        for m in &mut self.modules {
            let share = addition
                .modules
                .iter()
                .find(|a| a.id == m.id)
                .ok_or_else(|| AccountingError::Invariant("growth module missing".into()))?
                .nutrient;
            m.nutrient += share;
            m.developed += share;
        }
        Ok(())
    }

    pub fn remove(&mut self, amount: f64, module: Option<u64>) -> Result<f64> {
        finite(amount, "removed module tissue", false)?;
        let mut remaining = amount.min(self.total());
        let initial = remaining;
        if let Some(id) = module {
            let m = self
                .modules
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| AccountingError::Invalid("damage module is missing".into()))?;
            let removed = remaining.min(m.nutrient);
            m.nutrient -= removed;
            remaining -= removed;
            return Ok(initial - remaining);
        }
        let total = self.total();
        let count = self.modules.len();
        for (index, m) in self.modules.iter_mut().enumerate() {
            let removed = if index + 1 == count {
                remaining.min(m.nutrient)
            } else {
                (initial * m.nutrient / total.max(f64::MIN_POSITIVE))
                    .min(remaining)
                    .min(m.nutrient)
            };
            m.nutrient -= removed;
            remaining -= removed;
        }
        Ok(initial - remaining)
    }

    pub fn repair(&mut self, amount: f64) -> f64 {
        let mut remaining = amount;
        for m in &mut self.modules {
            let restored = remaining.min((m.developed - m.nutrient).max(0.0));
            m.nutrient += restored;
            remaining -= restored;
        }
        amount - remaining
    }

    pub fn circles(
        &self,
        genome: &BodyGenome,
        position: Position,
        heading: f64,
    ) -> Result<Vec<Circle>> {
        let scale = (self.developed() / genome.target_nutrient())
            .max(0.0)
            .sqrt();
        finite(scale, "expressed body scale", false)?;
        if genome.circle.is_some() {
            let root = &genome.modules[0];
            return Ok(vec![Circle {
                id: root.id,
                kind: root.kind,
                center: position,
                radius: root.radius * scale,
                nutrient: self.total(),
                integrity: self.integrity(root.id),
            }]);
        }
        let layout = genome
            .layout()
            .map_err(|e| AccountingError::Invalid(e.into()))?;
        Ok(genome
            .modules
            .iter()
            .map(|g| {
                let (x, y, _) = layout[&g.id];
                let integrity = self.integrity(g.id);
                Circle {
                    id: g.id,
                    kind: g.kind,
                    center: Position {
                        x: position.x + scale * (x * heading.cos() - y * heading.sin()),
                        y: position.y + scale * (x * heading.sin() + y * heading.cos()),
                    },
                    radius: g.radius * scale * integrity.sqrt(),
                    nutrient: self
                        .modules
                        .iter()
                        .find(|m| m.id == g.id)
                        .map_or(0.0, |m| m.nutrient),
                    integrity,
                }
            })
            .collect())
    }

    pub fn inertia(&self, genome: &BodyGenome) -> Result<f64> {
        Ok(self
            .circles(genome, Position { x: 0.0, y: 0.0 }, 0.0)?
            .iter()
            .map(|c| {
                c.nutrient
                    * (0.5 * c.radius * c.radius
                        + c.center.x * c.center.x
                        + c.center.y * c.center.y)
            })
            .sum::<f64>()
            .max(0.01))
    }
}
