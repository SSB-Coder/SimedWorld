# World: evolutionary microcosm plan

Status: the expanded prototype implements neural action/development/reproduction requests, local signals, filament capture/escape, camouflage/terrain perception, multiple diets and living recyclers, measured food flows, lifetime neural growth/pruning, learned-parameter archives, bounded studies, held-out subject evaluation and Windows release workflows. The earlier circle repair remains the two-role control. Three expanded 60,000-tick trials retained conservation and valid continuation, but higher guilds went extinct and some remaining consumers were sterile. This does not establish sustained food-web diversity or increased intelligence. Detailed articulation, senescence, sexual crossover, full 3D, distributed execution and robotics remain later scope. The walkthrough is intentionally frozen.

Current correction scope: area-limited capture, funded organic initialization, smaller detritus feeders and size-scaled offspring; diet-independent directional combat and feeding-assault feedback; local social observations and explicit food choice; private temporal-value learning state and activity-based pruning; material-funded neural growth, separate body mutation scale and inherited-learning controls; a selectable custom sparse LTC backend and online official-library LTC/CfC controls. The viewer exposes action execution, targets/attackers and a detritus field. These mechanisms are implemented; sustained guild viability, evolved cooperation and cognitive improvement remain experimental outcomes. See `artifacts/setup/social-implementation-verification.json` for the checked scope and failed as well as successful trials. The walkthrough remains frozen.

Current baseline: [WORLD_RULES.md](WORLD_RULES.md), [versioned coefficients](configs/world_rules_v0.json), the [worked reference](reference/worked_example.py), and [native accounting/habitat](docs/resource_accounting.md). [README.md](README.md) provides run/check commands. The independent ledgers and scripted habitat remain controls. Evolving organisms use inherited local neural control, funded mutated births, body geometry, and the same resource calculations. The observer receives coordinated world/neural state from the Python owner.

Dependency status: the isolated Python environment is installed and checked, including CPU/GPU LTC/CfC smoke tests. Windows SDK 10.0.26100.0 is installed. The production Rust/Python extension builds, installs, and passes native-call checks both editable and from a wheel; it exposes the habitat, resource ledger, and diagnostic bridge. The viewer activates reviewed Bevy dependencies; Rapier mechanics remains deferred. The active Rust graph contains 470 registry packages whose versions/checksums match the earlier reviewed graph. egui integration is deferred due to an unmaintained transitive font-parser advisory. See [DEPENDENCIES.md](DEPENDENCIES.md); this setup does not implement the full simulator or select a final controller family.

Companion design: [Core players](CORE_PLAYERS.md) defines the founder populations, shared organism model, resource pools, interaction rules, and experiment-system responsibilities.

Execution detail: [Implementation roadmap](IMPLEMENTATION_ROADMAP.md) specifies work packages, data contracts, provisional starting parameters, checks, and release gates. [Learning guide](LEARNING_GUIDE.md) is tailored to a developer comfortable with Python/PyTorch who is new to Rust.

## 1. Goal and agreed scope

Build a 2D artificial-life world with simple modular organisms. The first priority is observable evolution and a functioning ecosystem. Organisms should consume resources, grow, reproduce, interact, die, and return material to the environment. Their bodies and neural controllers should evolve together.

Survival is the organizing goal. Individuals must maintain themselves in the world; reproduction allows surviving strategies to persist across generations. Intelligence, neural growth, and improved learning are possible means of surviving, not independent objectives that earn ecological rewards. Preserve promising evolved networks for secondary analysis as final test subjects outside the evolving ecosystem. Support zero-neuron organisms and variable networks ranging from a few neurons through hundreds to a few thousand, subject to measured computational capacity.

Survival is not assumed to imply increasing intelligence. Cheap passive strategies, specialized reflexes, and complex learned behavior can all succeed under different conditions. Design ecological opportunities for information use, memory, and learning, then measure whether those capabilities evolve. Reproductive success, survival time, and measured cognitive performance remain separate recorded outcomes.

The proposed feedback loop is experience -> possible lifetime adaptation -> survival and reproduction -> inherited variation -> further experience. It resembles a small self-improving system, but ordinary adaptation or evolution alone does not establish recursive self-improvement. An experimental recursive component would require changes to the machinery of improvement itself, such as inherited changes to learning rules or developmental mechanisms, with evidence that these changes improve subsequent adaptation. Neither accelerating improvement nor unlimited intelligence is assumed. Such mechanisms remain subject to the same survival constraints and resource costs.

The simulator supplies consistent physical and biological rules. Heritable variation and differential reproduction determine which lineages persist. We will seed viable life; the first version does not attempt the origin of life from chemistry.

Success means measurable adaptation and ecological interactions under declared assumptions. Sustained novelty, increasing complexity, and coexistence are research outcomes to investigate, not guarantees. Simple organisms can remain successful, and extinction is a valid result.

## 2. Starting world

- A bounded, continuous 2D plane with simple collision and movement rules.
- A grid underneath the organisms for light, available mineral nutrients, detritus, and resource diffusion.
- Spatial variation in light and nutrients to create different opportunities.
- External light as the main energy input; heat as an energy sink.
- A finite mineral nutrient budget recycled through living tissue, waste, and dead matter.
- An initial producer population, simple consumers, and an aggregate decomposer process.
- Asexual reproduction initially, with overlapping lifetimes and local offspring placement.
- No automatic respawning during experiments. A separate, explicitly labeled demonstration mode may reseed a world.

Early producers and decomposition provide a controlled ecological scaffold. Later milestones replace their fixed traits with evolving producer and decomposer populations. We will distinguish scaffold behavior from evolved behavior in the interface and results.

### Renewable light and extinction diagnosis

The baseline light source continues indefinitely with finite energy flux per unit area and model time. Photosynthetic tissue, exposed area, absorption rate, efficiency, and storage limit capture; growth/reproduction still require finite minerals and construction energy. Preserve intended light flux, nutrient density, and founder density when comparing world areas or grid resolutions. Keep continuous light as the control before optional day/night experiments.

Before changing the current consumer-extinction outcome, record sensing/travel/digestion/maintenance expenditure, encounters, successful ingestion, accessible biomass, death causes, failed reproduction, mineral lock-up, and crowding. Compare declared presets across seeds and retain failures. A poor search policy, inaccessible food, and insufficient intake require different changes. Do not preserve diversity through hidden respawning or resource grants.

## 3. Resource accounting comes first

Track usable chemical energy separately from material. Use one limiting mineral nutrient initially, with explicit conversion factors between that nutrient and tissue. This is an abstract ecological model, not full biochemistry.

Every organism has structural biomass, stored energy, stored nutrients, health/damage, maturity, and age. Eating transfers resources from an existing pool. Growth converts stored resources into tissue. Reproduction transfers resources into an offspring and incurs construction costs. Death transfers remaining tissue and energy into detritus. Waste and respiration return mineral nutrients according to the same accounting rules.

Photosynthesis consumes light availability and nutrients to build energy stores and tissue. Movement, sensing, digestion, repair, neural activity, and maintenance expend energy. Decomposition transfers detrital nutrients into available pools and dissipates chemical energy; it never creates usable energy.

Audit these budgets every run:

```text
initial nutrient + nutrient imports
  = free nutrient + living nutrient + detrital nutrient + waste nutrient
    + cumulative nutrient exports + numerical error

initial chemical energy + initial mechanical energy + captured light energy + energy imports
  = stored chemical energy + tracked mechanical energy + cumulative dissipated heat
    + cumulative energy exports + numerical error
```

Pools must be disjoint. Digestion losses enter waste, detritus, or heat explicitly. Offspring, newly grown organs, and carcasses cannot receive resources already spent elsewhere. Set numerical tolerances and a consistent units table before adding evolution.

When movement stores kinetic or potential energy, include that mechanical energy rather than immediately counting the same expenditure entirely as heat. Declare the simplified work/efficiency model and track physics-solver energy residuals separately from biological transfer errors. Numerical correction is a reported residual, not a source of food or offspring resources.

## 4. One organism, one inherited genome

Keep the inherited genome separate from the living organism's changing state. The genome has coupled components:

| Component | Examples of heritable information |
| --- | --- |
| Body | Module identities, connections, dimensions, attachment positions, sensor and actuator properties |
| Physiology | Digestion efficiencies, maintenance rates, storage capacity, growth allocation, repair investment |
| Brain | Neurons, connections, initial weights, biases, time constants, recurrent connections |
| Plasticity/development | Learning rates, bounded weight-update rules, later neural growth/pruning rules |
| Reproduction | Maturity threshold, offspring investment, reproduction threshold |

Development turns this genome into a viable juvenile. Growth requires actual resources and follows developmental constraints. Mutation can produce poor or nonviable offspring; their failed development still has an accounted resource cost. Reject only structurally invalid representations, and record those rejections.

Body and brain mutations act on the same inherited organism. We do not optimize a body in isolation and attach an unrelated controller afterward.

### Genetic algorithm and ecological selection

Genetic machinery operates at offspring creation. Initially this is a mutation-only evolutionary algorithm with overlapping lifetimes. Later crossover adds sexual recombination. The ecosystem supplies selection through local resource access, mortality, and successful reproduction; no global fitness tournament, generational replacement, or intelligence bonus is applied.

The first inheritance operation is:

```text
offspring_genome = mutate(copy(parent_inherited_genome), birth_rng)
offspring = develop(offspring_genome, resources_transferred_from_parent)
```

The copied source is the inherited genome, not the parent's learned weights, current body size, injury state, or recurrent memory. Heritable developmental instructions and learning parameters can influence how an offspring later changes. A separate acquired-state-inheritance experiment may change this rule, but it is not the baseline.

| Genetic target | Initial operators | Later operators / constraints |
| --- | --- | --- |
| Body scalar traits | Perturb size, propulsion capacity, sensor range, storage allocation | Add/remove/duplicate/reposition modules while preserving valid attachment and port identities |
| Physiology | Perturb digestion allocation, maintenance-related tradeoffs, growth and offspring investment | Extend substrate specialization; enforce coupled costs so an efficiency gene cannot independently grant unlimited throughput |
| Neural parameters | Perturb initial weights, biases, time constants | Add/remove/duplicate neurons or connections; maintain declared recurrent update semantics and numerical bounds |
| Learning/development | Initially fixed parameters; evolve selected learning rates once learning exists | Mutate bounded plasticity and growth/pruning parameters; arbitrary executable code mutation is out of scope |
| Reproduction | Perturb maturity/resource thresholds and offspring investment within viable representation bounds | Add local mating, compatibility rules, and aligned crossover after asexual inheritance works |

Store mutation probabilities, perturbation distributions, and structural event rates in the experiment configuration. Begin with a small set of scalar operators and fixed mutation rates. Record the expected and realized number of mutations per offspring; a constant per-gene probability changes mutation load as genomes grow. Benchmark and test mutation budgets rather than assuming one rate suits both tiny and large genomes. Evolving mutation rates themselves is a later controlled experiment.

For each birth, perform an explicit transaction:

1. Verify parental maturity, viable reproductive opportunity, and available investment.
2. Generate the child genome with a birth-specific random stream and stable child/gene IDs.
3. Validate representation constraints, references, port compatibility, and computational limits. Record invalid mutation proposals; do not silently repair them into a favored design.
4. Reserve the required parental resource transfer and begin the declared development process. A larger inherited design must be funded or follow an explicit delayed/failed-development rule.
5. Commit the child's initial tissue/reserves, parental debits, construction losses, lineage edge, and mutation log together.
6. Publish a birth event for controller construction at the next eligible neural boundary. Give the child its declared initial recurrent state and inherited initial weights.

Distinguish malformed representations rejected before development from biologically nonviable designs that consume developmental investment and fail. Failed tissue and remaining resources follow the same detritus/heat accounting as other deaths. No automatic free retry until a successful offspring appears.

Structural neural mutation must preserve surviving gene IDs and their meaning. Adding or removing body modules changes only the corresponding sensory/motor ports and connections. Node insertion, deletion, and duplication are logged; they are not assumed behavior-preserving merely because a graph stays valid. At runtime, the Rust genotype/topology metadata and PyTorch controller must agree on organism ID and topology version.

When sexual reproduction is introduced, both parents fund explicit contributions. Align homologous genes using stable historical identifiers, define compatibility and conflict handling, and validate a connected body and executable brain. Do not splice arbitrary graph array positions or favor a parent using a hidden intelligence score.

Record survival duration, offspring count, descendants over a declared observation window, and causes of death as separate measurements. Account for individuals still alive at the end of a run and unequal reproductive opportunities. Genome clusters remain descriptive unless an explicitly modeled mating mechanism uses compatibility.

The mutation-disabled control must preserve inherited parameters across births while permitting ordinary development and, where enabled, lifetime learning. Separate body-disabled, brain-disabled, and plasticity-disabled experiments identify which mechanism contributes to an outcome. NEAT informs gene/topology representation [1]; its explicit fitness sharing and species quotas are not imported into baseline ecological selection.

### Inherited weights and continuity between generations

Offspring never receive a freshly randomized entire controller as the normal birth rule. They inherit parental encoded weights, biases, neural parameters, wiring, and learning/development genes, with only the configured mutations. Existing unmutated parameters retain their inherited values by stable gene ID. Initial founder genomes and genuinely new structural genes require explicit initialization; adding a neuron or connection must not reinitialize the rest of the network.

Distinguish inherited parameter knowledge from a parent's transient recurrent state and acquired plasticity. Resetting a child's recurrent state or plastic offsets does not reset its inherited weights. With mutation disabled, matching child genome parameters equal the parent's inherited genome parameters even when the parent has learned during life.

Support two clearly labeled inheritance experiments once plasticity exists:

```text
genetic baseline:
    child_birth_weight = mutate(parent_encoded_weight)

acquired-weight variant, for matching inherited connection IDs:
    child_birth_weight = mutate(parent_encoded_weight
                                + fraction * (parent_expressed_weight
                                              - parent_encoded_weight))
```

fraction = 0 is the baseline; fractions up to 1 test partial/full transfer of expressed weight changes on matching inherited connections. Apply parameter bounds and constraints after the declared transform. In that variant, absorbed changes become the child's encoded birth weights and its new plastic offset starts at zero. Other recurrent state, eligibility traces, optimizer state, and personal experience history still reset unless a separately declared experiment changes them. Newly acquired neurons/connections are not silently added to the inherited genome by this weight-only rule; structural inheritance requires its own gene mapping/development policy.

This acquired-weight mode is an artificial model choice, not necessary for ordinary weight inheritance or a claim that animal lifetime learning is generally copied into offspring. Keep its ancestry/configuration separate and compare it with genetic-only inheritance. Both modes preserve intergenerational continuity through inherited encoded parameters.

### Concrete birth-time operator schedule

Implement a custom event-driven evolutionary engine in Rust. Each funded birth is one inheritance/mutation event; organisms do not wait for a synchronized generation boundary. Mutation-only asexual inheritance is the first evolutionary algorithm. Local mating and crossover are a later addition using the same resource and lineage transactions.

Represent a genome as typed records with stable IDs, not a flat byte string:

```text
Genome
    schema_version, genome_id, parent_genome_ids, controller_family
    body_modules:       module_id, kind, attachment, geometry, sensor/actuator ports
    physiology_genes:   capacities, digestion investments, maintenance/growth tradeoffs
    neuron_genes:       neuron_id, kind, initial parameters, development metadata
    connection_genes:   connection_id, source_id, target_id, initial weight, enable state
    plasticity_genes:   permitted local-rule parameters and modulation wiring
    reproduction_genes:maturity, offspring investment, reproduction constraints
```

Some controller families encode more than one learned parameter per connection/neuron. Use a typed parameter schema rather than assuming every model has one unrestricted edge weight. Keep LTC/CfC parameter constraints and wiring semantics explicit. The initial experiment fixes controller family; switching among RNN/LTC/CfC is a controlled comparison, not an unvalidated cross-family mutation or crossover.

Proposed debugging settings, not calibrated biological constants:

| Operator | Starting schedule | Mutation |
| --- | --- | --- |
| Neural scalar parameters | Draw a Poisson edit count with mean 1 per birth, capped at eligible parameter count | Choose distinct eligible parameters; perturb by a configured zero-mean distribution in the appropriate parameter domain |
| Body/physiology scalar parameters | Draw a separate Poisson edit count with mean 1, capped at eligible trait count | Perturb a trait's bounded/transformed gene; recompute its coupled capability/cost model |
| Structural body edit | Disabled until M3; then a provisional 0.01 chance per birth | At most one add/remove/duplicate/reposition proposal chosen from enabled legal operators |
| Structural neural edit | Disabled until M4; then a provisional 0.01 chance per birth | At most one add/remove/duplicate neuron/connection proposal with stable IDs |
| Learning-rule parameter edits | Disabled until M6 | Add a separate explicit edit budget after the local rule exists |
| Crossover | Disabled initially | Later align compatible parental gene IDs; both parents fund declared contributions |

Tune perturbation scales in normalized gene coordinates and declare the actual bounds/distributions before experiments. A fixed mean edit count is a deliberate initial assumption about mutation load, not a biological law or a guarantee that all genome sizes are equally evolvable. Compare it against alternatives in later sensitivity studies. Zero edits are allowed; reproduction need not mutate every child. Useful phenotypes are not identified before choosing mutation targets.

The implementation contract is approximately:

```text
on_reproduction_opportunity(parent_id, tick):
    verify_parent_and_local_birth_opportunity()
    allocate_deterministic_birth_ids_and_rng()
    child_genome = copy(parent.inherited_genome)
    if declared_acquired_weight_mode:
        blend_matching_inherited_connection_weights()
    apply_enabled_scalar_and_structural_mutations()
    validate_representation_and_parameter_domains()
    plan_resource_funded_development_and_placement()
    atomically_commit_parent_debit_child_state_losses_and_lineage()
    emit_controller_birth_event(child_genome, child_id, topology_version)
```

This pseudocode describes the intended birth sequence; the executable path lives in `world_core::evolution` and `world_genetics::mutation`. Biological maturation, opportunity, and resource tests belong to the world rules. The mutation engine does not rank parents globally, target a food source for them, predict which child will be smarter, or keep an elite organism alive. Resource access, death, and descendants produce selection. The observer may rank results for analysis without influencing reproduction.

Prefer representationally legal mutation proposals by construction. If an operator has no valid target or creates malformed references, follow a predeclared rejection/no-op policy and record the attempt; do not repeatedly resample for a favorable child. Distinguish that from a representationally valid but poor design, whose development and survival face the normal costs. Operational graph/population ceilings are documented experiment limits; reaching them is reported and handled according to the study stop rule, not disguised as mortality or free reproductive success.

When building the child's PyTorch module, map by gene IDs. Copy all matching inherited tensors, initialize only genuinely new parameters, validate ports/domains, and create fresh recurrent/eligibility state. Structural edits must not reload a randomly initialized whole network. In acquired-weight mode, transfer the permitted parental learned values into child encoded parameters before mutation, with the topology-matching restriction already specified above.

Implement meaningful checks for parent-genome immutability, exact mutation-disabled inheritance, stable surviving weights after graph edits, deterministic seeded proposals, valid parameter domains/ports, balanced birth/development failure, and one-time birth/death events. Archive mutation attempts, realized changes, parental/child genome IDs, resource transfers, and eventual outcomes so an observed adaptation can be traced to inherited variation.

## 5. Body evolution

Evolve the inherited construction blueprint. An offspring develops the resulting anatomy using real resources. Ordinary genetic mutations do not instantly replace an adult parent's body; growth, injury, repair, and later developmental plasticity are distinct lifetime processes.

### Initial encoding and physical body

Use a rooted tree of body modules as the first restricted body graph. One vital core anchors attachment branches; every other module names its parent attachment. This makes connected development and subtree edits manageable. More general connection graphs or tissue/cell encodings are later alternatives rather than prerequisites.

Each module gene contains stable identity, module kind, parent attachment, local position/orientation, dimensions, bounded functional parameters, developmental timing, and any sensor/actuator port IDs. Start with simple circles/capsules and explicitly represented connecting tissue where a connection spans space. Anatomical geometry controls contact, mouth reach, force application, and the spatial origin/direction of sensing.

Initial phenotypes are compound rigid bodies in a damped 2D medium, with functional modules attached in the root frame. Propulsion applies forces at the expressed attachment location, affecting translation/turning through the chosen physics model. Articulated joints, active limb segments, soft tissue, and detailed fluid mechanics arrive only when a corresponding experiment needs them. A propulsion module initially represents an abstract actuator, not a full simulation of a flagellum or fin.

Validate connected attachments, finite positive dimensions, supported module types, and the declared policy for noninterpenetrating structural geometry. Derive mass/inertia from expressed geometry and modeled tissue density, rather than letting a mass gene independently become zero while retaining strength and protection. Use fixed density/material properties initially; future material changes require coupled strength, mass, and upkeep rules.

### Functional vocabulary

Start with structural tissue, propulsion, local sensing, ingestion, storage, and protective tissue. Fixed producers already require a declared photosynthesis/absorption mechanism; its evolvable modules and detritus-processing modules are added when those populations become evolvable.

| Module | Example mutable properties | Physical/ecological effects to account |
| --- | --- | --- |
| Structural tissue / connector | Size, shape, attachment geometry | Material and maintenance, collision shape, mass/inertia, vital integrity |
| Propulsion | Placement, orientation, tissue investment, bounded output capacity | Force/torque, construction/upkeep, funded work, resistance/mass burden |
| Local sensor | Placement, direction, range, field width, sensitivity | Local observation access, tissue and sensing/neural costs, occlusion policy |
| Intake / digestive investment | Aperture, placement, processing capacity, substrate allocation | Contact access, ingestion rate, compatibility, assimilation/waste, tissue and processing costs |
| Storage | Capacity and placement | Space/tissue requirements, usable reserve capacity, carried mass |
| Protective tissue | Coverage, thickness, material investment | Damage resistance, exposed area, construction/maintenance, mass/movement costs |
| Producer / recycler tissue later | Capture/uptake area, processing investment, substrate compatibility | Light/resource access and finite conversions with construction/maintenance tradeoffs |

The genotype may encode a module identity, but actual performance comes from geometry, physiology, available resources, and control. An intake module cannot consume a distant resource just because it is labeled mouth. Every new module must define these mechanisms before evolution can use it.

### Mutation operators

First evolve scalar traits on fixed anatomy. Introduce structural operators separately at M3:

| Operator | Concrete example | Representation / inheritance rule |
| --- | --- | --- |
| Resize / parameter perturbation | Slightly larger core, stronger actuator investment, wider sensor field | Preserve module/port IDs; update geometry and coupled costs |
| Reposition / rotate | Change a sensor direction or propulsion lever arm | Preserve functional identity; recompute local geometry and physical consequences |
| Add a module | Attach a small sensor, storage compartment, or actuator | New stable IDs, explicit developmental plan, resource-funded construction, new declared ports |
| Delete a subtree | Remove one appendage and its dependent parts | Never leave dangling attachments/ports; preserve unaffected genes |
| Duplicate a subtree | Copy an existing sensor/actuator branch with its construction parameters | Fresh IDs with recorded origin; copy permitted local wiring while retaining the original branch |
| Reattach a subtree | Move an existing branch to a different valid attachment | Disallow cycles/detached tissue in the initial tree encoding; preserve branch IDs |

Root deletion is not a legal tree operator. Poor but well-formed designs remain possible and face normal development/survival; malformed references are rejected according to the logged mutation policy. Mutation does not predict what anatomy is useful. Do not silently grant an unrequested compensatory improvement when another trait changes.

### Development, cost, and brain coordination

The juvenile starts with its funded initial tissue, then grows toward the inherited developmental plan as reserves and timing permit. A larger target body does not appear fully built for free. An organism may fail to reach maturity if its intended anatomy is too expensive. Functional sensory/action access begins only when the corresponding module develops and receives its declared operating budget.

Each module has construction, maintenance, and operating costs. Larger bodies need more material; stronger movement consumes more energy; longer-range sensing costs more; armor adds maintenance and movement burden. Initial coefficients are model parameters to test, not assumed biological truths. Movement accounting separates chemical expenditure, mechanical storage/work, and dissipation. Selection can favor losing expensive parts as well as adding them.

Keep body-to-brain wiring stable with persistent module and port IDs. A new sensor creates a sensory port; a new controlled actuator creates an action port. Removing a branch removes only its ports and dependent neural connections. Preserve all unaffected inherited weights and their meanings. A structurally valid new part may initially be poorly controlled; its mere existence does not grant an intelligent behavior.

For a permitted branch-duplication operator, optionally copy its explicit associated local sensor/motor connections and weights into fresh gene IDs, with a logged mapping. This is an inheritance operator whose wiring must be specified, not a hidden controller that understands the new anatomy. General local-circuit duplication is a later neural operator; do not copy the whole brain by default or create uncounted neural computation. New ports without copied wiring use explicit bounded initialization and must adapt through later inheritance/learning.

Body changes, sensor/actuator activation, and matching neural-port changes commit at the shared Rust/PyTorch topology barrier. The Rust decoder builds the physical phenotype and budgets; the Python registry constructs/updates neural tensors according to stable IDs. The viewer renders that actual expressed body, including growth and damage, rather than only coloring a generic organism by genome.

### What evolution can discover

For illustration, a lineage might inherit a slightly different sensor orientation, later a duplicated propulsion branch, and later reduced storage. Better food access could support more descendants, while extra mass/upkeep could instead eliminate the lineage. These outcomes follow the modeled geometry and accounting; no script declares the lineage more evolved or turns its analysis label into a different species class.

The first model evolves assemblies from designer-defined functional primitives. It cannot discover an organ, material, or physical interaction absent from that vocabulary. Later developmental grammars can encode repeated/branched structures, and a cell/tissue model could allow a different kind of anatomical novelty, but each changes the genotype-to-phenotype map and requires new validation. Complexity growth, symmetry, articulated locomotion, and novel trophic roles are possible research outcomes only within the implemented mechanisms.

## 6. Brain evolution and lifetime change

Three mechanisms must remain distinct:

| Mechanism | What changes | When | Inherited? |
| --- | --- | --- | --- |
| Neural dynamics | Activations and recurrent state | Every controller update | No |
| Lifetime learning/development | Expressed weights; later neuron and connection growth/pruning | During life | Rules inherited; acquired state reset by default |
| Evolution | Initial weights, neuron parameters, topology, and learning/development rules | Offspring creation | Yes |

The target controller is a sparse recurrent rate network with leaky neuron state, evolving topology, and later bounded neuromodulated plasticity. Begin with a small fixed-topology recurrent controller and evolving initial weights; verify its feed-forward subcase first. Then introduce structural mutations that add/remove neurons and connections or change neuron parameters. Use stable gene identifiers and topology mutation ideas informed by NEAT [1]. This will be an ecological neuroevolution system, not an unchanged implementation of NEAT's population selection algorithm.

Inputs describe only locally sensed information and internal condition: nearby resource cues, contact, stored energy, damage, and maturity. No global map, species label, or privileged instruction to chase a particular organism. Outputs request thrust, turning, feeding, and allocation to growth, repair, or reproduction. The simulation checks whether requested actions are physically and metabolically possible.

Add bounded local synaptic learning after inherited controllers work. Evolve learning parameters and compare learned controllers with fixed-weight recurrent controllers [2]. Start with local correlation-based rules; any later reward signal must come from explicitly modeled internal signals rather than a hidden global task score.

Add lifetime neuron growth and pruning last, through inherited developmental rules with resource costs and limits. Learned weights and acquired neurons do not automatically enter offspring genomes. Any experiment with acquired-state inheritance must be separately labeled.

Brain size and activity incur modeled maintenance and energy costs. Numerical bounds prevent unstable activations and weights. Computational limits are recorded as experimental constraints, and hitting a limit is reported rather than presented as a biological discovery.

### Concrete ANN and weight-update proposal

No externally labeled or pretrained dataset is required for the baseline. Organisms nevertheless generate experience: local observations, internal condition, actions, and subsequent outcomes. Evolution also requires many organism/environment evaluations. Absence of an external dataset does not imply an absence of data, learning signals, or computational cost.

Use sparse sensory connections into recurrent computational neurons, with designated output neurons mapped to actuator/allocation ports. A leaky update mixes previous neural state with bounded activation of current sensory and recurrent input. For example, a candidate rate-network equation is h_next = (1-a)*h + a*tanh(W_effective*h + U*s + b), with a = 1-exp(-neural_dt/tau). U and W are sparse evolved wiring, s contains permitted sensor/body observations, and tau is bounded positive. Initially validate this synchronous reference equation before attempting faster batching. Leaky bounded neurons do not guarantee globally stable dynamics or intelligent behavior.

Keep inherited birth weights separate from acquired plastic offsets. For an enabled plastic connection i -> j, a proposed first local rule is:

```text
effective_weight = clip(birth_weight + plastic_offset, -weight_limit, weight_limit)
eligibility_next = (1-beta)*eligibility + beta*pre_activity*post_activity
plastic_offset_next = clip((1-rho)*plastic_offset
                           + eta*modulation*eligibility_next,
                           -plastic_limit, plastic_limit)
```

This is an initial experimental rule, not a proven optimal survival learner or a verbatim implementation of a cited method. eta controls plasticity, beta controls the activity-trace timescale, rho controls forgetting, and clipping bounds expressed weights. Derive effective per-update coefficients from declared neural time units and test sensitivity to decision rate. Eligibility is local recent-activity state; it does not by itself establish correct causal credit assignment.

The first modulated variant uses a bounded modulatory output of the organism's own network, driven only by permitted sensory/body signals such as actual reserve levels and damage. Its wiring and plasticity parameters are inherited/evolved. The modulator is not a global fitness score, privileged prediction of future survival, or human intelligence label. It can be ineffective; compare with zero-plasticity and unmodulated local-rule controls. Do not silently hard-code a reward like net reserve increase: births, repair, and necessary movement spend reserves and could be penalized by that choice. Any explicit physiological reinforcement formula is a separately declared model variant.

At birth, reset recurrent state, eligibility, and plastic offsets to their declared initial values; offspring receive mutated birth weights and learning-rule genes. During life, preserve state across observations within its permitted history. At each neural update, use current weights for computation, produce actions/modulation, and commit plastic updates for the next update under the authorized resource budget. Expressed topology growth/pruning remains a later developmental operation, separate from both plastic weight updates and inheritance.

Implement recurrent evaluation, eligibility traces, and plastic offsets with PyTorch tensors. Direct local updates do not require autograd or a labeled loss. If later comparing backpropagation, differentiable plasticity, or policy-gradient learning, specify the objective and training experience explicitly. Differentiable/neuromodulated plasticity has research precedent [16, 17], but those studies do not demonstrate that our ecological outer selection will evolve useful plasticity. The fixed-weight recurrent baseline is essential because changing neural state can support adaptation without changing synapses [2].

Candidate selection order is: fixed-topology inherited weights; evolving recurrent topology; bounded local plasticity with evolved coefficients/modulation; then lifetime structural growth. Compare each against its predecessor under the same ecological rules. A successful small fixed-weight controller remains valid even if a larger plastic network does worse.

### Liquid neural network candidates

Include Liquid Time-Constant networks (LTCs) and Closed-form Continuous-time networks (CfCs) in the controller comparison. LTCs are continuous-time recurrent models whose effective response times depend on input and hidden state [18]. This is related to the leaky recurrent baseline, whose initial time constants are fixed parameters between inherited/developmental edits. CfCs offer a related solver-free formulation intended to reduce numerical-solver cost [19, 20]. Do not treat the two models as identical or assume either is the fastest for heterogeneous evolving organisms.

Liquid state dynamics do not automatically constitute online changes to learned synaptic parameters. A fixed-parameter LTC/CfC can produce input-dependent dynamics and temporal state without a weight update. Evolution, explicit plasticity, or an optimizer must still provide the chosen parameter-learning mechanism. Inherited liquid controllers copy the parameterization of their dynamics, not the parent's instantaneous input-dependent effective time constants or hidden state.

Keep the simple sparse leaky recurrent model as the correctness/reference controller. Benchmark sparse CfC next as an efficiency-oriented liquid candidate; use LTC to measure any benefit against its numerical-integration cost. The ncps package provides PyTorch implementations and explicit wiring support [20], making it a useful fixed-topology research baseline. It does not eliminate our need to implement stable gene mappings, topology mutation, cost accounting, or recurrent/parameter-state migration.

Run the same observation/action and inheritance contracts for all candidates. Compare actual parameter counts, connection counts, memory, completed neural update time, resource costs, reproduction/survival, and controlled memory/generalization outcomes. A named liquid neuron can involve multiple parameters or internal operations, so neuron count alone is not a fair compute comparison.

Adding neuromodulated plasticity or arbitrary structural mutation to an LTC/CfC changes the tested model and can invalidate assumptions behind an original implementation or analysis. Begin with inherited fixed-topology parameters, obey constrained parameter domains, then validate each extension. Choose a production controller family from these experiments rather than declaring liquid dynamics equivalent to learning or automatically superior to the reference.

### Brain sizes, instincts, and efficient intelligence

Allow zero neurons as a real phenotype. These organisms have metabolism and development and may survive by photosynthesis, passive absorption, or contact-based uptake if their anatomy permits. They have no neural decisions or hidden scripted foraging policy. Resource-threshold reproduction and constitutive growth are explicit basal physiology rules. Any non-neural chemical regulation added later must be represented and costed explicitly.

A zero-neuron organism is distinct from a zero-hidden-neuron ANN: direct sensor-to-motor connections can implement a controller even without hidden neurons. Report total computational neurons, input/output nodes, hidden neurons, connections, neural-state size, and update rate separately. Sensor/actuator ports are interface metadata; any computation they perform must be declared rather than concealed outside the brain count.

Use descriptive size bands for inspection and benchmarks: zero, 1-16, 17-64, 65-256, 257-1,024, and 1,025-4,096 neurons. These are provisional analysis ranges, not behavioral classes or mandatory growth stages. A small organism can be an effective specialist; a large brain can fail. Do not grant behavioral capabilities simply because a neuron-count threshold is crossed.

Inherited sensor-to-action wiring is our operational definition of an instinct. Learned changes are lifetime adaptation. Instinct count is not inferred from neuron count: a small circuit can encode an effective reflex, and a larger circuit can initially be uncoordinated. Track inherited performance and improvement during life separately.

Start evolving active controllers from minimal viable structures rather than requiring hundreds of neurons at birth. Let useful subnetworks grow, duplicate, and prune. Sparse recurrence and evolvable neural time constants provide candidate mechanisms for memory without requiring a large feed-forward network [1, 2]. Whether a small network achieves a particular behavior must be demonstrated experimentally.

Charge modeled costs for neurons, connections, actual neural updates, and plasticity/development operations. Profile real compute and memory independently of simulated metabolism. Increasing the number of neurons must not grant free sensing range, memory buffers, action speed, or learning machinery. Record average degree and edge-update throughput when benchmarking hundreds or thousands of neurons.

Do not force every lineage to pass from zero neurons to thousands. Initially seed controlled passive organisms and minimal active organisms so ecological accounting and active evolution can be studied. An optional zero-neuron-only origin experiment can later test whether coupled sensor/controller/actuator mutations and developmental intermediates produce a viable neural lineage; it is a distinct, harder experiment.

## 7. Ecological selection and the food web

Organisms reproduce locally when mature and able to fund an offspring. Offspring inherit mutated genomes. Individuals compete through access to actual resources, survival, and reproduction; there is no global tournament that replaces the population every generation.

Begin with mutation-only asexual inheritance. Add sexual reproduction and crossover only after stable gene alignment and compatibility rules exist. This is a staged evolutionary algorithm, rather than forcing crossover into the first ecosystem.

Support consumption of producer biomass, living tissue, and detritus through anatomy, contact, and digestive capabilities. Producers, grazers, predators, scavengers, and decomposers are descriptions of observed activity, not hard-coded behavioral classes. Initial seeded organisms can have different capabilities, but their descendants are not locked to a role label.

Construct the food web from recorded transfers of consumed biomass and energy between lineages or analysis clusters. Measure the web over time windows so a single accidental bite does not define a permanent trophic relationship.

Descriptive genome clusters help inspect divergence; they do not guarantee biological species. No forced population quotas, diversity bonuses, guaranteed predators, or hidden protection of rare groups in baseline experiments. Environmental heterogeneity and local competition provide opportunities for coexistence, which must be tested.

Expand consumption mechanisms through explicit, costed capabilities: passive dissolved-resource uptake, photosynthesis, contact grazing, filtering suspended organic particles, biting living tissue, scavenging, and detritus processing. Introduce the corresponding resource pools only when their accounting exists. Digestion efficiencies and anatomical investments can evolve, with construction and maintenance tradeoffs; an omnivore does not receive every capability for free.

Create opportunities for adaptive decisions through limited sensing, resource patches that deplete and recover, travel costs, refuges, heterogeneous food value, and changing local competition. Later add ecological changes where recent experience can be useful, such as shifting food availability. World changes occur independently of a creature's brain size or test score. Measure whether neural strategies outperform passive and reflex baselines under those conditions without removing successful simple organisms from the ecosystem.

### Functional diversity and hunting

The implemented foundation supplies inspectable body/physiology presets and a common sensor/action vocabulary connected to inherited sparse ANNs. Start with producers and mobile grazers; add pursuit-capable consumers, ambush-capable consumers, scavengers, and living recyclers as their substrates and physical interactions become available. These are founder designs, not permanent species classes or guaranteed survivors. Keep structural mutation disabled while validating initial capabilities.

Couple different capabilities to costs: size/protection, feeding reach/rate, substrate compatibility, storage, directional sensing, propulsion, turning, and later burst effort/recovery. Mobile photosynthetic organisms trade capture investment against locomotion. Controllers choose actions from local cues; pursuit/ambush scripts remain labeled diagnostic controls. A role label never grants a hidden chase command or concealment advantage. Introduce carcass feeding before living recyclers and partition existing detritus with aggregate decomposition. Measure food-web edges from actual nutrient/energy transfers over time windows.

### Social groups, communication, and cooperative hunting

Each organism keeps its own inherited ANN parameters, recurrent state, and permitted learning. Communication passes through costed environmental emitters/receivers with bounded channel values, range, attenuation, propagation delay, persistence/decay, bandwidth, and reception limits. Begin with discrete local emissions; chemical trails need a later transport/decay contract. Reserve optional signal ports in the first ANN interface, disabled in the initial solitary baseline. Enable them once movement, prey, contact damage, and receiver capabilities are validated.

Signals emitted in one decision window become available at a declared later boundary, avoiding instantaneous communication cycles and iteration-order advantages. Brains cannot read other organisms' weights, hidden states, private targets, or global group membership. Scripted calls are diagnostic controls; inherited or learned message interpretation is evaluated separately. Observational group labels never supply automatic team knowledge or a leader.

Provide scenarios where multiple smaller organisms can attack larger prey. Contact, reach, strength/protection, paid attack rates, movement, and prey defenses determine the outcome. Separate attack damage from ingestion. Resolve simultaneous attacks and feeding against one authoritative tissue inventory, with deterministic arbitration and per-attacker records. Do not grant an automatic pack-size damage multiplier, shared energy reserve, or duplicated carcass. Any food sharing is an explicit resource transfer.

Observe temporary associations, recruitment, repeated partners, coordinated approaches, encirclement, group defense, and competition over kills. Initially infer groups from recorded associations/interactions for analysis. Persistent affiliation or recognition later requires explicit observable cues and memory mechanisms. Cooperation, solitary strategies, and free riding may occur. Compare equal-number/biomass hunters with and without signaling, including cost-matched ineffective-signal controls; measure capture success, per-capita energy return, injuries, survival, and reproduction. Mere co-location around prey is not evidence that communication helps.

### Injury, impairment, and repair

The implemented body mode localizes damage to modules, shows injury, and funds repair. Derive integrity from remaining functional tissue relative to a declared healthy/developed capacity. Record source/attacker, affected module, tick, and resource transfers. Destroyed tissue enters detritus/debris or an explicit ingestion transfer exactly once; displayed health is not a second material pool.

Injured propulsion, sensors, intake, and protection lose their corresponding capabilities. Vital-core failure and starvation follow explicit death rules. Repair requires resources, elapsed time, developmental capacity, and construction losses. Define growth-versus-repair semantics so development cannot silently erase injury. Checkpoints preserve damaged geometry, integrity, repair progress, signal state, and pending interactions.

## 8. Simulation architecture

Keep a headless simulation core independent of rendering. The viewer observes snapshots and issues explicit pause, advance one tick, save, load, and intervention commands. Closing the viewer must not change evolutionary rules.

Suggested modules:

```text
world        environment fields, resource pools, diffusion, light
organisms    physiology, development, body graph, lifecycle
brains       neural evaluation, recurrence, plasticity
genetics     inheritance, mutation, stable gene identifiers
interactions movement, collisions, feeding, damage, reproduction
accounting   material and energy transfers, invariant checks
experiments  seed management, batch runs, interventions, controls
telemetry    lineage records, ecological events, metrics, checkpoints
viewer       world view, organism inspector, time controls, plots
```

Use fixed simulation timesteps, seeded random streams, spatial indexing for local queries, and sparse neural graphs. Collect action intentions before resolving shared-resource competition; use a documented resolution rule that avoids systematic iteration-order advantages. Run controller, ecological, and rendering updates at explicit rates.

Checkpoints include the entire dynamic state, random generator state, genomes, expressed neural state, resource fields, configuration, and simulator version. Promise reproducibility within a declared runtime/version; test cross-platform reproducibility separately.

The selected architecture is hybrid: a Rust world simulation library, a Python/PyTorch neural and experiment layer, and a Bevy viewer. Use each language at a coarse functional boundary, with one authoritative owner for each kind of state. Python/PyTorch are included in the initial architecture, not deferred to optional analysis. Implementation and dependency versions are installed and recorded in [DEPENDENCIES.md](DEPENDENCIES.md). Validate efficiency with end-to-end benchmarks rather than assuming any language or GPU is universally fastest. Provisional population benchmarks are 100, 1,000, and 10,000 organisms with representative bodies and heterogeneous brains; these are profiling scenarios, not performance promises.

Include heterogeneous brain-size distributions in those benchmarks, plus stress cases where every organism has hundreds or thousands of neurons. Record neural connection counts and update frequencies, not only organism counts. A successful 1,000-organism run with tiny brains does not establish capacity for 1,000 large brains. Use sparse adjacency storage and bounded neural update rates initially; choose acceleration only after profiling representative workloads.

### Recommended technology stack

| Layer | Proposed choice | Purpose |
| --- | --- | --- |
| Authoritative world | Rust library in a Cargo workspace | Spatial indexing, ecological fields, physiology, resource accounting, fixed-timestep updates, interaction resolution |
| Inherited genetics | Rust | Body/brain genomes, stable gene identifiers, mutation, offspring creation, and parentage |
| Active neural systems | Python + PyTorch | Tensor-based recurrent execution, learned weights/state, local plasticity, and explicitly designed gradient-based learning experiments [11] |
| Language bridge | PyO3 + rust-numpy/NumPy; maturin packaging | Expose the Rust core as a Python extension with batched array interfaces [9, 10, 12] |
| Viewer and inspection | Bevy; egui integration pending dependency review | Native world rendering, selected-organism inspection, neural views, controls; supports a future 3D world [5, 7]; current dependency status is recorded in DEPENDENCIES.md |
| Body physics | Rapier in the Rust core, initially rapier2d | Contacts and modular rigid-body dynamics as needed; rapier3d is available for a later 3D experiment [6] |
| Persistence | Serde world/genome schemas + PyTorch neural state, coordinated by a versioned manifest | Save the complete experiment at a common tick, including RNG and optional optimizer states [8, 11] |
| Experiments and analysis | Python CLI; NumPy; CSV/JSONL metrics and plotting scripts | Configure experiments, run independent seeds, evaluate subjects, and analyze results |

Keep ecological selection, genetics, and neural update rules explicit. A network is not self-updating merely because it uses PyTorch. Specify weight updates, neuron/connection addition/removal, and changes to learning/development parameters as separate operations with distinct timescales and resource costs. Use PyTorch tensor operations for neural execution and local plasticity; backpropagation is not required for evolutionary mutation or local learning. If using gradient-based learning, define the loss or reinforcement-learning objective and its observations explicitly; survival alone does not provide an automatically differentiable training signal. Do not add a hidden intelligence reward to ecological selection.

Use packed arrays/tensors for source/target indices, weights, neuron parameters, and neural state rather than a Python object per neuron or connection. Start with batched PyTorch operations. Benchmark packed disconnected graphs versus size buckets with padding/masks for heterogeneous topologies. Do not assume a single dense network shared by all organisms, and do not charge simulated metabolic costs for padding that exists only for computational batching. Keep neural tensors resident on the selected device between decisions where practical, rebuilding only affected groups when topology changes.

Use the physics engine as a component, not as the ecology model. The simulation core owns nutrients, metabolism, light capture, growth, feeding limits, and energy costs. Reconcile applied forces and movement with explicit actuator accounting. Explicitly save any physics state necessary for replay and verify continuation rather than assuming whole-simulator reproducibility.

Pin compatible Rust, Python, PyTorch, bridge, viewer, physics, and RNG versions at implementation time. Provide a PyTorch CPU execution path and benchmark GPU execution where supported. Include host/device transfer, synchronization, packing, and topology-update costs in those timings [14]. Keep relaxed floating-point transformations disabled initially while validating conservation and repeatability. Bound Rust worker pools, PyTorch CPU threads, and parallel experiment workers to avoid oversubscribing the machine. Record GPU/backend/determinism settings; identical seeds do not guarantee bitwise-identical results across devices or versions [15].

If tiny or irregular controllers remain a bottleneck, benchmark a Rust sparse execution backend against the PyTorch reference using the same genome, neural equations, observations, and update schedule. Add it only if end-to-end results justify maintaining a second implementation. Python/PyTorch remain the research and learning interface. Custom GPU kernels and GPU ecological fields are later optimizations driven by profiling, not initial requirements.

### Rust/Python state ownership and update boundary

Rust owns living bodies, resource pools, inherited genomes, genealogy, and the simulation clock. The Python neural registry owns expressed neural tensors, recurrent state, learned weights, and any optimizer state. Rust also tracks validated expressed topology and neural update budgets for biological costs. Pair each controller with an organism identifier and topology version; assert agreement at every exchange. Learned state is not silently copied into offspring genomes.

One controller decision proceeds as follows:

1. At a fixed decision boundary, Rust reserves the affordable neural update budget and publishes an ordered observation batch, organism IDs, topology versions, permitted update counts, and birth/death/topology events. Candidate actions depend on this consistent world snapshot. Unfunded neural updates are skipped according to explicit physiology rules.
2. Python creates controllers for births from inherited genome data, retires dead controllers, and applies committed structural changes. PyTorch advances recurrent computation and permitted learning using only the declared observation/history interface.
3. Python submits one action batch plus required neural-update accounting. Rust validates IDs, versions, permissions, and resource budgets, settles the reserved neural cost without double charging, resolves competing actions, and advances the configured number of physics/ecology substeps. An invalid exchange aborts the tick or restores its checkpoint rather than retaining unauthorized learned changes.
4. The next decision boundary reports resulting observations and ecological events. Newborns first act at the next eligible decision boundary. Reproduction and mutation use inherited genome information rather than mature learned tensors.

Python controls the experiment loop, calling the Rust extension for whole batches of native work. Avoid Python callbacks inside per-organism Rust loops, one bridge call per neuron, HTTP requests in the simulation loop, or separate Python processes for every organism. Use the same world-update contract for interactive and headless runs. Physics, ecological, and controller update rates are explicit; batching must not silently skip neural decisions or alter biological time.

CPU NumPy arrays can share storage with PyTorch through torch.from_numpy when dtype/layout requirements are met [13]. This does not imply zero-copy access from arbitrary Rust structures or zero-copy CPU/GPU transfers. Prefer stable contiguous observation/action buffers with explicit ownership and lifetimes; release views before reuse or resizing. Start with safe snapshot copies where necessary, then optimize measured transfer costs without introducing concurrent mutation of borrowed buffers.

Structural changes use a synchronization barrier. Inherited mutations are emitted by Rust at birth. Later lifetime neural growth/pruning is proposed by the neural/development layer, validated and resource-funded by Rust, and committed with a new topology version before execution resumes. Preserve surviving neuron/connection identifiers, state, weights, and compatible optimizer state. Rebuild parameter groups or optimizer state explicitly when PyTorch parameters are replaced; a dynamic autograd graph does not perform genetic/developmental edits automatically.

Checkpoint at a common tick/topology barrier: Rust world state, Python neural/optimizer state, all RNG states, backend configuration, and schema/simulator versions must agree. Publish the checkpoint manifest only after all components are saved. Restore and test the combined state, including birth/death queues and learning history. Baseline tests use a controlled CPU configuration; GPU reproducibility is evaluated separately [15].

The Bevy viewer is a separate observer process receiving a bounded, versioned local binary snapshot stream at display frequency, with detailed neural data requested only for selected organisms. Stale display frames may be dropped; biological updates may not. Viewer commands enter the experiment controller at explicit tick boundaries and are logged as interventions. Headless runs do not construct rendering entities or send display snapshots. Do not embed a second copy of Python/PyTorch in the viewer or duplicate the authoritative world.

### First integration and performance gates

At milestone 0, establish the Cargo workspace, Python package, PyO3/maturin build, typed batch schema, and a minimal PyTorch controller round trip. This is a bridge/contract validation rather than a claim of evolved behavior. Then build the resource cycle and basic Bevy observer before expanding controller complexity.

Measure simulated time per wall-clock second, births/generations reached, native world-update time, neural execution/learning time, bridge overhead, host/device transfer and synchronization, topology rebuilds, rendering overhead, and memory consumption. Use representative neuron/connection distributions and repeated seeds. GPU timing must wait for the work being measured rather than reporting only asynchronous kernel launch time [14].

Acceptance gates include a batch-order/ID contract check, stale-topology rejection, neural reference comparisons, resource conservation, CPU save/resume continuation across both languages, and confirmation that rendering frequency does not alter simulation state. Optimize the dominant measured cost; language count is not an efficiency metric.

### Why 2D first, and what a 3D version changes

The agreed starting world remains 2D. This prioritizes observable survival and evolution over locomotion complexity. A planar world reduces the number of spatial variables in movement, sensing, collisions, body development, and environmental fields. That is expected to reduce implementation and simulation costs for comparable model fidelity, but actual throughput must be measured.

Full 3D is appropriate if vertical habitats, climbing, flight, body balance, or volumetric swimming become central research questions. It adds orientation and body-dynamics requirements, a larger spatial search problem, and potentially volumetric resource fields. A ground-based 3D world can retain surface resource maps; a water-column world may require actual volumetric fields. These are distinct ecological models with different costs.

3D rendering of organisms confined to a plane is an optional presentation mode, not a 3D evolutionary environment. Switching to full 3D requires new body attachment geometry, sensors, actions, collision/physics configuration, developmental constraints, and validation scenarios. Some evolved controllers may transfer poorly and need further evolution. Bevy and Rapier support both dimensions [5, 6], while the Python/PyTorch neural layer remains behind the same versioned interface. This is not a renderer toggle or a cost-free migration.

Retain dimension-independent genome identifiers, resource accounting, neural interfaces, archives, and experiment protocols. Keep the spatial implementation concrete and simple for the initial 2D world rather than building two physics implementations before the resource cycle is validated. If the scope changes to 3D-first, begin with simple aquatic or ground-based modular bodies and limited sensing rather than detailed articulated locomotion and full fluid simulation simultaneously.

## 9. Milestones and acceptance gates

| Milestone | Deliverable | Gate before proceeding |
| --- | --- | --- |
| 0. Model and integration contract | Units, resource transfers, update order, genome/state schemas, Rust/Python batch bridge, minimal PyTorch round trip, reference scenarios, performance targets | Every lifecycle operation has explicit resource sources/destinations; organism IDs and topology versions survive a correct batched round trip |
| 1. Living resource cycle | Light, nutrients, controlled producers, simple feeders, growth, reproduction, death, aggregate decomposition; minimal viewer | Resource audits pass; no-light controls exhaust usable energy; nutrient recycling is observable; seeded runs/checkpoints reproduce |
| 2. Heritable adaptation | Scalar body/physiology mutations, fixed-topology ANN with evolving weights, ancestry records | Repeated experiments show inherited changes affecting reproductive success, supported by common-environment comparisons |
| 3. Evolving anatomy | Module-graph mutations, functional sensors/actuators, development and anatomical costs | Descendants show functional body differences; body changes preserve valid neural ports; resource audits still pass |
| 4. Evolving neural structure | Variable recurrent topology, neuron/connection mutations, neuron-parameter mutations, neural inspector | Topology differences affect behavior in controlled comparisons; neural growth cannot bypass modeled costs |
| 5. Evolving food web | Evolving producer and decomposer populations, broad feeding mechanics, trophic-flow records | Observed transfers support reported trophic roles; coexistence and extinction rates are reported across seeds |
| 6. Lifetime adaptation | Evolving synaptic learning rules, then developmental neuron growth/pruning | Isolate learning, inherited changes, and neural state with matched controls; verify acquired state resets in offspring |
| 7. Long experiments | Batch runner, robust save/resume, profiling, lineage archives, intervention tooling | Long-run accounting remains valid; computational limits are reported; outcomes include unsuccessful seeds |
| 8. Secondary subject evaluation | Archived candidates, separate validation/test environments, capability profiles, reproducible evaluation packages | Reproducible comparisons with declared baselines; report size, costs, uncertainty, and failure cases, including absent cognitive gains; test rankings never drive ecological selection |
| 9. Usable local product | Launcher, viewer/headless modes, versioned presets, complete save/load, exports, packaged components, installation/model guides | Repeatable supported-platform installation and example workflows; validated contracts/accounting; explicit experimental and performance limits |

Every milestone includes enough inspection to explain its behavior. Milestones 0-1 produce the inspectable resource-cycle prototype; milestone 2 adds inherited adaptation; milestones 3-5 add evolving anatomy, neural structure, and food-web capabilities. Milestone 6 completes the requested lifetime-changing brain. Milestones 7-9 produce long-run experiments, secondary subject evaluations, and a usable packaged local application. Later extensions include seasons, multiple limiting nutrients, articulated bodies, sexual reproduction, disease, full 3D, and richer developmental encodings.

Design the evaluation protocol and archive format at milestone 0, begin collecting candidates at milestone 2, and add task coverage as neural mechanisms become available. Final evaluation follows milestone 6 and sufficient experiment throughput. Fixed-weight stages can test memory and inherited generalization but cannot establish synaptic learning.

## 10. Evidence of evolution and ecosystem function

Track population and biomass, births and causes of death, lifetime reproductive output, resource budgets, trophic transfers, body traits, neural sizes, mutation histories, lineage survival, and behavioral diversity. Retain sampled ancestral genomes for later comparisons.

Use repeated independent seeds and report variation, extinction frequency, and selection of any showcase runs. Analyze adaptation separately from random drift. Replay ancestors and descendants in common environments, with repeated trials, and include mixed-population competition where interactions matter. Frozen historical opponents can help evaluate changing predator/prey relationships without treating one fixed benchmark as universal fitness.

Core controls include mutation disabled, body evolution disabled, brain evolution disabled, and plasticity disabled. Remove light to check energy dependence. Disable decomposition to test nutrient lock-up. Change resource distributions and test whether inherited strategies respond. Check that conclusions survive smaller timesteps and reasonable parameter changes.

At milestone 0, define numeric experiment horizons, seed counts, conservation tolerances, and effect criteria. Do not invent success thresholds after seeing the results. Increased neuron count, body size, or visual complexity alone is not evidence of better adaptation.

### Intelligence evaluation and final test subjects

Define intelligence operationally as effective behavior under uncertainty, use of relevant history, and adaptation or transfer to changed conditions. This definition supports specific measurements rather than claiming a universal intelligence score. Distinguish memory through recurrent activity from learning through changed weights or structure.

Predeclare a versioned suite of embodied tests:

- Efficient foraging: acquire sufficient resources with limited sensing and a constrained movement/energy budget.
- Memory: exploit an earlier cue that is no longer observable; randomize trials so current observations alone cannot identify the correct action.
- Flexible food choice: respond when resource payoffs change, while holding the body and digestive capabilities fixed within a trial.
- Risk-sensitive decisions: trade resource gain against locally detectable threats without privileged threat labels.
- Generalization: transfer to unseen layouts, patch configurations, and environmental seeds.
- Lifetime learning: improve over repeated experience; compare against clones with plasticity disabled and matched initial state.

Tests must be solvable through the subject's actual sensor and actuator interface. Test environments use valid resources and physics rather than presenting unsupported symbolic inputs. Publish baselines for passive physiology, simple reflex controllers, small recurrent controllers, and learning-disabled clones as applicable.

Maintain an archive of candidates spanning ecological success, task capability, and resource efficiency across brain sizes. Archive collection is observational and does not award reproduction, energy, or protection in the ecosystem. Select finalists using separate validation worlds. Reserve unseen test seeds and task instances for the final evaluation; do not feed final test results back into evolution or candidate selection. Repeated development requires new test instances or a new suite version.

Evaluate each candidate with its own body first: embodied performance depends on anatomy, sensors, and actions as well as its ANN. Use body-matched controller comparisons to isolate neural contributions. An optional network-transfer experiment may use a declared standard interface and deterministic adapters, but report transfer separately and do not add hidden planning or memory in the adapter.

Compare both absolute capability and efficiency under declared resource budgets. Report neuron and connection counts, simulated energy use, real evaluation compute, survival time, reproduction where relevant, task scores, learning curves, and variation across trials. Avoid ranking all candidates by a single performance-per-neuron ratio, which is undefined for zero neurons and ignores connection density and update costs.

For the optional intelligence study, choose a test-suite subject by a predeclared aggregate of normalized task scores, with normalization anchored to fixed reference scenarios and baselines. Also retain the strongest compact subject and the strongest ecological survivor. Weights and tie-breaking rules are fixed before final evaluation. These are analysis labels and never objectives imposed on organisms. A suite winner is the best measured subject under that protocol, not evidence of general-purpose or human-level intelligence. The ecosystem experiment remains valid if simple survivors persist and no candidate shows cognitive gains.

Export a reproducible subject package containing inherited genome, body phenotype, birth controller, lifetime learning/development rules, optional mature neural snapshot, configuration, simulator version, ancestry, and evaluation results. Evaluate birth-state and mature-state snapshots separately; a mature snapshot may retain experience, while a birth-state clone begins with the declared initial state. Trials never modify the archived source or the original ecosystem organism.

## 11. Observation and experiment interface

The viewer should let us pause, advance one tick, change playback speed, select an organism, inspect its body, genome, active neural graph, energy budget, parentage, and offspring, and follow a lineage. Add overlays for light, free nutrients, detritus, biomass, and actual feeding events.

Provide population histories, trait distributions, lineage trees, and a food-web view as data become available. Mark experimental interventions and reseeding in the timeline. Distinguish display playback speed from simulation timestep, and preserve uninterrupted headless batch runs.

Add candidate archiving, brain-size filters, inherited-versus-learned behavior comparisons, neural-cost views, and a separate evaluation screen. Show survival and cognitive-test results independently, including compact candidates that compete well with larger networks.

### Navigable world and readable scale

The default viewer now has a full-window camera, compact overlays, pan/zoom, fit-to-world, organism selection/following, resource layers and a B-toggle neural inspector. The legacy diagnostic viewer retains its minimap. Scale indicators, richer filters, and group/lineage following remain extensions. Camera viewport and world extent are independent. Larger habitats require declared resource densities and measured computational capacity; enlarging the window alone does not increase carrying capacity.

Render the habitat as continuous 2D space; make the resource grid an optional analytical overlay. Terrain, organisms, carcass/resource patches, and scale-aware detail should make the world readable without requiring every cell boundary to be visible. Functional scenery must agree with the simulation's collision, sensing, light, and resource rules.

Use three display levels: whole-world density/resource/food-flow summaries; medium-distance silhouettes and group activity; close-up expressed anatomy, feeding structures, sensor direction/range, contacts, injury, and repair. Cull off-screen rendering and aggregate distant detail while continuing the full biological simulation. Stream bounded visible-region summaries and selected details rather than every neural graph. Display limits must not delete organisms or suspend distant evolution.

Differentiate organisms through actual shape, size, attachments, orientation, and function. Colour may show diet/lineage or observed groups, with a legend and high-contrast symbols. Wounds, missing modules, contact markers, and repair changes must reflect recorded state/events. Expand inspection with energy expenditure, recent damage sources, food compatibility, current actions, and selected neural/signal state. Observer-only labels and threat/group overlays never become privileged ANN inputs.

Add feeding/damage/signal overlays, sampled event histories, death-cause summaries, trait histories, lineage views, and a food-web panel. Replay must use recorded states/events or deterministic re-execution from a compatible checkpoint. Verify that camera position, zoom, resolution, display rate, and layers do not change biological outcomes.

### Prioritized world improvements

After diagnostics and camera/inspection foundations, add static obstacles/refuges, distinct resource patches, and readable juvenile/adult development with explicit movement, sensing, and resource rules. Cover/occlusion must affect perception/contact when shown as functional terrain. These create opportunities for pursuit, ambush, escape, and memory without assigning the winning strategy.

Day/night cycles, seasonal light changes, and persistent chemical trails remain optional later experiments with continuous-light and solitary controls retained. Detailed fluid dynamics, disease, sexual reproduction, and full 3D stay outside this expansion. Thousands of organisms per role is a profiling target, not capacity already delivered by the viewer.

### Deferred robot side project

After a stable artificial-life release, add a separate evolutionary robotics mode sharing suitable genome, controller, physics, viewer, and experiment interfaces. Robots survive by scavenging usable parts from other robots or accounted wreckage, repairing themselves, and assembling larger bodies. Parts, salvage losses, energy, compatibility, attachment, and manufacturing/reproduction costs need their own conservative rules; reclaimed components cannot exist in two bodies at once. This mode remains deferred while the animal-style ecosystem is developed.

## 12. First implementation slice

Implement milestone 0 and the smallest milestone-1 world: one light field, one limiting nutrient, fixed producers, a simple feeding organism, juvenile growth, resource-funded offspring, death, and aggregate decomposition. Include a basic observer and automated conservation checks.

Use the selected Rust/Python architecture from the beginning: a Rust world library, batched PyO3 bindings built with maturin, a Python experiment runner with a minimal PyTorch controller integration, and a Bevy observer. Keep the first diagnostic controller deliberately simple; ecological accounting remains the first substantive validation gate.

Use deliberately simple diagnostic behavior to validate this resource cycle. Then replace it with an evolving ANN in milestone 2. This isolates ecological accounting failures from controller failures while preserving the intended body/brain interfaces.

Do not begin by combining full anatomy search, variable neural topology, plasticity, complex physics, and a large food web. Each addition needs a working baseline and a comparison that identifies what it changes.

## 13. Readiness and implementation order

The concrete work breakdown and proposed initial parameters are in [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md). Begin with the model contract and a small Rust/PyTorch batch bridge, then validate the resource cycle. Evolving scalar traits and weights precedes module-graph search, structural neural evolution, living recyclers, and lifetime neural development.

Before implementing an interaction, define its inputs, preconditions, units, resource transfers, state ownership, failure behavior, and event log. Before claiming adaptation, define the comparison, independent experimental unit, observation horizon, seed set, and reported uncertainty. Before optimizing, profile a representative mixed-brain workload with rendering disabled and enabled separately.

The learning path assumes existing Python/PyTorch competence. Preparation priorities are practical Rust ownership/data modeling, array interoperability, evolutionary/ecological reasoning, fixed-timestep numerical simulation, and experiment design. Learn Bevy, advanced physics, crossover, and GPU optimization when the corresponding milestone needs them. Completion of a broad course is not a gate; the demonstrated exercises in [LEARNING_GUIDE.md](LEARNING_GUIDE.md) define readiness.

The first deliverable is a reproducible resource-conserving simulation with inspectable births, deaths, and ancestry. The later deliverables add measurable heritable adaptation, evolving bodies/brains, a richer food web, and lifetime learning. A final test subject is a secondary research artifact, never a replacement for the survival-driven ecosystem goal.

Follow the numbered start-to-finish route at the beginning of [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md). The final initial product is a supported local 2D application with interactive and headless execution, actual body/brain inspection, declared experiment presets, coordinated save/resume, ancestry/trophic histories, and subject/result exports. Packaging and example workflows are part of completion; universal intelligence growth and stable coexistence are not promised product behaviors.

## Research basis

These sources inform the design; they do not demonstrate that this proposed ecosystem will exhibit indefinite novelty or stable diversity.

1. Stanley and Miikkulainen, *Evolving Neural Networks through Augmenting Topologies* (2002). Basis for evolving neural weights/topology and aligning structural genes. We borrow representation and mutation ideas while keeping ecological reproduction separate from NEAT's fitness sharing and selection. https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf
2. Stanley, Bryant, and Miikkulainen, *Evolving Adaptive Neural Networks with and without Adaptive Synapses* (2003). Supports evaluating plasticity separately from adaptation through recurrent state; their task results do not establish which method will work best here. https://nn.cs.utexas.edu/downloads/papers/stanley.cec03.pdf
3. Sims, *Evolved Virtual Creatures* (1994 project and linked publications). Precedent for evolving embodied morphology and control, under task-based selection distinct from this ecosystem proposal. https://www.karlsims.com/evolved-virtual-creatures.html
4. Taylor, *Requirements for Open-Ended Evolution in Natural and Artificial Systems* (2015). Motivates attention to viable mutation paths, reproductive robustness, and continued evolutionary opportunity. https://arxiv.org/abs/1507.07403
5. Bevy, *Introduction*. Official description of its Rust engine and 2D/3D scope. https://bevy.org/learn/quick-start/introduction/
6. Rapier, *Getting started* and *Determinism*. Official Rust physics integration and determinism conditions. https://rapier.rs/docs/user_guides/rust/getting_started/ ; https://www.rapier.rs/docs/user_guides/templates/determinism/
7. bevy_egui repository. Bevy integration for egui; select a version compatible with the chosen Bevy release. https://github.com/vladbat00/bevy_egui
8. Serde documentation. Rust serialization/deserialization framework; binary and human-readable formats need separate format implementations. https://docs.serde.rs/
9. NumPy, *Broadcasting*. Describes array vectorization and its memory/performance tradeoffs. https://numpy.org/doc/stable/user/basics.broadcasting
10. PyO3 user guide and rust-numpy. Native Python extensions in Rust and NumPy array integration. https://pyo3.rs/ ; https://pyo3.github.io/rust-numpy/
11. PyTorch, *Automatic differentiation package* and *torch.optim*. Autograd and explicitly managed parameter updates/optimizer state. https://docs.pytorch.org/docs/stable/autograd.html ; https://docs.pytorch.org/docs/stable/optim.html
12. Maturin user guide. Build and package mixed Rust/Python projects. https://www.maturin.rs/
13. PyTorch, *torch.from_numpy*. CPU tensor/NumPy memory sharing and associated constraints. https://docs.pytorch.org/docs/2.15/generated/torch.from_numpy.html
14. PyTorch, *CUDA semantics*. Device operations, data movement, synchronization, and timing implications. https://docs.pytorch.org/docs/main/notes/cuda.html
15. PyTorch, *Reproducibility*. Limits across releases/platforms and deterministic execution controls. https://docs.pytorch.org/docs/2.15/notes/randomness.html
16. Miconi, Stanley, and Clune, *Differentiable plasticity: training plastic neural networks with backpropagation* (2018). Precedent for parameterized plastic connections, with gradient-based training distinct from our baseline ecological outer loop. https://proceedings.mlr.press/v80/miconi18a.html
17. Miconi, Rawal, Clune, and Stanley, *Backpropamine: training self-modifying neural networks with differentiable neuromodulated plasticity* (ICLR 2019; arXiv version 2020). Precedent for network-controlled neuromodulation, not validation of the proposed sparse ecological learner. https://arxiv.org/abs/2002.10585
18. Hasani et al., *Liquid Time-constant Networks* (AAAI 2021). Input/state-dependent continuous-time recurrent dynamics, evaluated with numerical integration. https://ojs.aaai.org/index.php/AAAI/article/view/16936
19. Hasani et al., *Closed-form continuous-time neural networks* (2022). Related continuous-time models that avoid the numerical ODE-solver requirement. https://www.nature.com/articles/s42256-022-00556-7
20. ncps official quickstart and implementation. PyTorch LTC/CfC models, temporal state, and structured wiring. https://github.com/mlech26l/ncps/blob/master/docs/quickstart.rst ; https://github.com/mlech26l/ncps

The approved circle repair takes precedence over earlier germline-only inheritance and anatomy-first proposals in this document. The circle policy copies matching current parental learned parameters at the start of a birth decision, then mutates the child. Parent birth genes remain available separately from current execution parameters. Learning state and native world commit atomically. Do not advance claims of useful adaptation based only on neuron counts or changed weights.

## Current implementation and next decisions

The current reference is a circle-world experiment with online parameter learning and acquired-parameter inheritance. A/B work covers long-run failure diagnosis, persistent diagnostic capture and a full-window observer. The original collapse and failed intermediate interventions remain evidence; current results are linked from README. The three action outputs still control only thrust, turning and feeding. Growth, repair and reproduction remain automatic biological policies pending C.

The remaining order is neural agency, functional food web, hunting/defence/local communication, lineage differentiation and lifetime neural development, then scaling/evaluation/release. Distinguish planned, implemented and experimentally validated work. A working mutation operator does not prove speciation; changing weights does not prove useful intelligence.

Thread-like trapping appendages and camouflage are accepted design proposals for later hunting/habitat work. Threads need explicit material, mechanics, breakage and escape. Camouflage needs heritable traits plus local background-dependent perception. Richer terrain must affect movement, sensing, shelter and resource availability, not merely appearance. Preserve the simple circle reference as a control when those features are introduced. See IMPLEMENTATION_ROADMAP for the capability gates and conditions.
