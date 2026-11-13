# World: implementation roadmap

Current status: the remaining prototype software now includes neural agency/development, richer feeding and food-flow records, habitat/hunting/communication mechanics, candidate archives, controlled studies, subject evaluation and release packaging. [Expanded verification](artifacts/setup/ecosystem-expanded-verification.json) records execution evidence and limitations. The original build order below remains the design rationale. The expanded food-web trials do not meet a sustained-diversity research goal: predators/scavengers/recyclers disappeared, and one seed retained only an immature consumer. Installation/execution correctness and ecological/cognitive research outcomes remain separate gates. The walkthrough remains unchanged.

The Python/PyTorch environment, Rust toolchain, and Windows SDK are installed. Package, bridge, resource, and habitat contract checks establish the current foundation. The diagnostic cycle and basic observer/continuation are complete. Inherited sparse controllers and stable body/neural mutations now execute through validated local ports. The circle reference uses swept contacts, bounded funded development and conservative mineral diffusion. Legacy compound bodies use conservative contacts and an overdamped force/torque model; momentum/articulation, other transport processes and Rapier integration remain deferred. The dependency-scaffold build remains historical evidence. Exact versions, audit reports, deferred UI integration, and rebuild instructions are in [DEPENDENCIES.md](DEPENDENCIES.md).

## 0. Start-to-finish route

This is the user-facing build order. The detailed M0-M9 work packages below specify the contracts and checks behind it. Learn only the next stage's prerequisites; do not wait to master the entire stack before making a small working artifact.

| Area | Work | Tangible result / gate |
| --- | --- | --- |
| Scope and rules | Confirm the initial 2D scope, final-product features, and survival-driven rules; practice Rust ownership/data modeling using the Python resource-ledger exercise | A clear initial scope and a small understood state/transfer model |
| Build environment | Prepare a compatible Rust/Python toolchain, Cargo workspace, Python package, locked dependencies, and reference configuration | Repeatable local builds and a documented project layout |
| Native exchange | Build a toy batched Rust -> PyTorch -> Rust round trip with IDs, versions, and array contracts (M0A) | Actions reach the correct organism; invalid/stale exchanges are rejected |
| Resource accounting | Implement units, reserves/tissue, resource transfers, finite rates, and audits (M0B) | Individually correct feeding/growth/birth/death transactions before population complexity |
| Spatial habitat | Build a small headless habitat with light, minerals, spatial queries, and controlled producers (M1A) | Resources generate funded living tissue with inspectable inventories |
| Ecological cycle | Add diagnostic consumers, maintenance, growth, maturity, funded reproduction, death, and aggregate decomposition (M1A) | A complete resource cycle with no magical food or duplicate death/birth transfers |
| Observer and continuation | Add a basic Bevy observer and coordinated checkpoints (M1B) | Inspectable organisms/resources; pause/advance/save/load; viewer-independent continuation |
| Inherited controllers | First diagnose consumer extinction, add functional founder presets/basic injury records and camera navigation; then connect small inherited ANNs through a shared local interface with optional signal ports (M2A) | Inspectable survival bottlenecks and differentiated capabilities; ANN actions and inherited parameters are validated before mutation |
| Adaptation | Enable birth-time scalar/weight mutations, lineage records, and controlled adaptation experiments (M2A/M2B) | Traceable inherited variation and evidence evaluated across controls/seeds |
| Body development | Add funded module-tree development, physical attack/defense, local injury/impairment/repair, structural mutations, and close-up anatomy inspection (M3) | Actual body differences and visible functional damage, with stable ports, preserved weights, and conservative transfers |
| Neural topology | Add neuron/connection mutations and heterogeneous-controller batching (M4) | Variable neural topology and cost accounting; zero/small/larger networks execute correctly |
| Food web | Expand diets, producer/recycler populations, hunting opportunities, local communication, and cooperative hunting of larger prey (M5) | Measured trophic flows and social interactions; independent brains and costed signals, with no guaranteed teams or coexistence |
| Lifetime adaptation | Add bounded lifetime plasticity, controlled acquired-weight inheritance experiments, and then neural growth/pruning (M6A/M6B) | Distinguishable inheritance, recurrent memory, learning, and development, with funded costs |
| Long experiments | Run long seeded experiments; profile mixed bodies/brains, social interactions, spatial queries, and a viewer with scale-dependent detail (M7) | Reproducible throughput/memory limits and full simulation of organisms outside the camera view |
| Subject evaluation | Evaluate archived copies on unseen individual and matched social tasks, including communication-disabled controls (M8) | Reproducible ecological/compact/capability packages and evidence separating social benefit from simple aggregation |
| Release | Package and document the local application, presets, exports, and experiment runner (M9) | A usable, installable version of the simulator with a clear start command and tested example workflows |

Check correctness throughout implementation, not only before release. Initial performance measurements begin with the bridge and first population; M7 is the systematic scaling/long-run phase. Candidate collection begins early; M8 is the final held-out evaluation phase. The basic viewer is implemented; its inspection/analysis features grow with each later capability. The producer-only control retains the original 64 x 64 geometry; the diagnostic ecosystem uses 24 x 24 for denser initial interactions.

The finished initial product is a local 2D artificial-life research application with interactive and headless modes, differentiated anatomy/diets, visible injury/repair, and explicit local social-signal/group-hunting experiments. It should display actual bodies/neural state, preserve inherited parameters, log ancestry/trophic/social events, save/resume complete runs, and export results. The viewer must navigate larger worlds at multiple detail levels. Coexistence, effective communication, and increasing intelligence remain measured outcomes.

The route retains sixteen stages. Foundations through stage 11 exist, with partial behavioural validation; the staging remains: stage 8 adds diagnostics, functional presets, early navigation/inspection, and reserved signal ports before neural control; stage 10 supplies anatomy/contact damage/repair; stage 12 adds executable local signaling and cooperative food-web experiments; stage 14 validates larger populations and scalable rendering. Viewer improvements begin now and grow with those capabilities rather than waiting for final release. Stages 9, 11, 13, 15, and 16 retain mutation, topology, lifetime adaptation, evaluation, and packaging with the expanded contracts.

## 1. Fixed direction and experimental choices

Fixed direction: survival-driven 2D ecology; one shared organism model; finite material with external light input; bodies and brains inherited together; zero-neuron organisms supported; Python/PyTorch included; Rust owns world/genetic state; Bevy observes; final intelligence tests are secondary.

Do not lock unknown biological coefficients or throughput targets into claims of realism. The following are proposed debugging defaults, to be calibrated and versioned before scientific comparisons:

| Item | Proposed first setting | Verification / reason to revise |
| --- | --- | --- |
| Habitat | Bounded square, 64 model-length units per side | Test wall accumulation and boundary sensitivity |
| Environmental fields | 64 x 64 cells; one limiting mineral nutrient and aggregate detritus | Verify conservative transfers; compare a finer grid before spatial claims |
| Initial population | 64 controlled producers and 16 diagnostic consumers | Inspect individually; fund all initial tissue/reserves from the declared initial inventory |
| Neural population | Minimal active controller; passive producers have zero neurons | Declare sensor/input/output counts before setting total neural size |
| Physics timestep | 1/60 model-time unit | Halve timestep for sensitivity checks; do not equate model time with biological seconds |
| Neural decisions | Every 6 physics ticks | Document six-tick action hold; compare higher decision rates where behavior is sensitive |
| Ecology-field updates | Every 6 physics ticks initially | Derive stability/positivity limits for the chosen diffusion/uptake method and parameters |
| Display | Target 30 display updates per wall-clock second when interactive | Display speed has no effect on biological time; skip stale snapshots |
| Precision | f64 resource ledgers; f32 neural tensors initially | Validate finite actions, conservative transfers, and error accumulation separately |
| Inheritance | Asexual; immutable birth genome; reset acquired neural state | Check mutation-disabled clone births and later plasticity-enabled controls |
| Reproduction | Local resource-funded birth at defined maturity/investment thresholds | No food grants or population-filling replacement births |
| Mortality | Explicit vital-tissue and unfunded-maintenance rules | Define reserve use and permitted tissue catabolism before starvation tests |
| Decomposition | Controlled aggregate process first | Living recyclers later replace/partition the same substrate budget |
| Compute | CPU reference path; GPU measured using actual available hardware | Include transfer, packing, synchronization, topology edits, and learning |
| Diagnostic experiments | Fixed saved seeds and short inspectable horizons | Suitable for debugging, not evidence of general adaptation |
| Initial research batch | Provisionally 20 independent world seeds per condition | Choose horizons/effect criteria before analysis; increase based on variability and question, not a desired result |

Before the first research batch, record actual numeric rates, construction costs, initial inventories, mutation distributions, observation horizon, sampling schedule, tolerances, and computational ceilings. These remain explicit model assumptions. Rate constants must work at the selected timestep without producing negative pools or unlimited per-tick consumption.

## 2. Proposed source layout

The full layout below remains the target. The accounting core, bindings, Python runtime, and diagnostic brain registry exist. Genetics now owns stable body/neural genes, clone inheritance, and validated birth-time mutations. Pilots and neural runtime modules exist; bounded lifetime weight learning is implemented; lifetime neural growth, social food webs, and full analysis/release remain planned; the reference/configuration/tests remain available.

```text
crates/world_core/       authoritative world, resources, physiology, interactions
crates/world_genetics/   genome types, mutation operators, development instructions
crates/world_bindings/   PyO3 array bridge; no rendering dependency
crates/world_viewer/     Bevy observer and control client
python/world/brains/    PyTorch controller registry, packing, recurrence, plasticity
python/world/experiments/ runners, seeds, controls, evaluations, checkpoint coordinator
configs/                versioned world, mutation, physiology, and experiment settings
tests/                  contract/integration and conservation scenarios
analysis/               independent analysis of exported experiment results
```

Keep world_core and world_genetics independent of Python and Bevy APIs. The bindings translate batches; they do not contain a second copy of metabolism. The Python neural layer is not given a mutable Rust world reference to inspect arbitrarily. The raw habitat checkpoint preserves Rust-owned state. The coordinated world.ecosystem envelope now links it to sparse recurrent memory at the same decision tick. Birth random draws are counter-derived and neural execution has no mutable RNG stream. The diagnostic bridge retains its separate checkpoint format.

## 3. Data contracts before feature work

| Contract | Required information | Invariant |
| --- | --- | --- |
| ExperimentConfig | Schema version, simulator revision, units, clocks, coefficients, limits, seeds, enabled mechanisms | Every reported run identifies its exact configuration |
| OrganismId / GeneId | Stable IDs with deterministic creation rules | Deleted array positions are not reused as biological identity |
| Genome | Body graph, physiology genes, neural genes, learning/development genes, reproduction genes | Inherited source is separate from expressed/lifetime state |
| BodyModule | Module ID, attachments, geometry, tissue resources, ports, development status | Body remains representationally valid; construction and maintenance are funded |
| NodeGene / ConnectionGene | Stable IDs, node kind/parameters, endpoints, initial weights, enable state | References are valid; recurrence has explicit evaluation semantics |
| OrganismState | Genome reference, expressed body, reserves, damage, maturity, age, lifecycle state | No duplicate inventory or active dead organism |
| ObservationBatch | Tick, ordered organism IDs, topology versions, port layout/masks, local observations, update authorization | No global food map, analysis class, test score, or hidden planner input |
| ActionBatch | Matching tick/IDs/versions, actuator values, declared allocations, neural update accounting | Finite, correctly shaped, authorized actions; stale versions rejected |
| LifecycleEvent | Birth/death ID, tick, genealogy, mutation summary, resource transaction reference | A birth/death commits once and is visible consistently across Rust and Python |
| TopologyChange | Proposal, stable mapping, reserved cost, new version, commit/abort outcome | Surviving state retained; no neural growth without funding |
| ResourceTransaction | Source, destination, nutrient/energy quantity, reason, tick | Balanced transfer or explicit external input/output; no silent negative clamping |
| SignalEmission / Reception | Source, emission/reception ticks, local position/channel, bounded values, propagation/decay state, costs | Delayed local observations only; no other brain's private state or hidden team/target information |
| DamageEvent / RepairState | Attacker/source, victim/module, tick, tissue/integrity change, debris transfer, repair funding/progress | One authoritative tissue inventory; damage and ingestion are distinct; health is not duplicate material |
| GroupObservation | Time window, observed associations and joint interactions, analysis method | Observer/analysis metadata supplies no biological team privilege |
| ViewRequest / Snapshot | Camera bounds, detail level, selected IDs, common tick, bounded summaries | Culling/aggregation affects rendering only; off-screen organisms continue to simulate |
| CheckpointManifest | Common tick, all component files, versions, hashes, RNG/backend state | No restore from mismatched world and neural snapshots |

Represent masks and shapes explicitly for heterogeneous brains. Account biological costs from expressed nodes/connections and authorized updates, not padded tensor dimensions. Non-finite observations/actions are contract failures, not successful survival strategies. Decide how to abort/recover before an experiment, rather than silently replacing invalid actions with a helpful behavior.

## 4. Work packages mapped to milestones

### M0A: build and integration foundation

Build the Rust workspace and Python package using PyO3/maturin. Add a minimal Rust array operation callable from Python, then a toy batched observation -> PyTorch action -> Rust state update round trip. Use organism IDs and topology versions from the beginning. Confirm clean rebuild/import on the selected Windows toolchain.

Deliver: a headless runner, config loader, minimal controller registry, and tiny integration fixture. The fixture establishes the interface, not evolved behavior. Verify batch reordering, stale-ID/version rejection, safe buffer ownership, and a CPU save/resume round trip.

### M0B: biological and numerical contract

Write the units table and transfer table for producer capture, ingestion, assimilation, waste, maintenance, construction, reproduction, mortality, and decomposition. Define structural versus reserve nutrients/energy. Decide viable tissue thresholds, failure order, constitutive physiology, and action costs. Derive finite-rate updates and diffusion stability bounds for the actual chosen method.

Deliver: executable transaction accounting and reference scenarios with expected outcomes. Check individual transfers and whole-world ledgers. Begin with proposed absolute and relative tolerances of 1e-8 in normalized f64 ledger units, then justify any revision using accumulated numerical-error measurements. Physics-solver mechanical-energy residuals require separate measured tolerances and cannot excuse unbalanced biological transfers.

### M1A: minimal ecological cycle

Implement light capture, fixed producers, finite mineral uptake, growth, diagnostic feeding, reserves, maintenance, juvenile/mature state, funded birth, death, and aggregate decomposition. Route death and waste exactly once. Initialize populations from a declared resource inventory.

Deliver: a resource cycle with structured event logs and nutrient/energy audits. No-light controls must exhaust usable energy; disabling recycling should make mineral lock-up measurable under the declared setup. If a run crashes ecologically, expose the cause instead of silently reseeding it.

### M1B: observation and reproducible continuation

Add a basic Bevy observer with pause/advance, selected-organism information, and field overlays. Stream bounded snapshots; request detailed brain data only for selected subjects. Implement coordinated Rust/PyTorch checkpoints and an intervention timeline.

Deliver: the same state trajectory with the viewer absent or at different display rates under the controlled CPU configuration. Demonstrate a saved experiment continuing with the same events/state as an uninterrupted run. Save all RNG states rather than only the original seed.

### M2A: inherited variation and minimal neural behavior

Extend the diagnostic foundation first: instrument the observed consumer extinction; compare encounter/intake, energy expenditures, death causes, failed births, resource lock-up, and crowding across seeds. Add a small set of capability-differentiated founder presets with food compatibility, sensing, mobility, intake, and basic injury records, without structural mutation. Specify continuously renewed finite light flux and limited absorption. Add pan/zoom, resizable view, minimap, and useful energy/damage inspection. Reserve stable optional signal ports but keep communication disabled in the solitary baseline. Add richer predator/scavenger designs only when their physical interactions and substrates exist.

Replace the diagnostic controller with a small fixed-topology sparse leaky recurrent PyTorch controller, validating its feed-forward subcase first. Evolve initial weights and a few coupled scalar body/physiology traits. Initially keep expressed topology fixed for each controller design and lifetime plasticity disabled. Keep resource-funded birth and ecological selection unchanged.

Deliver: a visible ancestry tree, per-birth mutation log, exact inheritance checks, and stable neural/body port meaning. Run mutation-disabled, body-only, brain-only, and combined conditions. A mutation changing a cost-bearing trait must update both its capability and associated cost model.

Preserve all unmutated inherited neural parameters by stable gene identity. Never reinitialize the full network at birth or after a local topology edit. Add sparse CfC and then LTC as fixed-topology comparisons with the same observation/action interface, parameter-inheritance contract, and measured update budgets. Keep evolving topology/plasticity extensions separate from the unmodified candidate-model comparison.

### M2B: adaptation study

Choose a resource-distribution change with a measurable reproductive/survival consequence. Archive ancestral and descendant genomes. Evaluate them in common environments and, where relevant, mixed competitions. Use predeclared independent seeds and observation horizons.

Deliver: a report separating inherited effects, drift, ecological conditions, and uncertainty. Report extinct seeds and individuals still alive at the observation cutoff. Do not call a rising population or increased neuron count evidence of intelligence.

### M3: body graph and development

Introduce module addition/removal/duplication/attachment changes, with functional sensors/actuators and costed juvenile development. Maintain stable ports and valid connectivity. Add Rapier contacts/joints when the body model requires them. Connect actuator budgets to a declared mechanical work/efficiency approximation and track mechanical-energy residuals.

Implement the rooted module-tree decoder specified in PLAN.md first. Begin with compound rigid phenotypes built from simple shapes, explicit connector tissue where needed, and geometry-derived mass/inertia. Add scalar resize/reorientation before add/delete/duplicate/reattach operators; articulate joints only in a later, separately validated extension. Commit developed/destroyed sensor and actuator ports through the same neural topology/version barrier, with no full-controller reinitialization.

Deliver: functional anatomy variation, resource-funded construction, and an inspector that connects inherited modules to physical capabilities and neural ports. Verify mutation validity, broken/destroyed module behavior, genotype/phenotype separation, and concurrent feeding claims. Reject malformed representation edits; account for development that is valid but biologically unsuccessful.

Separate attack damage from feeding. Define reach/contact, strength/protection, paid attack limits, and local tissue destruction with resource-accounted debris. Damaged sensors/actuators/intake lose corresponding capability; repair spends resources/time and construction losses. Show actual injury, missing modules, and repair, and save their state. Validate concurrent attackers against one victim inventory before cooperative-hunting experiments.

### M4: evolving neural structure

Add neuron/connection insertion, deletion, duplication, and recurrent parameters. Define update order, time constants, activation bounds, and recurrent initial state. Pack heterogeneous controllers without changing the equations or hidden sensory access. Handle births/deaths and topology rebuilds at fixed barriers.

Deliver: zero-neuron, minimal, and larger-brain runs with accurate node/synapse counts and metabolic costs. Compare batched execution with a simple per-controller reference across supported shapes. Verify neuron/state identity after edits and that no controller gets another organism's hidden state.

### M5: evolving food web

Enable evolving producer traits, living recycler populations, substrate-specific digestion, and feeding on living tissue/detritus as anatomical capabilities become available. Remove or partition aggregate decomposition. Record resource flows by organism/lineage and time window.

Deliver: observed trophic roles and a food-web view derived from actual transfers. Report population persistence, extinctions, resource balance, and sensitivity across seeds. Do not promise stable diversity or force a fixed ratio of predators to prey.

Include mobile grazers, pursuit/ambush opportunities, scavenging, and compatible living recyclers through explicit funded capabilities. Connect optional costed local signal emitters/receivers: bounded bandwidth/values, range, attenuation, delay, lifetime, and reception rules. Each organism retains its own ANN and memory; private neural state/global team metadata is inaccessible. Reserve queued signals/decay state in coordinated saves. Chemical trails remain a later optional extension.

Add controlled small-hunter/large-prey scenarios. Capture outcomes follow actual contacts, paid attacks, injury accumulation, and prey defense, without pack multipliers or shared food creation. Record associations, messages, attacks, and feeding allocations. Test equal-number/biomass independent hunters against signaling-capable hunters with disabled/cost-matched ineffective-signal controls. Report per-capita return, injuries, survival, and reproduction as well as captures. Group labels are observational initially; persistent recognition requires a modeled cue/memory mechanism. Add optional group-follow, signal/damage overlays, and observed food-web panels without granting those overlays to ANNs.

### M6A: lifetime synaptic learning

Introduce the bounded eligibility-trace/plastic-offset candidate specified in PLAN.md and evolve a limited set of its parameters. Compare a network-controlled modulatory output with unmodulated and zero-plasticity baselines. Only declared local observations/internal physiology may drive modulation; no hidden intelligence or future-survival label. Keep a fixed-weight recurrent baseline. If adding autograd-based learning, explicitly define its objective, available experience, optimizer state, update frequency, and resource costs. Detach/reset state at declared boundaries; do not retain an unbounded computation graph across an entire lifespan by accident.

Deliver: matched plasticity-on/off trials, learning curves, and offspring reset checks. Separate performance from inherited wiring, recurrent memory, and learned changes. Global intelligence-test scores remain outside the ecosystem.

After genetic-only inheritance is validated, compare separately configured fractions of acquired-weight transfer on matching inherited connections. Recurrent state and eligibility still reset. This is a weight-inheritance experiment rather than an implicit transfer of lifetime-grown neurons, optimizer history, or whole-body acquired state.

### M6B: lifetime neural development

Add inherited growth/pruning rules. Propose edits, reserve resources in Rust, rebuild PyTorch structures preserving compatible state, then commit a new topology version. Development changes the expressed neural phenotype, while offspring inheritance still uses the birth genome.

Deliver: costed neural growth/pruning, successful abort/recovery behavior, and clearly separated birth-state and mature-state checkpoints. Compare with fixed-topology development controls before making claims about improved learning ability.

### M7: experimental throughput and long-run robustness

Profile each subsystem, memory use, neural topology distribution, birth/death churn, tensor packing, bridge overhead, and viewer overhead. Compare PyTorch CPU and supported GPU paths including completed device work. Run independent worlds in parallel with bounded worker/thread counts. Consider a Rust inference kernel or custom GPU kernel only if the measured bottleneck warrants it.

Deliver: reproducible throughput/memory reports and long runs that retain valid accounting/checkpoints. Predeclare the computational ceiling and whether reaching it ends a run or invalidates further complexity claims. Computational resource limits must not silently become hidden food or selective protection.

Profile targets of thousands of organisms per ecological role using mixed anatomy, network sizes, birth/death churn, signaling, and concurrent attacks. World extent is independent of window size. Near views show anatomy/injury, medium views show silhouettes/groups, and far views show density/resource/food-flow summaries; use viewport culling and bounded selected-detail requests. Off-screen biology continues unchanged. Verify camera/zoom/resolution/display-rate invariance and measure both simulation and renderer limits; no population-capacity claim follows from fitting more symbols on screen.

### M8: secondary subject evaluation

Archive candidates during earlier milestones. Select finalists using validation environments; evaluate copies on unseen tests of foraging, memory, risk, transfer, and learning. Keep embodied and body-matched controller comparisons separate. Freeze test definitions, normalization, and weights before final evaluation.

Deliver: reproducible subject packages for ecological, compact, and test-suite candidates, including negative results. A stronger cognitive subject is a possible finding, not an ecological success requirement.

Evaluate social candidates with matched group sizes/biomass and frozen partners/opponents where appropriate. Separate individual ability, partner effects, communication, and numerical advantage using the corresponding controls. Archive compatible bodies, signal interfaces, and partner/controller configurations so a group result can be reproduced without assuming a shared brain.

### M9: usable local product and release

Create a documented launcher/command that starts the Python experiment controller, imports the packaged Rust extension, and optionally opens the Bevy viewer. Provide a headless command using the same configuration and rules. Distribute an initial pinned Python environment/package, native extension wheel, and viewer executable for the chosen platform; select a more elaborate desktop installer only if needed later.

Include versioned presets for a small inspectable world, a reproducible evolution experiment, and a subject evaluation. Label diagnostic controllers/scaffolds and intervention/reseed modes. Presets document their assumptions and resource budgets; example scenes are not evidence that every random seed coexists or becomes more intelligent.

Finish the interface with world navigation, organism selection, body/neural inspection, resource overlays, lineage/event history, run configuration, pause/advance/speed controls, save/load, experiment progress, and result/subject export. Keep display speed separate from biological timestep. Handle invalid configurations, missing optional acceleration, checkpoint incompatibility, and failed runs with clear messages and preserved diagnostics.

Deliver: clean-environment installation/build instructions, CPU reference operation, documented optional GPU support, version compatibility information, example experiment recipes, the model/units guide, and reproducible export packages. Verify launch, headless execution, save/resume, inspection, and export workflows on the supported platform. Confirm that packaging does not change biological rules or introduce a second authoritative state.

Initial release acceptance requires demonstrated execution/inheritance/accounting correctness, repeatable example workflows, and documented experimental/performance limits. Cognitive improvement, stable diversity, and increased body complexity are reported research outcomes rather than installation/product-readiness gates. Sexual crossover, articulated locomotion, full 3D, disease, detailed chemistry, and distributed execution remain later scoped extensions unless explicitly selected for a future release.

## 5. Experiment protocol

Use a versioned record for each study:

```text
question and hypothesized mechanism
world/controller configuration and simulator revision
enabled/disabled mechanism and control condition
independent world seeds and evaluation seeds
observation horizon, sampling schedule, inclusion/stop rules
primary outcomes, effect criterion, and uncertainty method
accounting/numerical acceptance tolerances
CPU/GPU/backend, worker counts, and computational limits
checkpoint/archive locations and analysis version
all seed outcomes, interventions, extinction and failure records
```

The independent unit for a world-level intervention is normally the world/run, not every correlated offspring within it. Shared starting seeds can support paired comparisons, but different event histories still diverge. Report uncertainty at the appropriate level. Evaluate robustness to timestep, grid resolution, and plausible coefficient changes before extrapolating a result beyond its tested setup.

## 6. Decisions to settle before each expansion

Before M1: units, compartment definitions, producer conversion rules, initial inventory, rates, mortality, funded birth/death, diffusion method, numerical tolerances.

Before M2: sensor/action contract, genome schema, mutation distributions/rates, expected mutation load, initial controller, local selection semantics, ancestry/event records, control studies.

Before M3/M4: module vocabulary, connected-body rules, gene/port identity, body/brain cost coupling, recurrent equations, topology edit semantics, graph limits, mechanical-energy accounting, state migration.

Before M5/M6: substrate classes, decomposition partitioning, plasticity rules, allowed internal feedback, learning objective if any, inherited/acquired-state boundary, developmental commit/abort behavior.

Before M7/M8: required experimental throughput, hardware/backend choice, checkpoint compatibility policy, independent seed/horizon plan, candidate selection, frozen final tests, and export schema.

## 7. Immediate next work

The implementations through neural topology are available. Next expand the food web and social experiments: compatible carcass feeding, living recyclers with partitioned substrate, local signal propagation/costs, and equal-number/biomass group-hunting controls. Reserved signal ports currently stay disabled. Preserve current inheritance, geometry, resource, rollback, and world/neural continuation checks. The documented pilot results include mixed gains and extinctions; they are not evidence that intelligence or coexistence is increasing.

Continue toward ANN-controlled allocation/reproduction, lifetime neural development, long-run scaling, held-out evaluation, and supported release workflows. Refuges/obstacles and richer resource patches need explicit perception/contact/resource semantics. The deferred robotics mode will scavenge parts from other robots/wreckage and fund assembly, repair, and growth; it is outside the current animal-style implementation.

## Current capability gates

| Work | Implementation | Required evidence |
| --- | --- | --- |
| A: diagnose long-run collapse | Resource-aware intake; role-specific diagnostics; persistent manifests/autosaves; bounded growth, conservative mineral diffusion and declared grazing residual | Reproduce the original failure, retain failed interventions, test revised reference seeds beyond that horizon and check continued reproduction |
| B: full-window observation | Compact top-left HUD; B-toggle neural inspector; labelled hover inspection; responsive layout and input blocking | Actual window resize/toggle/selection, owner save/load, unchanged biological state under observation |
| C: neural agency | Planned; current circle outputs remain thrust, turn and feed | ANN-controlled growth, repair, reproduction and resource allocation, with funded biological gates and learning controls |
| D: functional food web | Partial legacy diet/anatomy mechanisms; absent from the circle reference | Diverse compatible diets, scavenging/recyclers and measured trophic flows |
| E: hunting and communication | Planned | Functional pursuit/ambush/defence, costed signals and matched cooperative-hunting studies |
| F: differentiation and development | Ancestry and birth mutations implemented; species grouping and lifetime graph development planned | Traceable traits/lineages, funded neural growth/pruning and controlled inheritance comparisons |
| G: scale, evaluation and release | Prototype build/save workflows exist | Measured resource limits, held-out capabilities and reproducible packaged release |

These gates supplement the original sixteen-stage history. A code path is not a validated ecological or cognitive outcome. Update this table, the existing walkthrough, and the verification report after each capability; do not declare all earlier stage promises complete because infrastructure exists.

## Hunting and habitat extensions

Keep the main bodies as circles until the richer mechanics can be observed and tested. Thread traps need funded deployable segments, attachment points, reach, tension, adhesion, breakage/escape, decay, and complete saved state. The ANN chooses deployment and retrieval; inherited genes change properties. Material remains owned tissue or environmental detritus throughout its lifecycle.

Camouflage needs inherited appearance/contrast traits and an actual local detection model that accounts for background, motion, light and occlusion. A recoloured sprite does not implement camouflage. Any active colour change has an explicit cost, and hidden targets cannot remain visible through perfect engine-supplied coordinates.

Add meaningful habitat patches, obstacles, shelter, light/shade and water/substrate conditions with corresponding contact, perception and resource rules. Terrain state persists independently of camera size. Validate background-dependent detection, trap escape and resource accounting before claiming new hunting strategies. Day/night cycles, transport beyond mineral diffusion, articulated appendages, detailed chemistry and 3D remain separately scoped work. Robots remain deferred.


## Expanded prototype delivery

Neural agency, food webs, habitat/hunting/social mechanisms and expressed neural development are connected to the existing resource/genetic/controller contracts. Throughput benchmarks cover a cohort above 1,000 organisms on CPU/CUDA and a separate 1,024-hidden-neuron controller cohort; those short benchmarks are not a claim of long-run capacity. Studies record sources/configuration, complete events, audits, bounded candidate retention and failure checkpoints. Nine matched small-hunter/large-prey trials and a 36-trial held-out subject battery were executed. Communication was available in the successful hunting scenario; these few seeded founder trials do not establish evolved social intelligence.

Long-run limitations remain explicit. Predatory/recycler/scavenger niches did not persist in the three expanded reference seeds. Small or wounded sterile consumers can survive without continuing a lineage; these are not counted as reproductive viability. The model still has starvation/consumption/damage mortality rather than senescence. The next research work is ecological calibration, reward/development tradeoffs, additional niche/learning controls and repeated held-out studies, without hidden reseeding or diversity subsidies. Articulated anatomy and the robot ecosystem remain outside the current prototype.

Development/test Cargo profiles omit debug symbols and incremental caches to reduce disk use. Runtime artifacts, the primary Python environment and saved biological state remain separate from regenerable Rust caches. The release workflow permits running the packaged simulation without retaining a Rust build tree.
