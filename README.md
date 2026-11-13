# World

A survival-driven 2D artificial-life prototype. Rust owns resources, physical interactions and birth mutations; PyTorch executes independent small neural controllers; Bevy observes the Python-owned run.

The default ecosystem uses one physical circle per organism, with anchored producers, grazers, pursuit/ambush founders, filament hunters, scavengers and living recyclers. Green indicates producers, blue grazers, orange predators, purple scavengers and olive recyclers. White marks show heading; red rims show injury. These are inspectable starting designs, not evidence that these niches persist indefinitely or become more intelligent.

## Run

From this directory in PowerShell:

```powershell
.\scripts\run_evolution.ps1
.\scripts\run_evolution.ps1 -Paused
.\scripts\run_evolution.ps1 -Preset circle
.\scripts\run_evolution.ps1 -Preset hunting
.\scripts\run_evolution.ps1 -RunDirectory artifacts/another-run
.\scripts\run_evolution.ps1 -Load artifacts/evolution-checkpoint.json
```

The launcher audits the existing dependency lock, builds cached native inputs and starts the simulation owner and observer. Native build output is captured as bytes, decoded explicitly and emitted as plain ASCII; PowerShell never has to decode raw decorative UTF-8 prefixes. No global encoding, execution-policy or driver settings change. Close the viewer or press Ctrl+C in the runner to save and finish. New launches create unique folders under `artifacts/runs`; explicit relative run folders resolve against the project directory. Separate run folders avoid output collisions; a live owner holds an OS file lock on its publication path.

CPU is the reference device. `-Device cuda` explicitly selects CUDA and fails if it is unavailable. Neural tensors move to that device; native world/genome data remains in RAM. There is no automatic RAM/VRAM offloading. Interactive/headless commands use one PyTorch CPU thread by default for these small networks; `--threads` makes the choice explicit for larger experiments.

| Control | Action |
| --- | --- |
| Space / N | Pause/resume / advance one whole biological decision |
| + / - or numpad + / - | Change requested speed |
| F5 / F9 | Save / load complete world and learned controller state |
| Left click / F / B | Select / follow / toggle the top-right neural inspector |
| H / V / E | Help / measured food web and recent selected events / export selected neural subject |
| Wheel / Home | Zoom / fit the world |
| WASD, arrows, right drag | Pan |
| G / L / F11 | Resource overlay / field type / fullscreen |

The sidebar separates target speed from measured model time per wall-clock second. 16x is a request, not a guaranteed throughput. Biological decisions are never skipped to imitate that rate. The camera uses the whole window, with compact statistics at the top left. B opens the selected brain at the top right. Panels block world clicks and wheel input. Resizing changes the camera, not biology; hidden organisms continue to simulate.

## Ecosystem mechanics

The default `food_web` preset funds 96 producers and 50 consumers in a 48 x 27 world with a 1,024-organism computational ceiling. Each revised neural consumer starts with 46 local inputs, 16 hidden neurons and 16 outputs. Movement, feeding, attack, growth, repair, reproduction attempts, neural growth/pruning, filament deployment/retraction, signals, bursts, escape effort, concealment posture and local food preference pass through its own recurrent network. Outputs request actions; available energy/material, maturity, cooldowns, contacts and geometry decide what can execute. Competing expenditures use a documented deterministic order. There is no global fitness tournament or intelligence reward.

Heritable investments include size/density, speed, perception, learning rate, light capture, substrate digestion, camouflage/pigment, armor, filaments, communication, burst propulsion and neural development. Birth-time diet mutations can change ingestion specialization. Predator-to-predator feeding is possible; trophic levels come from actual intake history rather than assigned secondary/tertiary labels. Scavengers and living recyclers consume local organic matter; recyclers return unassimilated mineral nutrients, while scavenger nutrient waste remains detrital. Slow aggregate decomposition is an explicit background scaffold that can be disabled.

Rocks block movement and sight; vegetation/water change drag; cover, background matching and movement affect camouflage detection; shade reduces capture. Filaments have paid material/energy, fixed ground anchors, finite reach, strength and lifetime. Contact can restrain eligible prey; paid escape weakens threads. Breakage, retraction, expiration and owner death release existing material into local detritus. Filament material is stored separately from edible detritus and counted once. Threads are simplified line constraints, not articulated soft-body physics.

Signals are local, scalar, attenuated and short lived. Receivers observe emitted values and bearings, never another network's private state. Multiple hunters can accumulate real injuries on one prey; there is no pack damage bonus or shared food creation. Pursuit/ambush labels describe founder biases and opportunities. Coordinated group hunting must be demonstrated against matched controls; signal availability alone is not proof.

Lifetime neural growth adds funded hidden neurons/connections at declared boundaries; pruning removes hidden nodes and their edges. Surviving learned weights, traces and memory map by organism/gene ID. `expressed_brain` is separate from immutable birth DNA. The default inherits matching learned parameters onto the birth graph; `inherit_structure` explicitly enables inheritance of the expressed graph. Newborn personal memory/traces reset. Descriptive genetic clusters and founder lineages appear in study results; clusters do not enforce species quotas or mating barriers.

The `circle` preset remains the earlier two-role control described below. Old published checkpoints retain their stored coefficients and controller interface. The new defaults do not retrofit old runs into a different ecosystem.

## Revised ecology, defense and social inputs

New food-web worlds enable explicit `embodied_ecology` settings. Light capture scales with current photosynthetic area and a shared cell budget. Founder ages vary within a declared interval; offspring tissue and reserves scale for small-bodied lineages. Starting detritus replaces a declared portion of mineral inventory and its chemical energy is included in initial funding. There is no later rescue injection. These coefficients are experimental, not a claim of stable coexistence.

Attacks are independent of ingestion eligibility. A sufficiently capable herbivore can damage a carnivore while its diet prevents meat consumption. Attacks require directional physical contact and paid effort; functional mass, offensive investment and armor affect damage. Multiple defenders contribute individual hits without a group bonus. Dead tissue becomes local organic material.

Additional observations describe pigment-similar neighbours, local alignment/crowding, unfamiliar organisms, recent damage, attacker bearing and left/right clearance. Similarity is a visible numerical phenotype, not access to another organism's diet or genome. Social investment scales social observation strength. The ANN continues to choose movement: no flocking force or forced defense controls it. Local grouping metrics are descriptive and do not prove cooperation. Food observations offer a nearby and a more profitable local candidate; output 124 chooses which is primary. Very small remaining live-food bites are excluded from the search cue so depleted plants need not monopolize attention.

The revised learner adds a small private temporal-value estimator, a longer trace horizon, injury feedback and a declared successful-birth feedback. These are learning assumptions, not proof of improved intelligence. Activity history and value state are checkpointed; newborn learning state resets. Lifetime neural growth funds material as existing body tissue plus construction heat, adds a zero-influence outgoing pathway, and retains existing parameters. Pruning uses activity and incident weight strength when available. Genetic brain insertion is conservative for revised lineages; other mutations can still be disruptive. Body mutation scale is separately configurable.

`V` includes recent requests, executed movement/feeding/damage, energy before/after, selected targets, attackers and failed actions. Target/attacker lines are observer overlays. `L` cycles mineral, light and detritus fields. The walkthrough remains unchanged.

## Liquid controller experiments

```powershell
.\scripts\run_evolution.ps1 -Controller ltc
.\.venv\Scripts\python.exe examples/liquid_learning.py --decisions 200 --output artifacts/liquid-study.json
.\.venv\Scripts\python.exe examples/run_study.py --scenario herding --controls full no_social no_signals --seeds 73 89 107 --output artifacts/herd-study
```

The selectable `plastic_sparse_ltc_v2` backend is a custom sparse conductance-based LTC variant. It uses bounded implicit integration, the heritable graph, independent parameters, approximate local perturbation eligibility, lifetime development and coordinated continuation. It is not the exact `ncps` LTC parameterization and is not CfC. The default remains the sparse leaky RNN. The earlier squared-conductance `plastic_sparse_ltc_v1` checkpoint update remains supported for exact continuation; new LTC runs use absolute conductance so zero-influence new pathways can start learning.

`liquid_learning.py` tests the actual installed `ncps` LTC and CfC models with online learning, frozen controls, inherited parameters and restored state in a physical contact-feeding task. Movement is held at zero as an explicit task restriction and neural parameter counts affect the energy charge. Those official-library candidates remain fixed-wiring experiments rather than fully topology-mutable ecological backends. No architecture has been declared the winner. Studies also support `defense`, `no_social` and `genetic_only`; social and learning benefits require matched comparisons.

## Circle control behaviour

`configs/circle_rules.json` and `configs/circle_habitat.json` fund 80 producers and 20 consumers in a 40 x 22.5 world (`circle-world-v0.3`). Consumers start with 15 inputs, 8 hidden neurons and 3 outputs. Their initial food-steering, motor and contact/forward-wall responses are explicit starter reflexes; learning is subsequently driven by their own food/energy outcomes. These are starting designs, not permanent species labels or proof of intelligence.

Movement advances to valid circle contact instead of cancelling the entire displacement. Circle feeding uses that same boundary. The developed collision envelope is retained during injury so a consumer cannot move into missing tissue and prevent its food from repairing; its size was funded during development. Injury changes tissue/function and colour. Repair spends real material and energy. Growth and births preserve a configured maintenance reserve, including brain, sensing and body upkeep. Injury is repaired rather than counted as new adult development. Circle growth can displace its own centre by at most the funded radius increase, with paid work and conservative contacts; truly crowded growth still fails.

The controller is `plastic_sparse_rnn_v1`: sparse leaky recurrence plus bounded reward-modulated eligibility traces. Recent pre-synaptic activity and independent neural perturbations set traces; later net food/energy feedback changes weights and biases. Essential metabolic/foraging costs enter feedback; growth and offspring investment are separately accounted. Plasticity has an explicit energy charge. There is no hidden food-direction reward or intelligence bonus.

At a birth, matching current parental learned weights/biases are copied before genetic mutation. This is the declared acquired-parameter inheritance policy for the circle experiment. The final outcome of the birth's decision is learned after that inheritance boundary. Newborn recurrent memory, traces and reward baseline start empty. Surviving parameters/traces remain intact when the population is repacked. Birth mutations can alter circle size/density/speed/perception/learning rate, add/delete neural structure, or create a first port-aligned brain in a brainless lineage. Larger networks must still pay their costs; capacity is not forced to increase each generation.

Each organism has separate parameters and memory. Living controllers share flat ragged execution tensors, not learned weights. Exploration is counter-derived from seed, tick, organism ID and neuron ID, so another organism's birth/death does not shift its random stream. Tensor storage is bounded by the active neural cohort; historical native genomes remain retained experiment data and contribute to long-run RAM/checkpoint growth.

The reference world declares 1 mineral N per cell, mineral diffusion at 0.25 distance-squared per model time, a 512-organism computational ceiling, and a 15% grazing residual. The residual is existing producer tissue that grazing cannot remove; it is a declared accessibility assumption, not replacement food or immunity to starvation. It does not guarantee coexistence. Consumers stop ingesting when neither reserve can use more material, and food sensing excludes inaccessible tissue. Older checkpoints retain their stored coefficients, including zero diffusion/residual when absent.

## Diagnostics

The runner writes append-only `evolution-run-telemetry.jsonl`, a uniquely identified manifest per invocation/restore segment, three rotating `evolution-run-autosave-*.json` checkpoints, and additional decline/extinction checkpoints. Samples contain populations, role-specific birth failures/deaths, last birth ticks, growth blocks, producer-cell minerals, global resource compartments, expenditure, and conservation audits. Manifests record rules, configuration, code hashes and execution device. They identify the run that actually produced an observation.

The expanded 60,000-tick trials retained accounting but lost their predatory/scavenger/recycler guilds. One seed ended with an immature consumer whose lineage had stopped reproducing; that is not reproductive viability. No senescence model is currently applied, so small or wounded sterile survivors can remain alive. Ecological calibration remains research work.

The original long-run collapse was reproduced and retained. Earlier short pilots were insufficient, and intermediate fixes also failed some seeds. See the current report for all declared outcomes and the tested horizon; persistence of one sterile organism is not a reproduction success.

## Experiments and continuation

```powershell
.\.venv\Scripts\python.exe examples/evolution_demo.py --decisions 200
.\.venv\Scripts\python.exe examples/evolution_demo.py --resume artifacts/evolution-checkpoint.json --decisions 100
.\.venv\Scripts\python.exe examples/learning_demo.py --seeds 73 89 107 --decisions 500
```

Circle checkpoints use `world.ecosystem` schema 2 and include the native world, live learned weights/biases, memory, eligibility traces, reward baselines and counters. They are data-only JSON with strict identity/shape/value checks. Failed native actions or learning reconciliation commit neither world nor controller state. CPU continuation is exact within the tested code/backend; GPU/cross-platform bitwise identity is not promised.

The default observer receives lightweight `world.display` packets containing render/inspection data, not full worlds/genomes. Neural parameters are shown only for the selected organism, with bounded graph detail. Display packets cannot be resumed as simulations. Save/load commands go to the owner, which writes or restores a complete checkpoint. Graph metadata crosses the controller boundary only when IDs/versions change; ordinary decisions exchange current observations.

## Legacy controls and remaining scope

```powershell
.\scripts\run_viewer.ps1
.\.venv\Scripts\python.exe examples/evolution_demo.py --rules configs/evolution_rules.json --config configs/evolution_habitat.json --decisions 100
.\.venv\Scripts\python.exe examples/controller_comparison.py --device cpu
```

The earlier scripted ecology and fixed-weight compound-body experiments remain available. Legacy schema-1 checkpoints keep their fixed-controller semantics. The official `ncps` CfC/LTC implementations remain fixed-topology comparison candidates; a custom sparse LTC variant is separately selectable as described above. Their optional `ncps==1.0.1` dependency is already in the reviewed environment.

Detailed articulated bodies, sexual crossover, full 3D, disease, detailed chemistry, automatic RAM/VRAM offloading and distributed execution remain outside this prototype. Robotics remains a separate later project: robots would scavenge parts/wreckage and fund assembly and growth. Population capacity and cognitive improvement remain measured research outcomes, not promises.

## Studies, subjects and distribution

```powershell
.\.venv\Scripts\python.exe examples/run_study.py --output artifacts/studies/example --seeds 73 89 107 --decisions 10000 --workers 2
.\.venv\Scripts\python.exe examples/run_study.py --output artifacts/studies/social --scenario hunting --controls full no_signals ineffective_signals --decisions 500
.\.venv\Scripts\python.exe benchmarks/population.py --scale 8 --decisions 100 --output artifacts/population.json
.\.venv\Scripts\python.exe examples/evaluate_subject.py PATH_TO_SUBJECT.json --output artifacts/evaluation.json
.\scripts\package_release.ps1 -Output artifacts/releases/world-local
```

Independent worlds use bounded worker/thread counts and separate directories. Disabled-feature controls are enforced by experiment rules, so descendant mutations cannot restore an ablated capability. The topology control disables both birth-time structural edits and lifetime growth/pruning. A study records its request hash, telemetry, events, checkpoints, audits, descriptive genetic clusters and explicit failures. Candidate exports include survivors, reproductive performers and compact controllers. Archives retain a declared bounded window of death subjects; final learned parameters are captured before dead controllers leave the active tensors. Full checkpoints retain all native ancestry, so historical RAM/checkpoint growth still needs monitoring.

Subjects contain the body/interface, birth DNA, expressed learned graph, personal state, origin rules/configuration and checksum, using JSON rather than executable model pickles. `E` exports the selected live subject. The held-out suite declares seeds and tasks before evaluation; embodied ecological trials compare learned, birth-weight and learning-disabled copies with the same body. A separate signed-cue delay probe measures frozen recurrent response. This is a limited test battery, not a general intelligence score or proof of social intelligence.

The release assembler includes the native wheel, viewer, presets, examples, pinned dependency lock and checksum manifest. On Windows x64 with CPython 3.13, run the bundled `scripts/install_release.ps1`, then `scripts/run_release.ps1` (or `-Headless`). Python dependencies are installed separately from their hash-locked sources; an offline install needs a prepared wheelhouse. Rust/Visual Studio is required to build source, not to run the packaged binaries. CPU is the reference backend; CUDA remains explicit and must pass local checks.

## Verification and reading

```powershell
.\.venv\Scripts\python.exe scripts/plain_command.py -- .venv/Scripts/python.exe -m unittest discover -s tests -v
.\scripts\build_native.ps1 -Wheel
```

The walkthrough is deliberately frozen at the user's request and does not describe these newer features. See [expanded verification](artifacts/setup/ecosystem-expanded-verification.json), [current A/B verification](artifacts/setup/ecosystem-ab-verification.json), the [walkthrough](artifacts/code-walkthrough.md), [body/resource contracts](docs/resource_accounting.md), [formats](docs/data_formats.md), [plan](PLAN.md), [roadmap](IMPLEMENTATION_ROADMAP.md) and [dependencies](DEPENDENCIES.md). Verification includes real feeding/learning controls, inherited parameters, invalid-state rollback, native observer commands and coordinated continuation. Short seeded pilots are evidence for their declared scenarios, not guarantees of indefinite coexistence or general intelligence.


## Disk usage

Rust build caches and debug symbols can be much larger than the simulator. Development/test profiles now disable debug symbols and incremental caches, while retaining the existing optimization levels. This reduces disk use at the cost of some incremental rebuild speed. The `target` directory is regenerable; the source, configuration, `.venv`, checkpoints and archived subjects are separate. A packaged release can run without a Rust build cache. The main CUDA-enabled PyTorch environment remains several gigabytes because it includes GPU libraries. A temporary wheel-test environment should be removed after verification rather than retained as a second runtime.
