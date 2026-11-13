use crate::{
    anatomy::Circle,
    environment::Position,
    habitat::Habitat,
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Result},
    spatial::distance,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use world_genetics::{
    body::{Diet, ModuleKind, Port},
    neural::NeuralGenome,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortValue {
    pub port: u64,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutputPort {
    pub port: u64,
    pub channel: u8,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentObservation {
    pub id: OrganismId,
    pub version: u64,
    pub brain: NeuralGenome,
    pub inputs: Vec<PortValue>,
    pub outputs: Vec<OutputPort>,
    pub authorized: bool,
    pub reward: f64,
    pub learning_rate: f64,
    pub inherit: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentFrame<A = Vec<AgentObservation>> {
    pub development_due: bool,
    pub temporal_feedback: bool,
    pub embodied_ecology: bool,
    pub development_interval_ticks: u64,
    pub trace_time: f64,
    pub plasticity_decay_per_time: f64,
    pub exploration: f64,
    pub schema_version: u32,
    pub tick: u64,
    pub duration: f64,
    pub agents: A,
    pub online_learning: bool,
    pub seed: u64,
}

impl<A> AgentFrame<A> {
    fn with_agents<T>(&self, agents: T) -> AgentFrame<T> {
        AgentFrame {
            agents,
            development_due: self.development_due,
            temporal_feedback: self.temporal_feedback,
            embodied_ecology: self.embodied_ecology,
            development_interval_ticks: self.development_interval_ticks,
            trace_time: self.trace_time,
            plasticity_decay_per_time: self.plasticity_decay_per_time,
            exploration: self.exploration,
            schema_version: self.schema_version,
            tick: self.tick,
            duration: self.duration,
            online_learning: self.online_learning,
            seed: self.seed,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnedParameters {
    pub weights: Vec<(u64, f64)>,
    pub biases: Vec<(u64, f64)>,
}

pub struct PackedParameters<'a> {
    pub node_keys: &'a [u64],
    pub biases: &'a [f64],
    pub edge_keys: &'a [u64],
    pub weights: &'a [f64],
}

impl PackedParameters<'_> {
    fn validate(
        &self,
        frame: &AgentFrame,
        batch: &ActionBatch,
    ) -> Result<BTreeMap<OrganismId, LearnedParameters>> {
        let nodes: usize = frame.agents.iter().map(|a| a.brain.nodes.len()).sum();
        let edges: usize = frame.agents.iter().map(|a| a.brain.connections.len()).sum();
        if self.biases.len() != nodes
            || self.weights.len() != edges
            || self.node_keys.len() != nodes.saturating_mul(2)
            || self.edge_keys.len() != edges.saturating_mul(2)
            || batch.agents.iter().any(|a| a.learned.is_some())
        {
            return Err(AccountingError::Invalid(
                "packed parameter shapes or sources disagree".into(),
            ));
        }
        let mut learned = BTreeMap::new();
        let (mut node_offset, mut edge_offset) = (0, 0);
        for agent in &frame.agents {
            let mut parameters = LearnedParameters {
                weights: Vec::new(),
                biases: Vec::new(),
            };
            for node in &agent.brain.nodes {
                let key = &self.node_keys[node_offset * 2..node_offset * 2 + 2];
                let value = self.biases[node_offset];
                if key != [agent.id.0, node.id]
                    || !value.is_finite()
                    || value.abs() > 8.0
                    || (node.kind == world_genetics::neural::NodeKind::Input && value != node.bias)
                {
                    return Err(AccountingError::Invalid(
                        "invalid packed neuron identity or value".into(),
                    ));
                }
                if agent.inherit {
                    parameters.biases.push((node.id, value));
                }
                node_offset += 1;
            }
            for edge in &agent.brain.connections {
                let key = &self.edge_keys[edge_offset * 2..edge_offset * 2 + 2];
                let value = self.weights[edge_offset];
                if key != [agent.id.0, edge.id] || !value.is_finite() || value.abs() > 8.0 {
                    return Err(AccountingError::Invalid(
                        "invalid packed connection identity or value".into(),
                    ));
                }
                if agent.inherit {
                    parameters.weights.push((edge.id, value));
                }
                edge_offset += 1;
            }
            if agent.inherit && !frame.online_learning {
                return Err(AccountingError::Invalid(
                    "unexpected learned inheritance parameters".into(),
                ));
            }
            if agent.inherit {
                learned.insert(agent.id, parameters);
            }
        }
        Ok(learned)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentAction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<Vec<(u64, f64)>>,
    pub id: OrganismId,
    pub version: u64,
    pub values: Vec<PortValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learned: Option<LearnedParameters>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub agents: Vec<AgentAction>,
}

pub(crate) struct Controls {
    pub activity: BTreeMap<OrganismId, BTreeMap<u64, f64>>,
    pub values: BTreeMap<OrganismId, BTreeMap<u64, f64>>,
    pub authorized: BTreeSet<OrganismId>,
    pub learned: BTreeMap<OrganismId, LearnedParameters>,
}

pub fn circles(organism: &Organism, position: Position, heading: f64) -> Result<Vec<Circle>> {
    match (&organism.genome.body, &organism.body) {
        (Some(g), Some(b)) => b.circles(g, position, heading),
        _ => Ok(vec![Circle {
            id: 0,
            kind: ModuleKind::Core,
            center: position,
            radius: 0.0,
            nutrient: organism.tissue.nutrient,
            integrity: 1.0,
        }]),
    }
}

pub fn edible(consumer: &Organism, target: &Organism) -> bool {
    if !target.alive || consumer.id == target.id || !consumer.genome.can_ingest() {
        return false;
    }
    match consumer
        .genome
        .body
        .as_ref()
        .map(|b| b.diet)
        .unwrap_or(Diet::Grazer)
    {
        Diet::Grazer => target.genome.photosynthetic(),
        Diet::Predator => target.genome.can_ingest() && !target.genome.photosynthetic(),
        Diet::Omnivore => true,
        Diet::Scavenger | Diet::Recycler => false,
    }
}

pub fn available_food(consumer: &Organism, target: &Organism, residual_fraction: f64) -> f64 {
    if !edible(consumer, target) {
        return 0.0;
    }
    let protected = if target.genome.photosynthetic()
        && consumer
            .genome
            .body
            .as_ref()
            .is_some_and(|g| g.circle.is_some() && g.diet == Diet::Grazer)
    {
        target.body.as_ref().map_or(0.0, |body| body.developed()) * residual_fraction
    } else {
        0.0
    };
    (target.tissue.nutrient - protected).max(0.0)
}

pub(crate) fn threatens(world: &Habitat, observer: &Organism, other: &Organism) -> bool {
    let remembered = world
        .diagnostics()
        .and_then(|d| d.behavior.get(&observer.id))
        .is_some_and(|b| {
            b.attacker == Some(other.id) && world.tick().saturating_sub(b.attacker_tick) <= 120
        });
    other.alive
        && (remembered
            || (edible(other, observer)
                && combat_strength(other) >= combat_strength(observer) * 0.8))
}

pub(crate) fn combat_strength(o: &Organism) -> f64 {
    let offense = crate::food_web::adaptations(o).map_or(0.0, |a| a.offense);
    let integrity = o
        .body
        .as_ref()
        .zip(o.genome.body.as_ref())
        .map_or(1.0, |(b, g)| b.functional(g, g.modules[0].id));
    (o.tissue.nutrient / 6.5).sqrt() * (0.25 + 1.5 * offense) * integrity
}

pub fn port_active(organism: &Organism, port: &Port) -> bool {
    if port.channel == 102 || port.channel == 18 {
        return false;
    }
    organism
        .body
        .as_ref()
        .zip(organism.genome.body.as_ref())
        .is_some_and(|(b, g)| b.functional(g, port.module) > 0.05)
}

pub fn neural_cost(
    organism: &Organism,
    config: &crate::evolution_config::EvolutionConfig,
    duration: f64,
) -> f64 {
    organism.brain().map_or(0.0, |b| {
        let plasticity = if config.online_learning
            && organism
                .genome
                .body
                .as_ref()
                .and_then(|g| g.circle.as_ref())
                .is_some_and(|c| c.learning_rate > 0.0)
        {
            b.connections.len() as f64 * config.plasticity_energy_per_connection_time
        } else {
            0.0
        };
        (b.nodes.len() as f64 * config.neuron_energy_per_time
            + b.connections.len() as f64 * config.connection_energy_per_time
            + if config.temporal_feedback {
                config.neuron_energy_per_time + 9.0 * config.plasticity_energy_per_connection_time
            } else {
                0.0
            }
            + plasticity)
            * duration
    })
}

pub fn sensor_cost(organism: &Organism, base: f64, duration: f64) -> f64 {
    if let Some(circle) = organism
        .genome
        .body
        .as_ref()
        .and_then(|b| b.circle.as_ref())
    {
        return base * circle.perception * duration;
    }
    let sensors = organism.genome.body.as_ref().map_or(0.0, |g| {
        g.modules
            .iter()
            .filter(|m| m.kind == ModuleKind::Sensor)
            .map(|m| {
                m.investment
                    * organism
                        .body
                        .as_ref()
                        .map_or(0.0, |b| b.functional(g, m.id))
            })
            .sum::<f64>()
    });
    base * sensors * duration
}

impl Habitat {
    pub fn events_since_json(&self, sequence: u64) -> Result<String> {
        let diagnostics = self
            .diagnostics
            .as_ref()
            .ok_or_else(|| AccountingError::Invalid("no evolution events".into()))?;
        if sequence > diagnostics.next_event
            || diagnostics
                .events
                .first()
                .is_some_and(|e| e.sequence > sequence)
        {
            return Err(AccountingError::Invalid(
                "event cursor is outside retained history".into(),
            ));
        }
        let events: Vec<_> = diagnostics
            .events
            .iter()
            .filter(|e| e.sequence >= sequence)
            .collect();
        serde_json::to_string(
            &serde_json::json!({"next_event":diagnostics.next_event,"events":events}),
        )
        .map_err(|e| AccountingError::Invalid(e.to_string()))
    }

    pub fn agent_frame_delta_json(&self, known: &str) -> Result<String> {
        let versions: BTreeMap<u64, u64> =
            serde_json::from_str(known).map_err(|e| AccountingError::Invalid(e.to_string()))?;
        let frame = self.agent_frame()?;
        #[derive(Serialize)]
        struct Row<'a> {
            id: OrganismId,
            version: u64,
            #[serde(skip_serializing_if = "Option::is_none")]
            brain: Option<&'a NeuralGenome>,
            inputs: &'a [PortValue],
            outputs: &'a [OutputPort],
            authorized: bool,
            reward: f64,
            learning_rate: f64,
            inherit: bool,
        }
        let rows: Vec<_> = frame
            .agents
            .iter()
            .map(|a| Row {
                id: a.id,
                version: a.version,
                brain: if versions.get(&a.id.0) == Some(&a.version) {
                    None
                } else {
                    Some(&a.brain)
                },
                inputs: &a.inputs,
                outputs: &a.outputs,
                authorized: a.authorized,
                reward: a.reward,
                learning_rate: a.learning_rate,
                inherit: a.inherit,
            })
            .collect();
        serde_json::to_string(&frame.with_agents(rows))
            .map_err(|e| AccountingError::Invalid(e.to_string()))
    }

    pub fn agent_frame_json(&self) -> Result<String> {
        serde_json::to_string(&self.agent_frame()?)
            .map_err(|e| AccountingError::Invalid(e.to_string()))
    }

    pub fn agent_frame(&self) -> Result<AgentFrame> {
        self.agent_frame_impl(true)
    }

    fn agent_frame_impl(&self, observe: bool) -> Result<AgentFrame> {
        // Constructors and committed updates audit the private state.
        let config = self.config.evolution.as_ref().ok_or_else(|| {
            AccountingError::Invalid("habitat has no biological controller interface".into())
        })?;
        if !self
            .tick()
            .is_multiple_of(self.rules.ecology_interval_ticks)
        {
            return Err(AccountingError::Invalid(
                "controller frame requires a decision boundary".into(),
            ));
        }
        let duration = self.rules.physics_dt * self.rules.ecology_interval_ticks as f64;
        let mut rows = Vec::new();
        let index = crate::spatial::SpatialIndex::new(
            &self.grid,
            &self.positions,
            self.organisms().iter(),
        )?;
        for organism in self
            .organisms()
            .values()
            .filter(|o| o.alive && o.brain().is_some())
        {
            let genome = organism
                .genome
                .body
                .as_ref()
                .ok_or_else(|| AccountingError::Invariant("neural body missing".into()))?;
            let body = organism
                .body
                .as_ref()
                .ok_or_else(|| AccountingError::Invariant("developed body missing".into()))?;
            let brain = organism
                .brain()
                .ok_or_else(|| AccountingError::Invariant("brain missing".into()))?;
            let position = self.positions[&organism.id];
            let heading = self.ecology.headings[&organism.id];
            let authorized = if config.energy_budgeting {
                organism.reserves.energy
                    + crate::metabolism::fuel_available(organism, &self.rules, config, duration)
                    + crate::foraging::contact_energy(self, organism, duration)
                    >= crate::metabolism::upkeep(organism, &self.rules, config, duration).total()
            } else {
                organism.reserves.energy
                    >= neural_cost(organism, config, duration)
                        + sensor_cost(organism, self.rules.sensor_energy_per_time, duration)
            };
            let shapes = body.circles(genome, position, heading)?;
            let mut sensory_cache = BTreeMap::new();
            let social = if observe
                && authorized
                && genome
                    .circle
                    .as_ref()
                    .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.social_senses))
            {
                crate::social::observations(
                    self,
                    organism,
                    self.config.ecology.perception_radius
                        * genome.circle.as_ref().expect("circle").perception,
                    &index.within(
                        &self.grid,
                        &self.positions,
                        position,
                        self.config.ecology.perception_radius
                            * genome.circle.as_ref().expect("circle").perception,
                    )?,
                )
            } else {
                [0.0; 18]
            };
            let foraging = genome
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.foraging_choices));
            let energy_senses = genome
                .circle
                .as_ref()
                .is_some_and(|c| c.adaptations.as_ref().is_some_and(|a| a.energy_senses));
            let choices = if observe && authorized && (foraging || energy_senses) {
                let range = self.config.ecology.perception_radius
                    * genome.circle.as_ref().expect("circle").perception;
                crate::foraging::details(
                    self,
                    organism,
                    range,
                    &index.within(&self.grid, &self.positions, position, range)?,
                )
            } else {
                [crate::foraging::FoodChoice::default(); 2]
            };
            let mut inputs = Vec::new();
            let mut outputs = Vec::new();
            for port in genome
                .ports()
                .map_err(|e| AccountingError::Invalid(e.into()))?
            {
                let active = authorized && port_active(organism, &port);
                if port.output {
                    outputs.push(OutputPort {
                        port: port.id,
                        channel: port.channel,
                        active,
                    });
                    continue;
                }
                let mut value = 0.0;
                if active && observe {
                    value = match port.channel {
                        30..=35 if config.social_observations => {
                            social[(port.channel - 30) as usize]
                                * crate::food_web::adaptations(organism)
                                    .map_or(1.0, |a| a.sociality)
                        }
                        36..=39 if config.social_observations => {
                            social[(port.channel - 30) as usize]
                        }
                        40..=47 => social[(port.channel - 30) as usize],
                        48..=51 => choices[1].signals[(port.channel - 48) as usize],
                        10..=13 if foraging => choices[0].signals[(port.channel - 10) as usize],
                        0 => {
                            organism.reserves.energy
                                / organism.reserve_capacity(&self.rules)?.energy
                        }
                        1 => {
                            organism.reserves.nutrient
                                / organism.reserve_capacity(&self.rules)?.nutrient
                        }
                        2 => (organism.tissue.nutrient / genome.target_nutrient()).min(1.0),
                        3 => (body.missing() / body.developed().max(1e-12)).min(1.0),
                        4 => {
                            if organism.mature(&self.rules) {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        5 => {
                            if organism.genome.photosynthetic() {
                                (self.grid.cells[self.grid.cell_index(position)?].light_available
                                    / (self.rules.max_absorbed_light_per_time * duration))
                                    .min(1.0)
                            } else {
                                0.0
                            }
                        }
                        6 => {
                            if organism.genome.photosynthetic() {
                                (self.grid.cells[self.grid.cell_index(position)?].mineral_nutrient
                                    / organism.tissue.nutrient)
                                    .min(1.0)
                            } else {
                                0.0
                            }
                        }
                        10..=16 => {
                            let sensor =
                                shapes.iter().find(|s| s.id == port.module).ok_or_else(|| {
                                    AccountingError::Invariant("sensor shape missing".into())
                                })?;
                            let gene = genome
                                .modules
                                .iter()
                                .find(|m| m.id == port.module)
                                .ok_or_else(|| {
                                    AccountingError::Invariant("sensor gene missing".into())
                                })?;
                            let direction = heading + gene.orientation;
                            let range = self.config.ecology.perception_radius
                                * gene.investment
                                * body.functional(genome, gene.id)
                                * genome.circle.as_ref().map_or(1.0, |c| c.perception);
                            let sensory = sensory_cache
                                .entry((port.module, port.channel <= 13))
                                .or_insert_with(|| {
                                    let nearby = index
                                        .within(&self.grid, &self.positions, sensor.center, range)
                                        .expect("validated sensor range");
                                    let target = nearby
                                        .iter()
                                        .map(|id| &self.organisms()[id])
                                        .filter(|o| o.alive && o.id != organism.id)
                                        .filter(|o| {
                                            if port.channel <= 13 {
                                                available_food(
                                                    organism,
                                                    o,
                                                    self.config.ecology.grazing_residual_fraction,
                                                ) > 1e-8
                                            } else {
                                                threatens(self, organism, o)
                                            }
                                        })
                                        .filter_map(|o| {
                                            let p = self.positions[&o.id];
                                            let d = distance(sensor.center, p);
                                            let delta = (p.y - sensor.center.y)
                                                .atan2(p.x - sensor.center.x)
                                                - direction;
                                            let angle = delta.sin().atan2(delta.cos());
                                            if crate::development::detected(
                                                self, organism, o, range,
                                            ) && (genome.circle.is_some()
                                                || angle.abs() <= std::f64::consts::PI * 0.65)
                                            {
                                                Some((o, d, angle))
                                            } else {
                                                None
                                            }
                                        })
                                        .min_by(|a, b| {
                                            a.1.total_cmp(&b.1).then(a.0.id.cmp(&b.0.id))
                                        });
                                    if target.is_none()
                                        && port.channel <= 13
                                        && crate::food_web::adaptations(organism).is_some()
                                        && matches!(
                                            genome.diet,
                                            Diet::Predator
                                                | Diet::Omnivore
                                                | Diet::Scavenger
                                                | Diet::Recycler
                                        )
                                    {
                                        return crate::foraging::organic_options(
                                            self, organism, range,
                                        )
                                        .into_iter()
                                        .min_by(|(a, _), (b, _)| {
                                            distance(sensor.center, *a)
                                                .total_cmp(&distance(sensor.center, *b))
                                        })
                                        .map_or(
                                            [0.0; 4],
                                            |(p, food)| {
                                                let d = distance(sensor.center, p);
                                                let angle = if d < 1e-12 {
                                                    0.0
                                                } else {
                                                    (p.y - sensor.center.y)
                                                        .atan2(p.x - sensor.center.x)
                                                        - direction
                                                };
                                                [
                                                    angle.sin(),
                                                    angle.cos(),
                                                    1.0 - d / range.max(1e-12),
                                                    food.nutrient
                                                        / (organism.tissue.nutrient
                                                            + food.nutrient),
                                                ]
                                            },
                                        );
                                    }
                                    target.map_or([0.0; 4], |(o, d, angle)| {
                                        let food = available_food(
                                            organism,
                                            o,
                                            self.config.ecology.grazing_residual_fraction,
                                        );
                                        [
                                            angle.sin(),
                                            angle.cos(),
                                            1.0 - d / range.max(1e-12),
                                            (food / (organism.tissue.nutrient + food)).min(1.0),
                                        ]
                                    })
                                });
                            sensory[match port.channel {
                                10 | 14 => 0,
                                11 | 15 => 1,
                                12 | 16 => 2,
                                _ => 3,
                            }]
                        }
                        17 => self
                            .diagnostics
                            .as_ref()
                            .and_then(|d| d.behavior.get(&organism.id))
                            .map_or(0.0, |b| {
                                (b.distance
                                    / (duration
                                        * self.config.ecology.consumer_speed
                                        * genome.circle.as_ref().map_or(1.0, |c| c.speed)))
                                .min(1.0)
                            }),
                        19 => self
                            .diagnostics
                            .as_ref()
                            .and_then(|d| d.behavior.get(&organism.id))
                            .map_or(0.0, |b| if b.blocked { 1.0 } else { 0.0 }),
                        20 => {
                            let radius = shapes[0].radius;
                            let mut clear = f64::INFINITY;
                            for (p, d, limit) in [
                                (position.x, heading.cos(), self.rules.world_width),
                                (position.y, heading.sin(), self.rules.world_height),
                            ] {
                                if d > 1e-12 {
                                    clear = clear.min((limit - radius - p) / d);
                                }
                                if d < -1e-12 {
                                    clear = clear.min((radius - p) / d);
                                }
                            }
                            (1.0 - clear
                                / (self.config.ecology.perception_radius
                                    * genome.circle.as_ref().map_or(1.0, |c| c.perception)))
                            .clamp(0.0, 1.0)
                        }
                        21 => crate::terrain::cover(&config.terrain, position),
                        22 => {
                            self.grid.cells[self.grid.cell_index(position)?]
                                .detritus
                                .nutrient
                                / (1.0
                                    + self.grid.cells[self.grid.cell_index(position)?]
                                        .detritus
                                        .nutrient)
                        }
                        23 => self
                            .ecology
                            .interactions
                            .filaments
                            .iter()
                            .filter(|t| t.captured == Some(organism.id))
                            .map(|t| t.strength)
                            .sum::<f64>()
                            .min(1.0),
                        24..=26 if config.receive_signals => {
                            let signal = self
                                .ecology
                                .interactions
                                .signals
                                .iter()
                                .filter(|s| {
                                    s.owner != organism.id
                                        && distance(position, s.position) <= s.range
                                        && crate::terrain::visible(
                                            &config.terrain,
                                            position,
                                            s.position,
                                        )
                                })
                                .min_by(|a, b| {
                                    distance(position, a.position)
                                        .total_cmp(&distance(position, b.position))
                                        .then(a.owner.cmp(&b.owner))
                                });
                            signal.map_or(0.0, |s| {
                                let angle = (s.position.y - position.y)
                                    .atan2(s.position.x - position.x)
                                    - heading;
                                match port.channel {
                                    24 => {
                                        s.value * (1.0 - distance(position, s.position) / s.range)
                                    }
                                    25 => angle.sin(),
                                    _ => angle.cos(),
                                }
                            })
                        }
                        52 => {
                            let rate =
                                crate::metabolism::upkeep(organism, &self.rules, config, 1.0)
                                    .total();
                            let horizon = config.foraging_reserve_time.max(duration);
                            organism.reserves.energy
                                / (organism.reserves.energy + rate * horizon).max(1e-12)
                        }
                        53 => choices[0].net_energy / (1.0 + choices[0].net_energy.abs()),
                        54 => choices[0].unit_energy / (1.0 + choices[0].unit_energy.abs()),
                        55 => {
                            if choices[0].signals == [0.0; 4] {
                                0.0
                            } else {
                                organism.reserves.energy
                                    / (organism.reserves.energy + choices[0].trip_energy).max(1e-12)
                            }
                        }
                        27 => (organism.age / (organism.age + 100.0)).min(1.0),
                        _ => 0.0,
                    };
                }
                if !value.is_finite() || value.abs() > 1.0 + 1e-10 {
                    return Err(AccountingError::Invariant(
                        "invalid local observation".into(),
                    ));
                }
                inputs.push(PortValue {
                    port: port.id,
                    value: value.clamp(-1.0, 1.0),
                });
            }
            rows.push(AgentObservation {
                id: organism.id,
                version: brain.version,
                brain: brain.clone(),
                inputs,
                outputs,
                authorized,
                reward: self
                    .diagnostics
                    .as_ref()
                    .and_then(|d| d.behavior.get(&organism.id))
                    .map_or(0.0, |b| b.reward),
                learning_rate: if config.learned_weight_updates {
                    genome.circle.as_ref().map_or(0.0, |c| c.learning_rate)
                } else {
                    0.0
                },
                inherit: config.inherit_learning
                    && self.config.ecology.reproduction
                    && genome.circle.is_some()
                    && organism.mature(&self.rules)
                    && organism.age >= organism.next_reproduction_age
                    && (config.embodied_ecology
                        || (organism.reserves.nutrient
                            >= self.rules.offspring_tissue_nutrient
                                + self.rules.offspring_reserve_nutrient
                            && organism.reserves.energy
                                + duration
                                    * (self.rules.max_absorbed_light_per_time
                                        * self.rules.photosynthesis_efficiency
                                        + self.rules.max_ingestion_nutrient_per_time
                                            * self.rules.tissue_energy_per_nutrient
                                            * self.rules.energy_assimilation_fraction)
                                >= self.rules.offspring_tissue_nutrient
                                    * (self.rules.tissue_energy_per_nutrient
                                        + self.rules.construction_heat_per_nutrient)
                                    + self.rules.offspring_reserve_energy)),
            });
        }
        Ok(AgentFrame {
            development_due: (self.tick() + self.rules.ecology_interval_ticks)
                .is_multiple_of(config.development_interval_ticks),
            temporal_feedback: config.temporal_feedback,
            embodied_ecology: config.embodied_ecology,
            development_interval_ticks: config.development_interval_ticks,
            trace_time: config.trace_time,
            plasticity_decay_per_time: config.plasticity_decay_per_time,
            exploration: config.exploration,
            schema_version: 1,
            tick: self.tick(),
            duration,
            agents: rows,
            online_learning: config.online_learning,
            seed: self.config.seed,
        })
    }

    pub fn advance_controlled(&mut self, text: &str) -> Result<()> {
        *self = self.propose_controlled(text)?;
        Ok(())
    }

    pub fn propose_controlled(&self, text: &str) -> Result<Self> {
        self.propose_controlled_impl(text, None)
    }

    pub fn propose_controlled_packed(
        &self,
        text: &str,
        parameters: PackedParameters<'_>,
    ) -> Result<Self> {
        self.propose_controlled_impl(text, Some(parameters))
    }

    fn propose_controlled_impl(
        &self,
        text: &str,
        parameters: Option<PackedParameters<'_>>,
    ) -> Result<Self> {
        let batch: ActionBatch = serde_json::from_str(text)
            .map_err(|e| AccountingError::Invalid(format!("invalid neural actions: {e}")))?;
        let frame = self.agent_frame_impl(false)?;
        let learned = parameters.map(|p| p.validate(&frame, &batch)).transpose()?;
        let mut controls = validate_actions(&frame, &batch)?;
        if let Some(learned) = learned {
            controls.learned = learned;
        }
        let mut candidate = self.clone();
        let count = self.rules.ecology_interval_ticks;
        for index in 0..count {
            candidate.advance_impl(if index + 1 == count {
                Some(&controls)
            } else {
                None
            })?;
        }
        candidate.audit()?;
        Ok(candidate)
    }
}

fn validate_actions(frame: &AgentFrame, batch: &ActionBatch) -> Result<Controls> {
    if batch.schema_version != 1
        || batch.tick != frame.tick
        || batch.agents.len() != frame.agents.len()
    {
        return Err(AccountingError::Invalid(
            "stale or incomplete neural batch".into(),
        ));
    }
    let mut values = BTreeMap::new();
    let mut activity = BTreeMap::new();
    let mut authorized = BTreeSet::new();
    let mut learned = BTreeMap::new();
    for action in &batch.agents {
        let expected = frame
            .agents
            .iter()
            .find(|a| a.id == action.id)
            .ok_or_else(|| AccountingError::Invalid("unknown neural identity".into()))?;
        if expected.version != action.version
            || action.values.len() != expected.outputs.len()
            || values.contains_key(&action.id)
        {
            return Err(AccountingError::Invalid(
                "duplicate identity, stale topology, or wrong action width".into(),
            ));
        }
        let mut ports = BTreeMap::new();
        for value in &action.values {
            let expected = expected
                .outputs
                .iter()
                .find(|p| p.port == value.port)
                .ok_or_else(|| AccountingError::Invalid("unknown action port".into()))?;
            let lower = if expected.channel == 111 || expected.channel == 102 {
                -1.0
            } else {
                0.0
            };
            if !value.value.is_finite()
                || value.value < lower
                || value.value > 1.0
                || (!expected.active && value.value != 0.0)
                || ports.insert(value.port, value.value).is_some()
            {
                return Err(AccountingError::Invalid(
                    "invalid, duplicate, or inactive action port".into(),
                ));
            }
        }
        if expected.authorized {
            authorized.insert(action.id);
        }
        if let Some(items) = &action.activity {
            let usage: BTreeMap<_, _> = items.iter().copied().collect();
            if usage.len() != items.len()
                || usage.len() != expected.brain.nodes.len()
                || expected.brain.nodes.iter().any(|n| {
                    usage
                        .get(&n.id)
                        .is_none_or(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                })
            {
                return Err(AccountingError::Invalid(
                    "invalid neural activity identities or values".into(),
                ));
            }
            activity.insert(action.id, usage);
        }
        if let Some(parameters) = &action.learned {
            if !frame.online_learning || !expected.inherit {
                return Err(AccountingError::Invalid(
                    "unexpected learned inheritance parameters".into(),
                ));
            }
            let mut brain = expected.brain.clone();
            let weights: BTreeMap<_, _> = parameters.weights.iter().copied().collect();
            let biases: BTreeMap<_, _> = parameters.biases.iter().copied().collect();
            if weights.len() != parameters.weights.len()
                || biases.len() != parameters.biases.len()
                || weights.len() != brain.connections.len()
                || biases.len() != brain.nodes.len()
            {
                return Err(AccountingError::Invalid(
                    "learned parameter identities disagree".into(),
                ));
            }
            for edge in &mut brain.connections {
                edge.weight = *weights
                    .get(&edge.id)
                    .ok_or_else(|| AccountingError::Invalid("learned connection missing".into()))?;
            }
            for node in &mut brain.nodes {
                node.bias = *biases
                    .get(&node.id)
                    .ok_or_else(|| AccountingError::Invalid("learned neuron missing".into()))?;
            }
            if brain
                .connections
                .iter()
                .any(|e| !e.weight.is_finite() || e.weight.abs() > 8.0)
                || brain.nodes.iter().any(|n| {
                    !n.bias.is_finite()
                        || n.bias.abs() > 8.0
                        || (n.kind == world_genetics::neural::NodeKind::Input
                            && n.bias
                                != expected
                                    .brain
                                    .nodes
                                    .iter()
                                    .find(|old| old.id == n.id)
                                    .map_or(0.0, |old| old.bias))
                })
            {
                return Err(AccountingError::Invalid(
                    "invalid learned parameter value".into(),
                ));
            }
            learned.insert(action.id, parameters.clone());
        }
        values.insert(action.id, ports);
    }
    Ok(Controls {
        activity,
        values,
        authorized,
        learned,
    })
}
