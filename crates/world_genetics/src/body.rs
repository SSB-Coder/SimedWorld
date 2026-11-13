use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Core,
    Propulsion,
    Sensor,
    Mouth,
    Armor,
    Storage,
    Photosynthetic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleGene {
    pub id: u64,
    pub parent: Option<u64>,
    pub kind: ModuleKind,
    pub radius: f64,
    pub angle: f64,
    #[serde(default)]
    pub orientation: f64,
    pub investment: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Diet {
    Grazer,
    Predator,
    Omnivore,
    Scavenger,
    Recycler,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyGenome {
    pub modules: Vec<ModuleGene>,
    pub next_id: u64,
    pub diet: Diet,
    pub density: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle: Option<CircleTraits>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircleTraits {
    pub photosynthetic: bool,
    pub can_ingest: bool,
    pub speed: f64,
    pub perception: f64,
    pub learning_rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptations: Option<crate::traits::Adaptations>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub id: u64,
    pub module: u64,
    pub channel: u8,
    pub output: bool,
}

impl BodyGenome {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(adaptations) = self.circle.as_ref().and_then(|c| c.adaptations.as_ref()) {
            adaptations.validate()?;
        }
        if let Some(circle) = &self.circle
            && (self.modules.len() != 1
                || !circle.speed.is_finite()
                || !(0.1..=3.0).contains(&circle.speed)
                || !circle.perception.is_finite()
                || !(0.2..=3.0).contains(&circle.perception)
                || !circle.learning_rate.is_finite()
                || !(0.0..=0.2).contains(&circle.learning_rate))
        {
            return Err("invalid circle traits");
        }
        if self.modules.is_empty()
            || self.modules.len() > 64
            || !self.density.is_finite()
            || !(0.2..=5.0).contains(&self.density)
        {
            return Err("invalid body size or density");
        }
        let mut ids = BTreeSet::new();
        let mut roots = 0;
        for m in &self.modules {
            if m.id == 0
                || m.id >= self.next_id
                || m.id > u64::MAX / 256
                || !ids.insert(m.id)
                || !m.radius.is_finite()
                || !(0.04..=1.0).contains(&m.radius)
                || !m.angle.is_finite()
                || m.angle.abs() > std::f64::consts::PI
                || !m.orientation.is_finite()
                || m.orientation.abs() > std::f64::consts::PI
                || !m.investment.is_finite()
                || !(0.1..=4.0).contains(&m.investment)
            {
                return Err("invalid body module");
            }
            if m.parent.is_none() {
                roots += 1;
                if m.kind != ModuleKind::Core {
                    return Err("root must be a vital core");
                }
            } else if m.kind == ModuleKind::Core {
                return Err("body has multiple cores");
            }
        }
        if roots != 1 {
            return Err("body requires exactly one root");
        }
        self.layout()?;
        self.ports()?;
        Ok(())
    }

    pub fn layout(&self) -> Result<BTreeMap<u64, (f64, f64, f64)>, &'static str> {
        let mut positions = BTreeMap::new();
        for _ in 0..self.modules.len() {
            for m in &self.modules {
                if positions.contains_key(&m.id) {
                    continue;
                }
                if let Some(parent) = m.parent {
                    if let Some(&(x, y, angle)) = positions.get(&parent) {
                        let p = self
                            .modules
                            .iter()
                            .find(|p| p.id == parent)
                            .ok_or("missing attachment")?;
                        let direction: f64 = angle + m.angle;
                        positions.insert(
                            m.id,
                            (
                                x + direction.cos() * (p.radius + m.radius),
                                y + direction.sin() * (p.radius + m.radius),
                                direction,
                            ),
                        );
                    }
                } else {
                    positions.insert(m.id, (0.0, 0.0, m.angle));
                }
            }
        }
        if positions.len() != self.modules.len() {
            return Err("disconnected or cyclic body");
        }
        for (i, a) in self.modules.iter().enumerate() {
            for b in self.modules.iter().skip(i + 1) {
                let pa = positions[&a.id];
                let pb = positions[&b.id];
                if (pa.0 - pb.0).hypot(pa.1 - pb.1) + 1e-10 < a.radius + b.radius {
                    return Err("body modules interpenetrate");
                }
            }
        }
        Ok(positions)
    }

    pub fn ports(&self) -> Result<Vec<Port>, &'static str> {
        let mut ports = Vec::new();
        for m in &self.modules {
            let (inputs, outputs): (&[u8], &[u8]) = if self
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.agency))
            {
                (
                    &[
                        0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23,
                        24, 25, 26, 27,
                    ],
                    &[
                        100, 101, 103, 104, 105, 106, 107, 108, 109, 110, 111, 120, 121, 122, 123,
                    ],
                )
            } else if self.circle.is_some() {
                (
                    &[0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20],
                    &[110, 111, 120],
                )
            } else {
                match m.kind {
                    ModuleKind::Core => (&[0, 1, 2, 3, 4, 5, 6], &[100, 101, 102]),
                    ModuleKind::Sensor => (&[10, 11, 12, 13, 14, 15, 16, 18], &[]),
                    ModuleKind::Propulsion => (&[], &[110, 111]),
                    ModuleKind::Mouth => (&[], &[120, 121]),
                    _ => (&[], &[]),
                }
            };
            for (&channel, output) in inputs
                .iter()
                .map(|v| (v, false))
                .chain(outputs.iter().map(|v| (v, true)))
            {
                let id =
                    m.id.checked_mul(256)
                        .and_then(|n| n.checked_add(channel as u64))
                        .ok_or("port identity overflow")?;
                ports.push(Port {
                    id,
                    module: m.id,
                    channel,
                    output,
                });
            }
            if self
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.social_senses))
            {
                for channel in 30..=47 {
                    ports.push(Port {
                        id: m.id * 256 + channel as u64,
                        module: m.id,
                        channel,
                        output: false,
                    });
                }
            }
            if self
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.foraging_choices))
            {
                for channel in 48..=51 {
                    ports.push(Port {
                        id: m.id * 256 + channel as u64,
                        module: m.id,
                        channel,
                        output: false,
                    });
                }
                ports.push(Port {
                    id: m.id * 256 + 124,
                    module: m.id,
                    channel: 124,
                    output: true,
                });
            }
            if self
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.energy_senses))
            {
                for channel in 52..=55 {
                    ports.push(Port {
                        id: m.id * 256 + channel as u64,
                        module: m.id,
                        channel,
                        output: false,
                    });
                }
            }
        }
        ports.sort_by_key(|p| p.id);
        Ok(ports)
    }

    pub fn target_nutrient(&self) -> f64 {
        self.modules
            .iter()
            .map(|m| std::f64::consts::PI * m.radius * m.radius * self.density * 12.0)
            .sum()
    }
}
