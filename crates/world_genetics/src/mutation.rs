use std::collections::{BTreeMap, BTreeSet};

use crate::{
    body::{BodyGenome, ModuleGene, ModuleKind},
    genome::Genome,
    neural::{ConnectionGene, NodeGene, NodeKind},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MutationConfig {
    pub body_sigma: Option<f64>,
    pub parameter_edits: f64,
    pub body_edits: f64,
    pub sigma: f64,
    pub body_structure_rate: f64,
    pub neural_structure_rate: f64,
    pub brain_creation_rate: f64,
    pub max_nodes: usize,
    pub max_connections: usize,
    pub max_modules: usize,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            body_sigma: None,
            parameter_edits: 1.0,
            body_edits: 1.0,
            sigma: 0.15,
            body_structure_rate: 0.01,
            neural_structure_rate: 0.01,
            brain_creation_rate: 0.0,
            max_nodes: 1024,
            max_connections: 16384,
            max_modules: 24,
        }
    }
}

impl MutationConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self
            .body_sigma
            .is_some_and(|v| !v.is_finite() || !(0.0..=2.0).contains(&v))
        {
            return Err("invalid body mutation scale");
        }
        if !self.parameter_edits.is_finite()
            || !(0.0..=16.0).contains(&self.parameter_edits)
            || !self.body_edits.is_finite()
            || !(0.0..=16.0).contains(&self.body_edits)
            || !self.sigma.is_finite()
            || !(0.0..=2.0).contains(&self.sigma)
            || [
                self.body_structure_rate,
                self.neural_structure_rate,
                self.brain_creation_rate,
            ]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || !(1..=4096).contains(&self.max_nodes)
            || !(1..=65536).contains(&self.max_connections)
            || !(1..=64).contains(&self.max_modules)
        {
            return Err("invalid mutation configuration");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub kind: String,
    pub gene: u64,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub accepted: bool,
    pub reason: Option<String>,
}

pub struct BirthRandom(u64);
impl BirthRandom {
    pub fn new(seed: u64, parent: u64, child: u64, tick: u64) -> Self {
        let mut rng =
            Self(seed ^ parent.rotate_left(17) ^ child.rotate_left(39) ^ tick.rotate_left(7));
        rng.bits();
        rng
    }
    pub fn bits(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
    }
    pub fn unit(&mut self) -> f64 {
        (self.bits() >> 11) as f64 / 9007199254740992.0
    }
    pub fn index(&mut self, count: usize) -> usize {
        (self.bits() % count as u64) as usize
    }
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(f64::MIN_POSITIVE).ln()).sqrt()
            * (std::f64::consts::TAU * self.unit()).cos()
    }
    fn poisson(&mut self, mean: f64) -> usize {
        if mean == 0.0 {
            return 0;
        }
        let limit = (-mean).exp();
        let mut product = 1.0;
        for count in 0..64 {
            product *= self.unit();
            if product <= limit {
                return count;
            }
        }
        64
    }
}

pub fn mutate(
    parent: &Genome,
    config: &MutationConfig,
    seed: u64,
    parent_id: u64,
    child_id: u64,
    tick: u64,
) -> Result<(Genome, Vec<Edit>), &'static str> {
    config.validate()?;
    parent.validate()?;
    let mut child = parent.clone();
    let mut edits = Vec::new();
    let mut rng = BirthRandom::new(seed, parent_id, child_id, tick);
    if let Some(brain) = &mut child.brain {
        let mut eligible: Vec<_> = (0..brain.connections.len()
            + brain
                .nodes
                .iter()
                .filter(|n| n.kind != NodeKind::Input)
                .count()
                * 2)
            .collect();
        let nodes: Vec<_> = brain
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.kind != NodeKind::Input)
            .map(|(i, _)| i)
            .collect();
        let count = rng.poisson(config.parameter_edits).min(eligible.len());
        for _ in 0..count {
            let pick = rng.index(eligible.len());
            let index = eligible.swap_remove(pick);
            let delta = rng.normal() * config.sigma;
            let (gene, kind, value, bounds) = if index < brain.connections.len() {
                let e = &mut brain.connections[index];
                (e.id, "weight", &mut e.weight, (-8.0, 8.0))
            } else {
                let k = index - brain.connections.len();
                let n = &mut brain.nodes[nodes[k / 2]];
                if k % 2 == 0 {
                    (n.id, "bias", &mut n.bias, (-8.0, 8.0))
                } else {
                    (n.id, "leak", &mut n.leak, (0.05, 1.0))
                }
            };
            let before = *value;
            *value = (*value + delta).clamp(bounds.0, bounds.1);
            edits.push(Edit {
                kind: kind.into(),
                gene,
                before: Some(before),
                after: Some(*value),
                accepted: true,
                reason: None,
            });
        }
    }
    if let Some(body) = &mut child.body {
        let mut eligible: Vec<_> = (0..if body.circle.is_some() {
            if body
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.is_some())
            {
                if body
                    .circle
                    .as_ref()
                    .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.social_senses))
                {
                    17
                } else {
                    15
                }
            } else {
                5
            }
        } else {
            body.modules.len() * 3
        })
            .collect();
        let count = rng.poisson(config.body_edits).min(eligible.len());
        for _ in 0..count {
            let pick = rng.index(eligible.len());
            let index = eligible.swap_remove(pick);
            if body.circle.is_some() {
                let mut candidate = body.clone();
                let circle = candidate.circle.as_mut().ok_or("circle traits missing")?;
                let (kind, value, bounds) = match index {
                    0 => ("radius", &mut candidate.modules[0].radius, (0.04, 1.0)),
                    1 => ("density", &mut candidate.density, (0.2, 5.0)),
                    2 => ("speed", &mut circle.speed, (0.1, 3.0)),
                    3 => ("perception", &mut circle.perception, (0.2, 3.0)),
                    4 => ("learning_rate", &mut circle.learning_rate, (0.0, 0.2)),
                    other => {
                        let a = circle.adaptations.as_mut().ok_or("adaptations missing")?;
                        match other {
                            5 => ("camouflage", &mut a.camouflage, (0.0, 1.0)),
                            6 => ("pigment", &mut a.pigment, (0.0, 1.0)),
                            7 => ("armor", &mut a.armor, (0.0, 1.0)),
                            8 => ("filament", &mut a.filament, (0.0, 1.0)),
                            9 => ("communication", &mut a.communication, (0.0, 1.0)),
                            10 => ("burst", &mut a.burst, (0.0, 1.0)),
                            11 => ("neural_growth", &mut a.neural_growth, (0.0, 1.0)),
                            12 => ("neural_pruning", &mut a.neural_pruning, (0.0, 1.0)),
                            13 => ("capture", &mut a.capture, (0.0, 1.0)),
                            14 => ("digestion", &mut a.digestion, (0.0, 1.0)),
                            15 => ("offense", &mut a.offense, (0.0, 1.0)),
                            _ => ("sociality", &mut a.sociality, (0.0, 1.0)),
                        }
                    }
                };
                let before = *value;
                *value = if kind == "learning_rate" {
                    (*value + config.body_sigma.unwrap_or(config.sigma) * 0.02 * rng.normal())
                        .clamp(bounds.0, bounds.1)
                } else if index >= 5 {
                    (*value + config.body_sigma.unwrap_or(config.sigma) * 0.2 * rng.normal())
                        .clamp(bounds.0, bounds.1)
                } else {
                    (*value * (config.body_sigma.unwrap_or(config.sigma) * rng.normal()).exp())
                        .clamp(bounds.0, bounds.1)
                };
                let after = *value;
                let validation = candidate.validate();
                let accepted = validation.is_ok();
                edits.push(Edit {
                    kind: kind.into(),
                    gene: 1,
                    before: Some(before),
                    after: Some(after),
                    accepted,
                    reason: validation.err().map(str::to_string),
                });
                if accepted {
                    *body = candidate;
                }
                continue;
            }
            let mut candidate = body.clone();
            let m = &mut candidate.modules[index / 3];
            let gene = m.id;
            let (kind, value, bounds) = match index % 3 {
                0 => ("radius", &mut m.radius, (0.04, 1.0)),
                1 => ("investment", &mut m.investment, (0.1, 4.0)),
                _ => (
                    "orientation",
                    &mut m.orientation,
                    (-std::f64::consts::PI, std::f64::consts::PI),
                ),
            };
            let before = *value;
            *value = if kind == "orientation" {
                let v = *value + config.body_sigma.unwrap_or(config.sigma) * rng.normal();
                v.sin().atan2(v.cos())
            } else {
                (*value * (config.body_sigma.unwrap_or(config.sigma) * rng.normal()).exp())
                    .clamp(bounds.0, bounds.1)
            };
            let after = *value;
            let validation = candidate.validate();
            let accepted = validation.is_ok();
            edits.push(Edit {
                kind: kind.into(),
                gene,
                before: Some(before),
                after: Some(after),
                accepted,
                reason: validation.err().map(str::to_string),
            });
            if accepted {
                *body = candidate;
            }
        }
        if body
            .circle
            .as_ref()
            .is_some_and(|c| c.adaptations.is_some() && c.can_ingest)
            && rng.unit() < config.body_structure_rate
        {
            let before = body.diet;
            let diets = [
                crate::body::Diet::Grazer,
                crate::body::Diet::Predator,
                crate::body::Diet::Omnivore,
                crate::body::Diet::Scavenger,
                crate::body::Diet::Recycler,
            ];
            body.diet = diets[rng.index(diets.len())];
            edits.push(Edit {
                kind: format!("diet_{:?}_to_{:?}", before, body.diet),
                gene: 1,
                before: None,
                after: None,
                accepted: true,
                reason: None,
            });
        }
        if body.circle.is_none() && rng.unit() < config.body_structure_rate {
            let mut candidate = body.clone();
            let (kind, gene) = body_structure(&mut candidate, &mut rng)?;
            let validation = if kind.contains("_rejected") {
                Err("root edits are prohibited")
            } else {
                candidate.validate()
            }
            .and_then(|_| {
                if candidate.modules.len() > config.max_modules
                    || child.brain.as_ref().is_some_and(|b| {
                        candidate.ports().map_or(true, |p| {
                            p.len()
                                + b.nodes
                                    .iter()
                                    .filter(|n| n.kind == NodeKind::Hidden)
                                    .count()
                                > config.max_nodes
                        })
                    })
                {
                    Err("module ceiling")
                } else {
                    Ok(())
                }
            });
            let accepted = validation.is_ok();
            edits.push(Edit {
                kind,
                gene,
                before: None,
                after: None,
                accepted,
                reason: validation.err().map(str::to_string),
            });
            if accepted {
                *body = candidate;
            }
        }
    }
    if let Some(body) = &mut child.body
        && body.circle.is_some()
        && child.brain.is_none()
        && rng.unit() < config.brain_creation_rate
    {
        let mut brain = crate::neural::NeuralGenome {
            version: 1,
            next_node: 1,
            next_connection: 1,
            nodes: Vec::new(),
            connections: Vec::new(),
        };
        brain.sync_ports(&body.ports()?)?;
        let sources: Vec<_> = brain
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Input)
            .map(|n| n.id)
            .collect();
        let targets: Vec<_> = brain
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Output)
            .map(|n| n.id)
            .collect();
        for source in sources {
            for target in &targets {
                add_edge(&mut brain, source, *target, rng.normal() * 0.1)?;
            }
        }
        if brain.nodes.len() <= config.max_nodes
            && brain.connections.len() <= config.max_connections
        {
            child.brain = Some(brain);
            edits.push(Edit {
                kind: "brain_create".into(),
                gene: 1,
                before: None,
                after: None,
                accepted: true,
                reason: None,
            });
        }
    }
    if let (Some(brain), Some(body)) = (&mut child.brain, &child.body) {
        brain.sync_ports(&body.ports()?)?;
        if rng.unit() < config.neural_structure_rate {
            let mut candidate = brain.clone();
            let operation = neural_structure(
                &mut candidate,
                &mut rng,
                body.circle
                    .as_ref()
                    .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.social_senses)),
            );
            let validation = operation
                .as_ref()
                .map(|_| ())
                .map_err(|e| *e)
                .and_then(|_| candidate.validate(&body.ports()?))
                .and({
                    if candidate.nodes.len() > config.max_nodes
                        || candidate.connections.len() > config.max_connections
                    {
                        Err("neural ceiling")
                    } else {
                        Ok(())
                    }
                });
            let accepted = validation.is_ok();
            let (kind, gene) = operation.unwrap_or_else(|_| ("neural_noop".into(), 0));
            edits.push(Edit {
                kind,
                gene,
                before: None,
                after: None,
                accepted,
                reason: validation.err().map(str::to_string),
            });
            if accepted {
                *brain = candidate;
            }
        }
    }
    if child != *parent
        && let Some(brain) = &mut child.brain
    {
        brain.version = brain
            .version
            .checked_add(1)
            .ok_or("topology version overflow")?;
    }
    child.refresh_capabilities();
    child.validate()?;
    Ok((child, edits))
}

fn body_structure(
    body: &mut BodyGenome,
    rng: &mut BirthRandom,
) -> Result<(String, u64), &'static str> {
    let index = rng.index(body.modules.len());
    let chosen = body.modules[index].clone();
    let operation = rng.index(4);
    match operation {
        0 => {
            let kinds = [
                ModuleKind::Sensor,
                ModuleKind::Propulsion,
                ModuleKind::Mouth,
                ModuleKind::Armor,
                ModuleKind::Storage,
                ModuleKind::Photosynthetic,
            ];
            let id = body.next_id;
            body.next_id = id.checked_add(1).ok_or("module identity overflow")?;
            body.modules.push(ModuleGene {
                id,
                parent: Some(chosen.id),
                kind: kinds[rng.index(kinds.len())],
                radius: 0.08,
                angle: rng.unit() * std::f64::consts::TAU - std::f64::consts::PI,
                orientation: 0.0,
                investment: 1.0,
            });
            Ok(("module_add".into(), id))
        }
        1 => {
            if chosen.parent.is_none() {
                return Ok(("module_delete_root_rejected".into(), chosen.id));
            }
            let descendants = subtree(body, chosen.id);
            body.modules.retain(|m| !descendants.contains(&m.id));
            Ok(("module_delete_subtree".into(), chosen.id))
        }
        2 => {
            if chosen.parent.is_none() {
                return Ok(("module_duplicate_root_rejected".into(), chosen.id));
            }
            let descendants = subtree(body, chosen.id);
            let mut mapping = BTreeMap::new();
            for id in &descendants {
                mapping.insert(*id, body.next_id);
                body.next_id = body
                    .next_id
                    .checked_add(1)
                    .ok_or("module identity overflow")?;
            }
            let copies: Vec<_> = body
                .modules
                .iter()
                .filter(|m| descendants.contains(&m.id))
                .map(|m| {
                    let mut copy = m.clone();
                    copy.id = mapping[&m.id];
                    copy.parent = m.parent.map(|p| mapping.get(&p).copied().unwrap_or(p));
                    if m.id == chosen.id {
                        copy.angle = -copy.angle;
                    }
                    copy
                })
                .collect();
            body.modules.extend(copies);
            Ok(("module_duplicate_subtree".into(), chosen.id))
        }
        _ => {
            if chosen.parent.is_none() {
                return Ok(("module_reattach_root_rejected".into(), chosen.id));
            }
            if chosen.parent.is_some() {
                let next = body.modules[rng.index(body.modules.len())].id;
                body.modules[index].parent = Some(next);
                body.modules[index].angle = (rng.unit() * 2.0 - 1.0) * std::f64::consts::PI;
            }
            Ok(("module_reattach".into(), chosen.id))
        }
    }
}

fn subtree(body: &BodyGenome, root: u64) -> BTreeSet<u64> {
    let mut ids = BTreeSet::from([root]);
    loop {
        let n = ids.len();
        for m in &body.modules {
            if m.parent.is_some_and(|p| ids.contains(&p)) {
                ids.insert(m.id);
            }
        }
        if n == ids.len() {
            break;
        }
    }
    ids
}

fn neural_structure(
    brain: &mut crate::neural::NeuralGenome,
    rng: &mut BirthRandom,
    conservative: bool,
) -> Result<(String, u64), &'static str> {
    match rng.index(5) {
        0 => {
            if brain.connections.is_empty() {
                return Err("no connection to split");
            }
            let at = rng.index(brain.connections.len());
            let edge = if conservative {
                brain.connections[at].clone()
            } else {
                brain.connections.remove(at)
            };
            let id = brain.next_node;
            brain.next_node = id.checked_add(1).ok_or("neuron identity overflow")?;
            brain.nodes.push(NodeGene {
                id,
                kind: NodeKind::Hidden,
                port: None,
                bias: 0.0,
                leak: 0.5,
            });
            for (source, target, weight) in [
                (edge.source, id, 1.0),
                (
                    id,
                    edge.target,
                    if conservative { 0.0 } else { edge.weight },
                ),
            ] {
                add_edge(brain, source, target, weight)?;
            }
            Ok(("neuron_insert".into(), id))
        }
        1 => {
            let candidates: Vec<_> = brain
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Hidden)
                .map(|n| n.id)
                .collect();
            if candidates.is_empty() {
                return Err("no hidden neuron to remove");
            }
            let id = candidates[rng.index(candidates.len())];
            brain.nodes.retain(|n| n.id != id);
            brain
                .connections
                .retain(|e| e.source != id && e.target != id);
            Ok(("neuron_delete".into(), id))
        }
        2 => {
            let candidates: Vec<_> = brain
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Hidden)
                .cloned()
                .collect();
            if candidates.is_empty() {
                return Err("no hidden neuron to duplicate");
            }
            let source = candidates[rng.index(candidates.len())].clone();
            let mut node = source.clone();
            let id = brain.next_node;
            brain.next_node = id.checked_add(1).ok_or("neuron identity overflow")?;
            node.id = id;
            let edges = brain.connections.clone();
            brain.nodes.push(node);
            for e in edges
                .iter()
                .filter(|e| e.source == source.id || e.target == source.id)
            {
                add_edge(
                    brain,
                    if e.source == source.id { id } else { e.source },
                    if e.target == source.id { id } else { e.target },
                    e.weight,
                )?;
            }
            Ok(("neuron_duplicate".into(), id))
        }
        3 => {
            if brain.nodes.is_empty() {
                return Err("empty neural graph");
            }
            let targets: Vec<_> = brain
                .nodes
                .iter()
                .filter(|n| n.kind != NodeKind::Input)
                .map(|n| n.id)
                .collect();
            if targets.is_empty() {
                return Err("no neural target");
            }
            let source = brain.nodes[rng.index(brain.nodes.len())].id;
            let target = targets[rng.index(targets.len())];
            if brain
                .connections
                .iter()
                .any(|e| e.source == source && e.target == target)
            {
                return Err("connection already exists");
            }
            let id = brain.next_connection;
            let weight = rng.normal() * 0.2;
            add_edge(brain, source, target, weight)?;
            Ok(("connection_add".into(), id))
        }
        _ => {
            if brain.connections.is_empty() {
                return Err("no connection to delete");
            }
            let at = rng.index(brain.connections.len());
            let id = brain.connections.remove(at).id;
            Ok(("connection_delete".into(), id))
        }
    }
}

fn add_edge(
    brain: &mut crate::neural::NeuralGenome,
    source: u64,
    target: u64,
    weight: f64,
) -> Result<(), &'static str> {
    let id = brain.next_connection;
    brain.next_connection = id.checked_add(1).ok_or("connection identity overflow")?;
    brain.connections.push(ConnectionGene {
        id,
        source,
        target,
        weight,
    });
    Ok(())
}
