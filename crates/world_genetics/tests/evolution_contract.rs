use world_genetics::{
    body::{BodyGenome, Diet, ModuleGene, ModuleKind},
    genome::Genome,
    mutation::{MutationConfig, mutate},
    neural::{ConnectionGene, NeuralGenome, NodeGene, NodeKind},
};

fn genome() -> Genome {
    let mut genome = Genome::new(false, true, vec![0.25, -0.5]).unwrap();
    let body = BodyGenome {
        circle: None,
        modules: vec![
            ModuleGene {
                id: 1,
                parent: None,
                kind: ModuleKind::Core,
                radius: 0.3,
                angle: 0.0,
                orientation: 0.0,
                investment: 1.0,
            },
            ModuleGene {
                id: 2,
                parent: Some(1),
                kind: ModuleKind::Mouth,
                radius: 0.15,
                angle: 0.0,
                orientation: 0.0,
                investment: 1.0,
            },
            ModuleGene {
                id: 3,
                parent: Some(1),
                kind: ModuleKind::Sensor,
                radius: 0.1,
                angle: 1.57,
                orientation: 0.0,
                investment: 1.0,
            },
        ],
        next_id: 4,
        diet: Diet::Grazer,
        density: 1.0,
    };
    let mut nodes: Vec<_> = body
        .ports()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, p)| NodeGene {
            id: i as u64 + 1,
            kind: if p.output {
                NodeKind::Output
            } else {
                NodeKind::Input
            },
            port: Some(p.id),
            bias: 0.0,
            leak: 1.0,
        })
        .collect();
    let hidden = nodes.len() as u64 + 1;
    nodes.push(NodeGene {
        id: hidden,
        kind: NodeKind::Hidden,
        port: None,
        bias: 0.25,
        leak: 0.5,
    });
    let input = nodes.iter().find(|n| n.kind == NodeKind::Input).unwrap().id;
    let output = nodes
        .iter()
        .find(|n| n.kind == NodeKind::Output)
        .unwrap()
        .id;
    genome.brain = Some(NeuralGenome {
        version: 1,
        next_node: hidden + 1,
        next_connection: 3,
        nodes,
        connections: vec![
            ConnectionGene {
                id: 1,
                source: input,
                target: hidden,
                weight: 0.75,
            },
            ConnectionGene {
                id: 2,
                source: hidden,
                target: output,
                weight: -0.2,
            },
        ],
    });
    genome.body = Some(body);
    genome.validate().unwrap();
    genome
}

#[test]
fn disabled_mutation_is_exact_inheritance_and_never_changes_the_parent() {
    let parent = genome();
    let before = parent.clone();
    let config = MutationConfig {
        parameter_edits: 0.0,
        body_edits: 0.0,
        body_structure_rate: 0.0,
        neural_structure_rate: 0.0,
        ..Default::default()
    };
    let (child, edits) = mutate(&parent, &config, 73, 1, 2, 120).unwrap();
    assert_eq!(child, parent);
    assert!(edits.is_empty());
    assert_eq!(parent, before);
}

#[test]
fn structural_proposals_preserve_validity_identity_and_unedited_parameters() {
    let parent = genome();
    let config = MutationConfig {
        parameter_edits: 0.0,
        body_edits: 0.0,
        body_structure_rate: 1.0,
        neural_structure_rate: 1.0,
        ..Default::default()
    };
    let mut kinds = std::collections::BTreeSet::new();
    for seed in 0..500 {
        let (child, edits) = mutate(&parent, &config, seed, 1, 2, 120).unwrap();
        child.validate().unwrap();
        let again = mutate(&parent, &config, seed, 1, 2, 120).unwrap();
        assert_eq!(child, again.0);
        for edit in edits {
            kinds.insert(edit.kind);
        }
        let brain = child.brain.as_ref().unwrap();
        for edge in &brain.connections {
            if let Some(old) = parent
                .brain
                .as_ref()
                .unwrap()
                .connections
                .iter()
                .find(|e| e.id == edge.id)
            {
                assert_eq!(edge, old);
            }
        }
        for node in &brain.nodes {
            if let Some(old) = parent
                .brain
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|n| n.id == node.id)
            {
                assert_eq!(node, old);
            }
        }
    }
    for required in [
        "neuron_insert",
        "neuron_delete",
        "neuron_duplicate",
        "connection_add",
        "connection_delete",
        "module_add",
        "module_delete_subtree",
        "module_duplicate_subtree",
        "module_reattach",
    ] {
        assert!(kinds.contains(required), "missing {required}");
    }
}

#[test]
fn malformed_bodies_and_neural_references_are_rejected() {
    let original = genome();
    let mut bad = original.clone();
    bad.body.as_mut().unwrap().modules[1].parent = Some(2);
    assert!(bad.validate().is_err());
    let mut bad = original.clone();
    bad.body.as_mut().unwrap().modules[2].angle = 0.0;
    assert!(bad.validate().is_err());
    let mut bad = original.clone();
    bad.brain.as_mut().unwrap().connections[0].source = 999;
    assert!(bad.validate().is_err());
    let mut bad = original;
    bad.brain.as_mut().unwrap().nodes[0].bias = f64::NAN;
    assert!(bad.validate().is_err());
}
