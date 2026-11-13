# Native resource accounting

`world.ResourceLedger` is a thin Python wrapper around the Rust ledger in `world_core`. Rust owns the rules, inventories, organism records, and committed receipts. The Python reference remains an independent implementation for comparison. The configured coefficients and hand-calculated fixture are unchanged.

Run the checked lifecycle from the repository root:

```powershell
.\.venv\Scripts\python.exe examples/native_lifecycle.py --output artifacts/native-lifecycle.json
.\.venv\Scripts\python.exe -m unittest discover -s tests -v
```

The example covers initialization, light capture, feeding, growth, maintenance, parent-funded birth, death from damage, and decomposition. All eight summaries must match [the expected fixture](../examples/lifecycle_expected.json). The labels producer, consumer, and child map to numeric IDs only in this example; they are not permanent organism classes.

## Python interface and initial inventory

```python
import json
from pathlib import Path
from world import ResourceLedger

rules = json.loads(Path("configs/world_rules_v0.json").read_text())
initial = {
    "schema_version": 1,
    "initial_nutrient": 13.0,
    "initial_energy": 11.0,
    "mineral_nutrient": 10.0,
    "light_available": 10.0,
    "organisms": [{
        "id": 1,
        "genome": {"photosynthetic": True, "initial_weights": []},
        "tissue": {"nutrient": 2.0, "energy": 4.0},
        "reserves": {"nutrient": 1.0, "energy": 7.0},
    }],
}
ledger = ResourceLedger(rules, initial)
ledger.apply("photosynthesize", organism_id=1, light_energy=1.0)
print(ledger.audit())
print(ledger.summary())
state = ledger.snapshot()
```

Initial nutrient and energy totals must equal the actual inventory. Available external light is a separate input budget: it enters the energy audit only when absorbed. Heat, mechanical energy, and cumulative absorbed light begin at zero. Optional `detritus` defaults to zero; if supplied, it contains both `nutrient` and `energy`.

Each organism requires `id`, `genome`, `tissue`, and `reserves`. IDs are unsigned 64-bit integers, including values above floating-point integer precision, and must be unique. Age and reproduction cooldown default to zero; alive defaults to true; parent ID and death cause default to null. A living organism needs viable tissue, the configured tissue energy ratio, and reserves within its tissue-dependent capacities. A dead record must have zero resources and a death cause of `starvation`, `consumption`, or `damage`.

The genome contains `photosynthetic`, `can_ingest`, and finite `initial_weights`; omitted flags default to false and omitted weights to an empty list. This immutable blueprint is copied at birth. These encoded weights are inert data here: no controller executes, learns, or mutates them. Recurrent controller memory from the diagnostic fixture is a separate interface.

`audit()` returns nutrient/energy totals and residuals. `summary(label="state")` returns aggregate inventories. `snapshot()` returns independent decoded rules, budgets, pools, organisms, and receipt history. Editing this Python result cannot change the native ledger. The snapshot is an inspection format; there is currently no resource-ledger restore API.

## Operations

Use `ledger.apply(name, **arguments)`. Durations default to 1 model-time unit when omitted and must be finite and positive. Amounts and energy expenditures must also be finite and positive. Nutrients, chemical energy, heat, and mechanical energy are separate compartments; see [the rule contract](../WORLD_RULES.md) for coefficients and arithmetic.

| Name | Required arguments besides optional duration | Effect |
| --- | --- | --- |
| `photosynthesize` | `organism_id`, `light_energy` | Spend available light; divide captured energy between reserves and heat |
| `absorb` | `organism_id`, `nutrient` | Transfer existing minerals to a capable organism's reserves |
| `feed` | `consumer_id`, `target_id`, `tissue_nutrient`, `contact` | Prepay digestion; transfer existing target tissue into assimilation and waste |
| `grow` | `organism_id`, `tissue_nutrient` | Spend nutrient and energy reserves on tissue and construction heat |
| `maintain` | `organism_id` | Pay maintenance and advance this organism's age; insufficient reserves cause death |
| `reproduce` | `parent_id`, `child_id`, `placement_available` | Fund offspring tissue, reserves, and construction entirely from the parent |
| `damage` | `organism_id`, `tissue_nutrient` | Transfer destroyed tissue to detritus; kill below viability |
| `decompose` | `fraction` | Transfer a fraction of existing detrital nutrient to minerals and energy to heat |
| `sense` | `organism_id` | Spend the configured sensing energy as heat |
| `actuate` | `organism_id`, `energy` | Divide expended reserve energy between tracked mechanical work and heat |
| `dissipate_motion` | `energy` | Transfer existing mechanical energy to heat |

`reproduce`, `damage`, `actuate`, and `dissipate_motion` take no duration. Decomposition fraction is in `(0, 1]`; nutrient and energy rate caps are checked independently, including energy-only waste. Uptake and assimilation overflow returns nutrient to minerals and energy to heat. Tissue loss recomputes reserve capacities. Death empties tissue and reserves into detritus exactly once; previously stored mechanical energy remains in the world.

Birth requires maturity, completed cooldown, available placement, a previously unused ID, and sufficient reserves. The child receives an independent clone of the inherited genome, age zero, its parent's ID, and the configured juvenile inventory. The parent receives a new reproduction cooldown. No free offspring resources or replacement births are supplied.

## Validation and commit

The binding receives versioned JSON. For example, the wrapper encodes a maintenance request as:

```json
{"schema_version": 1, "operation": {"kind": "maintain", "organism_id": 1, "duration": 1.0}}
```

Rules, initial inventories, organisms, and operations reject unknown fields or unsupported schemas. Numeric fields reject Booleans; IDs also reject negative or floating-point values. Non-finite values and arithmetic overflow are rejected. Configuration retains the reference's positive-coefficient validation and fraction bounds; disabling processes with zero-valued coefficients is not supported by this model.

Each operation audits the existing state, stages copies of the affected organisms and resource pools, executes transfers, and audits the proposed state. Only an accepted draft updates the ledger and appends a receipt. A receipt contains its sequence, operation arguments, before/after pools, before/after inventories for affected organisms, and conservation residuals. Newborns have no prior inventory. Invalid requests and failed proposed-state audits preserve inventories, ages, ancestry, cooldowns, and receipt history.

A valid maintenance request that cannot be funded commits starvation and the corresponding transfers. This is a biological outcome, not a rejected request. Python `ValueError` indicates malformed or disallowed input; `RuntimeError` indicates an accounting invariant failure. Both leave a previously valid ledger unchanged when raised by an operation.

## Current scope and cost

The standalone ledger has no grid, diffusion, spatial contact solver, replenishment schedule, or global ecological clock. Its caller supplies contact/placement decisions, durations, and operation order. The producer habitat below now supplies scheduling and local access for its supported operations. Direct ledger calls still apply caps per supplied window. Only maintenance advances organism age. Maintenance uses reserves without tissue catabolism.

Transactions clone only affected organism records, but full conservation audits scan all records before and after every operation. Touched genomes are cloned with those records, and inspection serializes the full receipt history. This correctness-oriented interface is not a demonstrated throughput path for hundreds of thousands of organisms. Future scheduling, batching, and profiling must preserve these semantics.

Tests compare Rust with both independent expected values and the Python reference, including 200 seeded mixed requests, strict input boundaries, funded inheritance, mortality, and rollback. They establish accounting behavior under the declared model; they do not establish ecological realism, adaptation, or neural learning. See [verification evidence](../artifacts/setup/accounting-verification.json).

## Headless producer habitat

```powershell
.\.venv\Scripts\python.exe examples/habitat_demo.py --ticks 3600 --output artifacts/producer-habitat.json
```

`world.Habitat(rules, config)` owns a bounded grid, organism positions, a resource ledger, and a scheduler. `advance(ticks=1)` advances native ticks; `summary()`, `audit()`, and `snapshot()` return independent inspection data. Tick counts must be actual Python integers in the unsigned 64-bit range. `advance(0)` leaves state unchanged. Each native tick commits atomically; a multi-tick Python call retains successful earlier ticks if a later one fails. `save(path)` and `Habitat.load(path)` provide complete continuation; inspection snapshots remain a separate format.

### Initialization and local access

The [preset](../configs/producer_habitat.json) funds 64 identical producers with 2 N tissue, 0.5 N reserves, and 5 E reserves each. Tissue contains 4 E, so founders require 160 N and 576 E altogether. The 4096 cells initially contain 4096 N, giving the declared 4256 N total. Inventories are checked before the habitat is accepted; seeding does not add resources outside those declared budgets.

`producers.positions`, when supplied, must contain one `{x, y}` point per founder. Otherwise seed/counter pairs use the versioned SplitMix64 mixing calculation recorded as `splitmix64-counter-v1`. IDs start at 1 and remain integers. Identical inputs reproduce the initial layout and execution on the verified backend; this is not a cross-platform floating-point guarantee.

Coordinates are finite and within the closed outer rectangle, including its walls. Internal cells use the interval beginning at their lower edge; a point on the outer maximum edge belongs to the final row/column. Producers are stationary points with no collision area. They can access only their occupied cell. `cell_overrides` can change a specific row/column's initial minerals or light power; duplicate, empty, unknown, or out-of-bounds overrides fail.

### Time, inputs, and process order

The shared `physics_dt` and `ecology_interval_ticks` determine update timing. In the default preset, six ticks of 1/60 T form one 0.1 T ecological window. Intermediate ticks advance the clock without repeating biological work. Ages advance during maintenance once per completed ecological window; unfinished windows do not change them.

Each window performs this order:

1. Expire the preceding window's uncaptured light into `escaped_light`.
2. Offer each cell `light_energy_per_time * duration`, adding it to cumulative `incident_light`.
3. Recycle existing detritus when enabled, then give each living producer resident an equal initial share of that cell's light and minerals. Each actual request is also limited by physiology rates, current availability, and reserve headroom. Unused shares stay in the cell until a later window; they are not reassigned immediately.
4. Capture light, absorb minerals, pay maintenance, and then grow if still alive and funded. Starvation transfers all remaining resources into that same cell's detritus. Mechanical and heat accounting remain global.
5. Run paid local consumer sensing, movement, feeding, maintenance, and growth; then attempt mature, funded, locally placed births when enabled.
6. Audit the proposed whole habitat, then commit pools, organisms, grid, light counters, latest window receipt, and clock. Failed processing or audits preserve the preceding tick, including age and cell resources.

The preset's per-cell light power is 8 E/T multiplied by `1 - gradient + gradient * (column + 0.5) / columns`, with gradient 0.75. This supplies an explicit left-to-right field. Gradient must be in `[0, 1]`; zero power supports dark scenarios. Per-cell powers and mineral amounts are configuration inputs, so grid-resolution comparisons require adjusting them to preserve the intended total inputs.

### Ownership and audits

Cells own minerals, detritus, and available light. The ledger's corresponding pool values are cached aggregate views of those cells, not additional compartments. Physiology temporarily receives one cell's pools while a proposed window runs; totals are rebuilt before the global audit. Whole-world nutrient/chemical-energy checks count cell inventories once. A separate light identity checks:

```text
cumulative incident light = cumulative absorbed light + escaped light + currently available light
```

Only absorbed light enters the chemical-energy budget. Uncaptured light never becomes organism energy. The latest window receipt records duration, processed producers, deaths, light captured, nutrient absorbed, and tissue grown. Per-operation standalone-ledger receipts are not duplicated for every scheduled producer action; the habitat retains the latest window receipt rather than an unbounded history.

The implementation copies the grid and changed organism records for a proposed window and scans grid/population inventories for audits. This is a correctness-oriented baseline, not a demonstrated large-population throughput path. The producer preset remains a stationary control with reproduction/recycling disabled. The earlier ecosystem preset enables the scripted diagnostic processes below. Neural control, mutation and modular anatomy run in the separate evolving preset described later in this document. Diffusion, Rapier mechanics and automatic reseeding remain absent. The diagnostic PyTorch bridge remains separate.


## Diagnostic ecological cycle

The ecosystem preset uses a 24 x 24 world/grid so the initial 64 producers and 16 consumers interact locally. Its declared 4296 N and 3136 E exactly fund cell minerals and founder tissue/reserves. This denser preset is distinct from the unchanged producer-only control.

Consumers pay sensing cost before querying producers within `perception_radius`; they choose nearest distance with ID as a deterministic tie-breaker. Without a sensed target they follow a deterministic heading turn. Travel is bounded by speed, duration, energy, walls, and target stand-off. A cell index narrows local queries; swept-path clearance prevents passing through another living point within `minimum_spacing`. There is no impulse response or evolving physical body. Initial founder points may overlap; spacing constrains accepted movement and newborn placement, not founder sampling.

Actual travel incurs `movement_energy_per_distance`; the existing movement efficiency divides expenditure into work and heat, then the kinematic scaffold dissipates the work immediately. No kinetic energy is retained. Consumers ingest only sensed, living producers within `feeding_radius`, limited by tissue, the per-window ingestion rate, and prepaid digestion. Waste stays in the consumer's destination cell; a killed target's carcass and a surviving target's released mineral overflow stay in the target cell, including across cell boundaries. No global food search is available to this policy.

Producers and consumers pay maintenance once per completed window and grow only when funded. When reproduction is enabled, reaching the maturity tissue threshold switches the diagnostic allocation policy from growth to saving for offspring. A birth additionally requires minimum age, completed cooldown, full tissue/reserve/construction investment, capacity below `max_population`, and one clear in-bounds position from eight deterministic local attempts. Failed placement/capacity spends nothing. IDs are monotonic; the parent genome and encoded weights are cloned, while child age/cooldown begin at zero. Newborns wait until the next window to act. There is no mutation or learned allocation decision yet.

Decomposition runs before acquisition, so new waste/deaths become eligible in the next window. One world-level nutrient and energy rate budget limits a shared detritus fraction across all cells; each cell returns minerals locally and dissipates substrate energy as heat. This is a controlled aggregate recycler, not a living organism. Nutrient is finite; only declared light enters externally. No births, food, minerals, or organisms are inserted to repair an ecologically unsuccessful run.

The latest window receipt and cumulative `ecology` report expose processed consumers, feeding events, ingested nutrient, distance moved, growth, births, deaths, and recycled nutrient/energy. Ancestry remains in organism records. The habitat retains these counters and its latest receipt rather than an unbounded per-operation event log; detailed lineage/experiment exports grow with later work.

## Complete habitat continuation

`Habitat.save(path)` writes strict versioned JSON with kind `world.habitat`, schema 1, and the exact engine version. It includes rules/configuration, all cells and pools, original budgets, every living/dead organism and encoded genome, positions, consumer headings, unused offspring ID, cumulative metrics, incident/escaped light, scheduler including unfinished windows, and the latest receipt. Deterministic heading/placement draws are derived from seed, ID, and tick; there is no omitted mutable RNG stream in this scaffold.

Saving uses a uniquely created sibling temporary file, flushes contents, and renames it over the destination after successful serialization. Failed replacement leaves the old checkpoint intact and removes the temporary file. This protects ordinary failed writes; it is not a cross-platform power-loss durability guarantee. Checkpoints use data-only JSON, with no pickle or executable deserialization.

Loading builds a candidate before replacing any viewer state. Unknown fields, unsupported schemas/versions, duplicate identities, invalid geometry/clocks/ancestry/headings, inconsistent metrics, starting-budget changes, and resource discrepancies are rejected. Finite f64 values round-trip exactly through the enabled JSON parser feature. Tests compare complete future states through partial windows on this build. The headless core and viewer use the same save/load API; render timing does not enter simulation state.

The habitat does not yet execute PyTorch controllers, so their independent fixture checkpoint is not part of this format. Controller integration must introduce coordinated world/controller state at a shared tick before claiming complete neural-ecosystem continuation.


## Evolving bodies and inherited control

`world.ecosystem.EcosystemRuntime` owns the evolving run. Its native habitat requires `evolution` configuration and funded founder genomes; the scripted presets retain their old behavior. The new preset's passive producers have no neural graph. Grazers and pursuit/ambush-capable founder designs have separate sparse recurrent graphs, directional sensors, mouths, and propulsion. Their initial weights implement declared reflex seeds; strategy names do not command the controller or prove that hunting was learned.

The current update uses one neural decision per ecology window. Rust supplies local observations and port permissions at the starting tick. PyTorch proposes actions and recurrent values. A cloned native candidate validates exact tick, IDs, graph versions, widths, finite bounds, duplicate/missing ports, and inactive outputs, then advances the complete window. Python reconciles live cohorts and stable neuron identities before publishing either candidate. Failures preserve the previous world and recurrent memory. Native `advance()` rejects evolving worlds; callers must use the coordinator.

Inputs include bounded internal reserves, tissue/development, injury, maturity, local photosynthetic cues where supported, and directional local food/neighbor cues. Sensor location, orientation, range, expressed tissue, and investment control visibility; no terrain occlusion is implemented yet. Resource acquisition uses the occupied core cell while photosynthetic tissue limits its rate. Feeding/attacks use actual module contact. Optional signal channels remain zero. Remote/global role labels, viewer annotations, and other brains' private state are excluded from neural inputs.

### Tissue, shape, and function

A genome contains a connected tree with a vital core and tangent circle modules. Core, photosynthetic tissue, sensors, propulsion, mouths, armor, and storage have stable IDs. Radius, attachment angle, orientation, investment, and density determine the phenotype. Invalid cycles, roots, overlaps, dimensions, identities, or ports are rejected. The declared adult tissue target is `sum(pi * radius^2 * density * 12)`; juvenile geometry scales to its funded developed tissue. Maturity also requires the body tissue target in addition to the base rule thresholds.

`Organism.tissue` is authoritative. Module tissue values are a checked allocation of that same inventory, never additional biomass. Healthy developed allocation follows blueprint area proportions; damage decreases current tissue while retaining its developed capacity. Physical radii shrink with integrity. Missing tissue, inactive ports, and impaired ancestor modules reduce function. Destroying the core or crossing the minimum viable tissue threshold kills the organism and transfers remaining resources once.

Growth spends reserves and construction heat, adding current and developed tissue proportionally. Repair spends reserves/time to replace missing tissue up to developed capacity. Ordinary growth does not erase the absolute injury deficit. Geometry must remain inside walls and clear of other living bodies before expansion commits. Storage adds nutrient capacity equal to 1.5 times functional invested storage tissue and energy capacity equal to 5 times it; injury can release nutrient overflow locally and energy overflow as heat. These are declared normalized prototype coefficients, not measured biological constants.

### Motion, damage, and resource transfers

Propulsion tissue and investment produce force at expressed attachment positions. Explicit turning effort and off-center forces produce torque. Translation is overdamped by tissue mass/drag; rotation is overdamped by geometry-derived rotational resistance/drag. Swept module paths conservatively include rotational arcs, preventing tunneling. Blocked travel can attempt a clear rotation. No velocity, stored kinetic energy, articulated joints, or fluid solver is present.

Movement pays effort plus actual translational/rotational work multiplied by `work_energy_scale` and divided by movement efficiency. In this overdamped model work dissipates within the update; its energy is recorded as heat rather than retained as a second pool. Body upkeep, sensing, expressed neurons, connections, attack effort, digestion, maintenance, construction, and repair have explicit costs. Expenditure records distinguish activity costs from offspring/growth investment; their sum is not a second conservation ledger.

Attack and ingestion are separate. An attack pays effort, checks mouth reach/contact, applies the declared armor reduction, and removes tissue from the contacted module. Debris stays in the victim's cell. A bite is limited by the contacted module, digestion funding, compatible diet, and one per-organism ingestion budget shared across mouths. Assimilation/waste obey the existing coefficients. Cross-cell carcasses and victim overflow remain in the victim cell; digestive waste remains in the consumer cell. Dead matter is recycled by the existing aggregate process; carcass feeding and living recyclers remain later work.

Action resolution uses a declared rotating deterministic ID priority between windows. Observations are collected before those interactions; each tissue claim sees the remaining inventory. No shared target can be consumed twice. The initial implementation does not grant group bonuses or communication.

### Birth and mutation

The parent funds base offspring tissue plus `offspring_module_nutrient` for each additional module, `offspring_neuron_nutrient` for each neuron, and `offspring_connection_nutrient` for each connection. Tissue embodiment, construction losses, and juvenile reserves use the same transfer rules as the standalone ledger. A proposal additionally requires maturity, cooldown, reserves, population capacity, and a clear local compound-body placement. Failed funding/placement creates no child and spends no juvenile inventory; diagnostics distinguish those causes.

Rust copies the inherited genome, draws birth-specific mutation proposals from seed/parent/child/tick counters, validates edits, and commits one funded child. Parameter/body scalar edit counts are bounded Poisson draws; structural proposals have separate configured probabilities. Invalid representation proposals are recorded rather than retried until useful. Valid but poorly functioning descendants receive no protection. Accepted-proposal counts do not mean beneficial mutations; before/after values and candidate genome hashes support interpretation.

Body operators resize/change investment/orientation, add/delete/duplicate subtrees, and reattach modules. Neural operators perturb weights/bias/leak, insert/delete/duplicate hidden neurons, and add/delete connections. Required body input/output ports are preserved or synchronized by module identity. New ports initially have zero bias and no invented learned behavior; subsequent valid connections can recruit them. Existing unaffected encoded parameters retain their IDs and values. Only founder/new genes receive declared initialization. Parents do not mutate during these birth operations.

The configurable and hard graph/module ceilings are computational constraints. A ceiling is not free food or protection. Birth-block reasons, mutation attempts, deaths, damage, feeding, and repair are recorded in bounded native history. The runner exports events with session/branch markers and checks for history gaps. Independent clone, brain-only, body-only, and combined scalar-mutation pilots keep structural mutation/plasticity disabled for their comparisons; separate tests exercise structural operators and heterogeneous execution.

### Observer and continuation

The viewer's evolving mode reads `world.observer` publications from the Python owner. Camera controls, minimap, selected-body details, and selected neural activity do not advance native biology. Pause/advance/speed/save/load requests use session-tagged commands and acknowledgements at decision boundaries. Failed load keeps the current coordinated state. Files are written atomically, with bounded retries for Windows sharing conflicts; transient display read conflicts retain the last valid frame.

The `world.ecosystem` save envelope contains the complete `world.habitat` checkpoint and sparse neural memory keyed by organism/neuron IDs at the same tick. Birth-derived random draws need no omitted mutable RNG stream. Newborn memory is zero; retained individuals keep compatible state; deleted/dead identities disappear from the registry. CPU continuation is checked across births, mutations, and topology changes. The older standalone/fixture checkpoint formats remain separate. See `artifacts/setup/evolution-verification.json` for current evidence.

## Active circle experiment

Circle bodies have one funded core and inherited physiological traits. The physical collider and visible body use the same developed radius. Missing tissue changes integrity/colour but retains its already-developed envelope until repair/death; it does not create new nutrient. This prevents a feeding neighbour from entering a wound and blocking regrowth. Time-of-impact movement stops at walls/other circles, permits turning at contact, and scales effort/travel to the available budget. Feeding is allowed at the same circle surface plus the declared 0.08 reach. Cross-cell waste and carcasses retain their owners.

The preset lowers neural/body maintenance, charges plasticity per connection, and requires a 24-T maintenance reserve before growth/birth. The horizon includes neural, sensing and tissue upkeep. Births remain resource-funded; initial maturity is 8 T and cooldown 6 T. The tested thousand-hidden-neuron grazer has positive possible intake headroom. These are versioned modelling coefficients, not biological measurements.

Online feedback measures the organism's actual reserve-energy change over the decision, adding back development expenditures and excluding subsequent offspring investment. It therefore includes net acquisition and actual metabolic/foraging costs without directly rewarding a hidden target position. Separate records track displacement and contact, which are permitted observations. Neural state, parameters and traces are proposed first and commit only after native validation and reconciliation.

Circle births copy matching current learned parental parameters, then mutate and fund the resulting genome. Brain creation is an explicit funded mutation, not a free runtime controller. Circle scalar mutations affect radius/density/speed/perception/learning rate; compound-module edits remain disabled in circle mode. The legacy anatomy/fixed-controller contracts above remain available for controlled comparisons.

Circle birth opportunities rotate deterministically among mature candidates, so early producer IDs do not monopolize the population ceiling. Blocking/forward-wall feedback is local and the initial avoidance weights are declared mutable reflexes. Event export reads only events after its cursor and rejects history gaps, avoiding repeated full-history serialization in the interactive loop.

## Grazing, development and mineral transport

Circle ingestion is bounded by the amount that can satisfy either nutrient headroom or net energy headroom after digestion. A full consumer cannot keep stripping tissue into overflow. Growth uses already-developed tissue to determine the adult target; wounds use the repair path. Growth-driven centre displacement stays within the radius increase and pays work, and all final/intermediate circles remain within the conservative envelope.

The circle reference declares a 15% inaccessible fraction of producer developed tissue for grazing. This material remains in the producer, can still be lost through other permitted damage/starvation, and is not regenerated for free. Food sensing and contact selection use harvestable material. This accessibility assumption is configurable and defaults to zero for old worlds.

Mineral diffusion transfers equal-and-opposite amounts across adjacent grid faces with closed boundaries. The explicit finite-volume update uses stable substeps; it creates no minerals and moves neither tissue nor detritus. Its coefficient has units distance squared per model time. The default is zero for old worlds and 0.25 in the new reference.


## Filaments, feeding and neural development

A deployed filament debits parental nutrient reserves and chemical energy. Its embodied material enters a separate owner-keyed cell inventory, included in the global detrital ledger exactly once; construction heat enters the existing heat sink. Free detritus and bound filament inventories are disjoint. Decomposition/scavenging access only free detritus. Retraction, breakage, expiry or owner death transfers the existing inventory into local free detritus. No inverse subtraction of rounded aggregate reservations is required.

Detrital feeding transfers a bounded fraction of existing local nutrient/chemical energy, spends digestion energy and applies inherited substrate investments. Recycler waste mineralizes; scavenger/predator waste nutrient remains detrital. Unassimilated energy dissipates. Aggregate decomposition is a separately configured scaffold and shares the finite organic pool with living recyclers.

Lifetime neurons occupy an allocation within existing tissue, capped by tissue and configured graph ceilings. Adding neural structure dissipates a declared construction-energy charge; ongoing neurons/edges/plasticity incur maintenance. Pruning reduces ongoing cost but does not create tissue, nutrients or an energy refund. Body development and repair debit both reserves. The deterministic expenditure order is neural/sensing authorization, photosynthetic capture/uptake, interactions/signals/filaments, locomotion, attacks/feeding, repair/body growth, upkeep/maintenance, births, then neural topology development. Outputs are competing requests, not grants of resources.
