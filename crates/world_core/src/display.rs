use crate::{
    agents,
    environment::Position,
    habitat::Habitat,
    resources::{AccountingError, Result},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayOrganism {
    #[serde(default)]
    pub profile: serde_json::Value,
    pub id: u64,
    pub parent: Option<u64>,
    pub alive: bool,
    pub position: Position,
    pub heading: f64,
    pub radius: f64,
    pub integrity: f64,
    pub tissue: f64,
    pub energy: f64,
    pub nutrient: f64,
    pub age: f64,
    pub producer: bool,
    pub neurons: usize,
    pub connections: usize,
    pub reward: f64,
    pub blocked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayFrame {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corpses: Vec<crate::corpses::Corpse>,
    #[serde(default)]
    pub ecosystem: serde_json::Value,
    pub schema_version: u32,
    pub tick: u64,
    pub model_time: f64,
    pub physics_dt: f64,
    pub width: f64,
    pub height: f64,
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<[f64; 3]>,
    pub organisms: Vec<DisplayOrganism>,
    pub summary: serde_json::Value,
    #[serde(default)]
    pub brain: serde_json::Value,
}

impl DisplayFrame {
    pub fn validate(&self) -> Result<()> {
        let mut corpse_ids = std::collections::BTreeSet::new();
        if self.corpses.len() > 65536 {
            return Err(AccountingError::Invalid(
                "too many displayed corpses".into(),
            ));
        }
        for corpse in &self.corpses {
            corpse.validate()?;
            if !corpse_ids.insert(corpse.source)
                || corpse.position.x < 0.0
                || corpse.position.y < 0.0
                || corpse.position.x > self.width
                || corpse.position.y > self.height
                || corpse.radius() > self.width.hypot(self.height)
            {
                return Err(AccountingError::Invalid(
                    "invalid displayed corpse geometry".into(),
                ));
            }
        }
        if !self.ecosystem.is_null() {
            let finite_position = |v: &serde_json::Value| {
                v["x"]
                    .as_f64()
                    .is_some_and(|x| x.is_finite() && (0.0..=self.width).contains(&x))
                    && v["y"]
                        .as_f64()
                        .is_some_and(|y| y.is_finite() && (0.0..=self.height).contains(&y))
            };
            for key in ["terrain", "filaments", "signals"] {
                if let Some(items) = self.ecosystem[key].as_array() {
                    if items.len() > 65536 {
                        return Err(AccountingError::Invalid(
                            "display overlay exceeds limits".into(),
                        ));
                    }
                    for item in items {
                        if key == "terrain" {
                            let patch: crate::terrain::TerrainPatch =
                                serde_json::from_value(item.clone())
                                    .map_err(|e| AccountingError::Invalid(e.to_string()))?;
                            patch.validate(self.width, self.height)?;
                        } else if key == "filaments" {
                            if !finite_position(&item["anchor"]) || !finite_position(&item["end"]) {
                                return Err(AccountingError::Invalid(
                                    "invalid displayed filament".into(),
                                ));
                            }
                        } else if !finite_position(&item["position"])
                            || item["range"].as_f64().is_none_or(|v| {
                                !v.is_finite()
                                    || v <= 0.0
                                    || v > self.width.hypot(self.height) * 2.0
                            })
                        {
                            return Err(AccountingError::Invalid(
                                "invalid displayed signal".into(),
                            ));
                        }
                    }
                } else if !self.ecosystem[key].is_null() {
                    return Err(AccountingError::Invalid("invalid display overlay".into()));
                }
            }
        }
        if self.schema_version != 1
            || !self.width.is_finite()
            || self.width <= 0.0
            || !self.height.is_finite()
            || self.height <= 0.0
            || !self.model_time.is_finite()
            || self.model_time < 0.0
            || !self.physics_dt.is_finite()
            || self.physics_dt <= 0.0
            || self.columns.checked_mul(self.rows) != Some(self.cells.len())
            || self.columns == 0
            || self.rows == 0
        {
            return Err(AccountingError::Invalid(
                "invalid display dimensions".into(),
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for o in &self.organisms {
            if !ids.insert(o.id)
                || o.id == 0
                || [
                    o.position.x,
                    o.position.y,
                    o.heading,
                    o.radius,
                    o.integrity,
                    o.tissue,
                    o.energy,
                    o.nutrient,
                    o.age,
                    o.reward,
                ]
                .iter()
                .any(|v| !v.is_finite())
                || o.radius < 0.0
                || !(0.0..=1.0).contains(&o.integrity)
                || o.tissue < 0.0
                || o.energy < 0.0
                || o.nutrient < 0.0
                || o.age < 0.0
            {
                return Err(AccountingError::Invalid("invalid display organism".into()));
            }
        }
        if !self.brain.is_null() {
            let object = self
                .brain
                .as_object()
                .ok_or_else(|| AccountingError::Invalid("invalid displayed brain".into()))?;
            if object.len() != 5 {
                return Err(AccountingError::Invalid(
                    "invalid displayed brain fields".into(),
                ));
            }
            let id = self.brain["id"].as_u64().ok_or_else(|| {
                AccountingError::Invalid("invalid displayed brain identity".into())
            })?;
            if !ids.contains(&id)
                || self.brain["updates"].as_u64().is_none()
                || self.brain["parameter_change"]
                    .as_f64()
                    .is_none_or(|v| !v.is_finite() || v < 0.0)
            {
                return Err(AccountingError::Invalid(
                    "invalid displayed learning state".into(),
                ));
            }
            let nodes = self.brain["nodes"]
                .as_array()
                .ok_or_else(|| AccountingError::Invalid("displayed neurons missing".into()))?;
            let edges = self.brain["edges"]
                .as_array()
                .ok_or_else(|| AccountingError::Invalid("displayed connections missing".into()))?;
            let mut node_ids = std::collections::BTreeSet::new();
            if nodes.len() > 144 || edges.len() > 512 {
                return Err(AccountingError::Invalid(
                    "displayed graph exceeds inspection bounds".into(),
                ));
            }
            for node in nodes {
                let node_id = node["id"].as_u64().ok_or_else(|| {
                    AccountingError::Invalid("invalid displayed neuron identity".into())
                })?;
                if !node_ids.insert(node_id)
                    || !["input", "hidden", "output"].contains(&node["kind"].as_str().unwrap_or(""))
                    || node["value"]
                        .as_f64()
                        .is_none_or(|v| !v.is_finite() || v.abs() > 1.0)
                {
                    return Err(AccountingError::Invalid("invalid displayed neuron".into()));
                }
            }
            for edge in edges {
                if edge["source"]
                    .as_u64()
                    .is_none_or(|id| !node_ids.contains(&id))
                    || edge["target"]
                        .as_u64()
                        .is_none_or(|id| !node_ids.contains(&id))
                    || edge["weight"]
                        .as_f64()
                        .is_none_or(|v| !v.is_finite() || v.abs() > 8.0)
                {
                    return Err(AccountingError::Invalid(
                        "invalid displayed connection".into(),
                    ));
                }
            }
        }
        if self
            .cells
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(AccountingError::Invalid("invalid display field".into()));
        }
        Ok(())
    }
}

impl Habitat {
    pub fn display_json(&self, selected: Option<u64>) -> Result<String> {
        let (width, height, columns, rows) = self.grid().dimensions();
        let mut organisms = Vec::new();
        for (id, o) in self
            .organisms()
            .iter()
            .filter(|(id, o)| o.alive || Some(id.0) == selected)
        {
            let position = self.positions()[id];
            let heading = self.heading(*id).unwrap_or(0.0);
            let shapes = agents::circles(o, position, heading)?;
            let integrity = shapes.first().map_or(1.0, |s| s.integrity);
            let radius = shapes
                .iter()
                .map(|c| ((c.center.x - position.x).hypot(c.center.y - position.y)) + c.radius)
                .fold(0.0_f64, f64::max);
            let behavior = self.diagnostics().and_then(|d| d.behavior.get(id));
            organisms.push(DisplayOrganism {
                profile:serde_json::json!({"guild":crate::food_web::guild(o),"adaptations":crate::food_web::adaptations(o),
                    "energy_capacity":o.reserve_capacity(&self.rules)?.energy,
                    "edible_tissue":if o.genome.photosynthetic() {(o.tissue.nutrient-o.body.as_ref().map_or(0.0,|b|b.developed())*self.config.ecology.grazing_residual_fraction).max(0.0)}else{0.0},
                    "trophic_level":self.ecology.interactions.food_web.trophic.get(id),
                    "birth_neurons":o.genome.brain.as_ref().map_or(0,|b|b.nodes.len()),
                    "behavior":if Some(id.0)==selected {behavior}else{None},
                    "energy_history":if Some(id.0)==selected {self.diagnostics.as_ref().and_then(|d|d.energy_history.get(id))}else{None},
                    "death_history":if Some(id.0)==selected {self.diagnostics.as_ref().and_then(|d|d.death_history.iter().find(|h|h.id==*id))}else{None},
                    "events":if Some(id.0)==selected {self.diagnostics.as_ref().map(|d|d.events.iter().rev().filter(|e|e.actor==*id || e.target==Some(*id)).take(4).collect::<Vec<_>>())}else{None}}),
                id: id.0,
                parent: o.parent_id.map(|p| p.0),
                alive: o.alive,
                position,
                heading,
                radius,
                integrity,
                tissue: o.tissue.nutrient,
                energy: o.reserves.energy,
                nutrient: o.reserves.nutrient,
                age: o.age,
                producer: o.genome.photosynthetic(),
                neurons: o.brain().map_or(0, |b| b.nodes.len()),
                connections: o.brain().map_or(0, |b| b.connections.len()),
                reward: behavior.map_or(0.0, |b| b.reward),
                blocked: behavior.is_some_and(|b| b.blocked),
            });
        }
        let nutrient = self.ledger.pools().mineral_nutrient
            + self.ledger.pools().detritus.nutrient
            + self
                .organisms()
                .values()
                .map(|o| o.tissue.nutrient + o.reserves.nutrient)
                .sum::<f64>();
        let summary = serde_json::json!({"tick":self.tick(),"producer_count":self.organisms().values().filter(|o|o.alive&&o.genome.photosynthetic()).count(),
            "consumer_count":self.organisms().values().filter(|o|o.alive&&o.genome.can_ingest()).count(),
            "total_nutrient":nutrient,"absorbed_light":self.ledger.pools().absorbed_light,
            "ecology":self.metrics(),"health":self.telemetry()?});
        let frame = DisplayFrame {
            corpses: self.grid.corpses().cloned().collect(),
            ecosystem: serde_json::json!({"terrain":self.config.evolution.as_ref().map(|c|&c.terrain),
                "filaments":self.ecology.interactions.filaments,"signals":self.ecology.interactions.signals,
                "food_web":self.ecology.interactions.food_web}),
            schema_version: 1,
            tick: self.tick(),
            model_time: self.tick() as f64 * self.rules.physics_dt,
            physics_dt: self.rules.physics_dt,
            width,
            height,
            columns,
            rows,
            cells: self
                .grid()
                .cells()
                .iter()
                .map(|c| {
                    [
                        c.mineral_nutrient(),
                        c.detritus().nutrient,
                        c.light_available(),
                    ]
                })
                .collect(),
            organisms,
            summary,
            brain: serde_json::Value::Null,
        };
        serde_json::to_string(&frame).map_err(|e| AccountingError::Invalid(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> DisplayFrame {
        DisplayFrame {
            corpses: Vec::new(),
            ecosystem: serde_json::Value::Null,
            schema_version: 1,
            tick: 0,
            model_time: 0.0,
            physics_dt: 1.0 / 60.0,
            width: 4.0,
            height: 4.0,
            columns: 1,
            rows: 1,
            cells: vec![[1.0, 0.0, 2.0]],
            organisms: vec![DisplayOrganism {
                profile: serde_json::Value::Null,
                id: 1,
                parent: None,
                alive: true,
                position: Position { x: 2.0, y: 2.0 },
                heading: 0.0,
                radius: 0.5,
                integrity: 1.0,
                tissue: 1.0,
                energy: 1.0,
                nutrient: 0.0,
                age: 0.0,
                producer: false,
                neurons: 1,
                connections: 0,
                reward: 0.0,
                blocked: false,
            }],
            summary: serde_json::json!({}),
            brain: serde_json::json!({"id":1,"updates":0,"parameter_change":0.0,
                "nodes":[{"id":1,"kind":"output","value":0.0}],"edges":[]}),
        }
    }

    #[test]
    fn display_rejects_invalid_geometry_and_duplicate_identities() {
        let original = frame();
        original.validate().unwrap();
        let mut invalid = original.clone();
        invalid.organisms[0].radius = f64::NAN;
        assert!(invalid.validate().is_err());
        invalid = original.clone();
        invalid.organisms.push(invalid.organisms[0].clone());
        assert!(invalid.validate().is_err());
        invalid = original.clone();
        invalid.cells[0][0] = -1.0;
        assert!(invalid.validate().is_err());
        original.validate().unwrap();
    }

    #[test]
    fn display_rejects_unbounded_or_disconnected_neural_inspection() {
        let mut invalid = frame();
        invalid.brain["edges"] = serde_json::json!([{"source":1,"target":999,"weight":1.0}]);
        assert!(invalid.validate().is_err());
        invalid = frame();
        invalid.brain["nodes"][0]["value"] = serde_json::json!(2.0);
        assert!(invalid.validate().is_err());
        invalid = frame();
        invalid.brain["nodes"] = serde_json::json!(vec![
            serde_json::json!({"id":1,"kind":"output","value":0.0});
            145
        ]);
        assert!(invalid.validate().is_err());
    }
}
