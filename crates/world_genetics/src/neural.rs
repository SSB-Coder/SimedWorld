use std::collections::BTreeSet;

use crate::body::Port;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Input,
    Hidden,
    Output,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeGene {
    pub id: u64,
    pub kind: NodeKind,
    pub port: Option<u64>,
    pub bias: f64,
    pub leak: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionGene {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeuralGenome {
    pub version: u64,
    pub next_node: u64,
    pub next_connection: u64,
    pub nodes: Vec<NodeGene>,
    pub connections: Vec<ConnectionGene>,
}

impl NeuralGenome {
    pub fn validate(&self, ports: &[Port]) -> Result<(), &'static str> {
        if self.nodes.len() > 4096 || self.connections.len() > 65536 || self.version == 0 {
            return Err("neural graph exceeds limits");
        }
        let mut ids = BTreeSet::new();
        let mut seen_ports = BTreeSet::new();
        for n in &self.nodes {
            if n.id == 0
                || n.id >= self.next_node
                || !ids.insert(n.id)
                || !n.bias.is_finite()
                || n.bias.abs() > 8.0
                || !n.leak.is_finite()
                || !(0.05..=1.0).contains(&n.leak)
            {
                return Err("invalid neuron gene");
            }
            match n.kind {
                NodeKind::Hidden => {
                    if n.port.is_some() {
                        return Err("hidden neuron has a port");
                    }
                }
                kind => {
                    let p = ports
                        .iter()
                        .find(|p| Some(p.id) == n.port)
                        .ok_or("neural port is missing")?;
                    if p.output != (kind == NodeKind::Output) || !seen_ports.insert(p.id) {
                        return Err("neural port meaning mismatch");
                    }
                }
            }
        }
        if seen_ports.len() != ports.len() {
            return Err("body and neural ports disagree");
        }
        let inputs: BTreeSet<_> = self
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Input)
            .map(|n| n.id)
            .collect();
        let mut edges = BTreeSet::new();
        let mut edge_ids = BTreeSet::new();
        for e in &self.connections {
            if e.id == 0
                || e.id >= self.next_connection
                || !edge_ids.insert(e.id)
                || !edges.insert((e.source, e.target))
                || !ids.contains(&e.source)
                || !ids.contains(&e.target)
                || inputs.contains(&e.target)
                || !e.weight.is_finite()
                || e.weight.abs() > 8.0
            {
                return Err("invalid connection gene");
            }
        }
        Ok(())
    }

    pub fn sync_ports(&mut self, ports: &[Port]) -> Result<(), &'static str> {
        self.nodes.retain(|n| {
            n.kind == NodeKind::Hidden
                || ports
                    .iter()
                    .any(|p| Some(p.id) == n.port && p.output == (n.kind == NodeKind::Output))
        });
        let ids: BTreeSet<_> = self.nodes.iter().map(|n| n.id).collect();
        self.connections
            .retain(|e| ids.contains(&e.source) && ids.contains(&e.target));
        for p in ports {
            if !self.nodes.iter().any(|n| n.port == Some(p.id)) {
                let id = self.next_node;
                self.next_node = id.checked_add(1).ok_or("neuron identity overflow")?;
                self.nodes.push(NodeGene {
                    id,
                    kind: if p.output {
                        NodeKind::Output
                    } else {
                        NodeKind::Input
                    },
                    port: Some(p.id),
                    bias: 0.0,
                    leak: 1.0,
                });
            }
        }
        self.nodes.sort_by_key(|n| n.id);
        self.validate(ports)
    }
}
