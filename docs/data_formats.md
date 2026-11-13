# Diagnostic bridge contracts

The fixture exercises batched Rust/PyTorch exchange. Its organisms have fixed signals,
controller versions, last actions, and accumulated action values. It has no spatial
world, metabolism, evolving genome, or learned weights.

The separate `world.ResourceLedger` implements resource and lifecycle transactions.
Its JSON contracts are documented in [resource accounting](resource_accounting.md).
The fixture's observations/actions are not connected to those biological operations yet.

## Configuration

`configs/bridge_fixture.json` contains schema version 1 and an ordered `organisms`
list. Each entry has an unsigned 64-bit `id`, unsigned 64-bit `topology_version`,
and three finite `signals` in `[0, 1]`. IDs must be unique. An empty cohort is valid.
Unknown fields and unsupported schemas are rejected. Tick and action state begin at zero.

Signals are diagnostic values named left, right, and level. They are not measured
food, energy, or privileged information from a biological world.

## Batches

Both batch types contain these fields:

| Field | Representation | Constraint |
| --- | --- | --- |
| `schema_version` | Python integer | Exactly 1; Booleans are rejected |
| `tick` | Python integer | Unsigned 64-bit range; Booleans are rejected |
| `ids` | NumPy `uint64`, shape `(N,)` | Unique identities, never floating-point IDs |
| `topology_versions` | NumPy `uint64`, shape `(N,)` | Version corresponding to each ID |
| `values` | NumPy `float32`, shape `(N, K)` | Finite, C-contiguous values |

For observations, `K=3` and values are in `[0, 1]`. For actions, `K=2`: turn is in
`[-1, 1]` and thrust is in `[0, 1]`. All arrays must have the declared native dtype,
rank, row count, and C-contiguous layout. There is no implicit dtype conversion.

An action batch must contain exactly one row for every current fixture organism.
Rows may be reordered, with IDs and versions carried alongside them. Partial,
duplicate, unknown, stale, non-finite, out-of-range, or malformed batches fail.
Tick overflow also fails. Empty cohorts still advance their logical clock.

Rust observation arrays use independent NumPy allocations. Python batch objects
copy their inputs and mark those copies read-only. The native input boundary copies
array data into Rust values; no external array is retained as mutable world state.
Callers must not mutate input buffers concurrently with a native call.

## Controller and commit

The diagnostic controller has two recurrent memory values per ID, initially zero:

```text
turn   = clamp(right - left + 0.25 * previous_turn, -1, 1)
thrust = clamp(level + 0.25 * previous_thrust, 0, 1)
next_memory = [turn, thrust]
```

PyTorch computes a complete batch using CPU or explicitly requested CUDA execution.
The controller has no optimizer, reward, or plasticity. Registry lookups preserve
memory identity when observation rows are reordered.

Rust validates the full batch and calculates all resulting totals before committing.
An accepted batch records each action, adds it to that organism's totals, and
advances the tick once. Rejection changes neither the tick nor any organism state.
The Python registry commits its prepared memory only after Rust accepts the actions.
No failed batch is replaced with a helpful default action.

The runtime is a single-coordinator interface: do not advance a world independently
of its associated registry. It is not a concurrent multi-client transaction service.

## Fixture continuation

`BridgeRuntime.save(path)` writes a single JSON checkpoint through a temporary file
and atomic replacement. The checkpoint contains schema version 1, the Rust fixture
snapshot, and the diagnostic controller snapshot. Loading validates IDs, versions,
tick alignment, memory bounds, and agreement between memory and last actions.

The fixture uses no randomness, so no RNG state is consumed or required. CPU
save/resume is checked against uninterrupted execution. CPU/CUDA results are checked
within declared float tolerances; cross-hardware bitwise identity is not promised.
This format does not yet cover a biological world's complete checkpoint requirements.

## Commands

Run from the repository using the installed project environment:

```powershell
.\.venv\Scripts\python.exe examples/bridge_demo.py --device cpu --ticks 5 --checkpoint artifacts/bridge/checkpoint.json
.\.venv\Scripts\python.exe examples/bridge_demo.py --device cpu --ticks 3 --resume artifacts/bridge/checkpoint.json
.\.venv\Scripts\python.exe examples/bridge_demo.py --device cuda --ticks 5
.\.venv\Scripts\python.exe benchmarks/bridge.py --device cpu --population 128 --ticks 200
```

Benchmark timing includes packing, copies, controller execution, action validation,
and completed device-to-host transfer. It measures this fixture, not ecosystem or
large heterogeneous-ANN capacity.


## Biological sparse-controller exchange

The evolving mode uses a separate `AgentFrame` with schema 1, decision tick, window duration, and exact organism rows. Each row includes a graph version, encoded neural genes, ordered `{port, value}` inputs, `{port, channel, active}` output permissions, and execution authorization. Values are finite normalized local observations. The current correctness path uses one cohort JSON exchange at the Python/native boundary, not per-neuron callbacks. The diagnostic fixture's NumPy contract above remains unchanged.

Body port IDs are `module_id * 256 + channel`. Core channels 0-6 expose declared internal/local physiology; sensor channels 10-16 expose local directional cues. Outputs 100/101 allocate growth/repair, 110/111 request thrust/turn, and 120/121 request feeding/attack. Reserved 18/102 signal channels stay disabled. Required ports have one matching input/output neuron. Hidden neurons have no port. Organism IDs scope neural gene identities; IDs are unsigned integers, including values beyond floating-point integer precision.

An `ActionBatch` has schema 1, the matching starting tick, and rows `{id, version, values:[{port,value}]}`. All live neural organisms must be represented, including those unable to afford an update; those rows have zero physical outputs and unchanged memory. Missing, duplicate, unknown, stale, inactive, non-finite, or out-of-bounds entries reject the complete candidate. Positive effort channels are in [0,1]; turn is in [-1,1]. Physical affordability/contact still constrain accepted requests.

The baseline graph is `sparse_leaky_coo_v1`. Input nodes receive current permitted observations. Every other node uses the prior recurrent state in a synchronous weighted sum, followed by tanh and its inherited per-decision leak coefficient. PyTorch flattens ragged organism graphs with indexed edge sums, without dense largest-topology padding. Float64 is the reference precision. Stable `(organism_id, neuron_id)` keys preserve compatible memory through registry migration. Exact CPU and tolerance-based CUDA checks compare against an independent scalar implementation.

## Coordinated biological checkpoint and observer

`world.ecosystem` schema 1 has exactly `schema_version`, `kind`, `engine_version`, `world`, and `neural`. `world` is the full native checkpoint. `neural` has schema/kind/tick, ordered identity keys, and finite bounded recurrent values. Genotypes and encoded weights remain in Rust-owned organism records. Loading validates both components before accepting either. No optimizer or lifetime plasticity state exists in this version.

`world.observer` schema 1 wraps a coordinated state with session, publication sequence, paused/speed/status, and last acknowledged command. Optional `actual_speed` reports achieved model time per wall-clock second; `speed` remains the requested multiplier. Older publications without this field remain readable. Observer commands have exactly schema version, session, unique ID, kind, and path. Local selection/camera preferences remain observer-only. The Python owner handles commands; the viewer never runs a second active neural world. Ordinary publication is capped at 10 Hz, with immediate command/single-decision updates. Publications may be skipped for display; biological decisions cannot be skipped. The current full-world JSON publication is a reference path; region-limited/binary streaming and large-population optimization remain scaling work.

Native event sequences are monotonic within the world. Exported JSONL records include session, branch segment, and event; restore markers distinguish rewound branches. Native history is bounded, while the runner checks for dropped event ranges. Coordinated saves and observer files use data-only JSON and atomic replacement with bounded Windows sharing retries.

Fixed wired CfC/LTC candidate snapshots use a separate diagnostic format containing model specification, named/shaped parameters, and recurrent memory. Candidate outputs select the wiring's motor IDs, apply the same action bounds/masks, and undergo inheritance/continuation checks. These experiments do not establish topology mutation or ecological fitness equivalence for liquid models.

## Plastic circle checkpoints and display packets

`world.ecosystem` schema 2 identifies online-learning worlds. The native habitat still has schema 1 with optional `body.circle` traits and evolution settings. `neural.kind` is `plastic_sparse_rnn_v1`; schema 2 neural records include ordered node/edge identities, current weights/biases, memory, node/edge eligibility traces, per-organism reward baselines/change/update counters, the seed, and `noise_kind=splitmix_box_muller_v1`. The loader checks exact fields, identity sets, dimensions, finite/bounded values and immutable input biases. Schema-1 fixed-controller worlds remain readable with their earlier semantics.

Exploration is derived from seed, tick, organism ID and neuron ID, using SplitMix-style counters and Box-Muller Gaussian conversion. There is no omitted mutable RNG stream or dependence on another organism's batch position. Learning updates weights and biases, while leak parameters remain inherited birth genes. Born organisms receive encoded inherited parameters and fresh recurrent/tracing state.

An action row may include `learned={weights:[[gene,value],...],biases:[[node,value],...]}` only when the native circle frame requests inheritance synchronization. Exact identities, bounds, duplicates and immutable input biases are checked. A newly mature parent waits for this synchronization rather than silently copying stale birth parameters. The inheritance boundary precedes outcome learning for that decision. Ordinary controller exchange can omit a brain whose organism ID/version matches the coordinator's cached immutable graph; new versions must provide a full graph.

`world.display` schema 1 uses the same session/sequence/control envelope as the legacy observer, but its state is only a typed render frame: clock/dimensions, current fields, live circle summaries (plus an inspected retired record when requested), and a bounded selected neural graph. It contains no world checkpoint or full population genomes. The observer validates it without restoring a Habitat; it cannot be used as a continuation file. `inspect` uses the command path field as a decimal ID, not a filesystem path, and has no biological effect. F5/F9 and bounded capture saves are acknowledged owner commands that write full coordinated checkpoints.

The owner locks its publication path for the lifetime of the runner. An overlapping owner is rejected; OS locks release on process exit. Camera/selection/publication cadence remain outside biological evolution. Full ancestral native genomes are retained as experiment history, so long-run RAM/checkpoint cost remains a measured scaling limit rather than an automatic disk-offload feature.

## Persistent diagnostic capture and full-window HUD

`Habitat.telemetry_json()` exports compact resource/population information without genomes or learning tensors. Diagnostic maps partition birth rejections and deaths by producer/consumer role, record last birth ticks and growth blocks, and survive complete checkpoints. Missing maps in older checkpoints default empty; missing birth history is reported as unknown rather than reconstructed from incomplete event history.

The default runner samples telemetry every 120 ticks and keeps three complete autosaves at 6,000-tick intervals, plus cohort-decline/extinction snapshots. JSONL rows carry a unique invocation ID and restore segment. Matching manifests contain actual rules/configuration, source hashes and execution settings. Repeated invocations can append without conflating their identities. Autosaves are complete resumable worlds/controllers, not display packets.

`world.display` includes compact health telemetry in its summary and optional channel IDs for selected neural nodes. The viewer uses a full-window camera; overlays are independent of biological dimensions. B toggles the selected inspector, H help, and overlays consume world pointer interactions. Large selected graphs remain bounded and show the number of displayed versus actual nodes/edges.


## Ecological extensions

The optional `body.circle.adaptations` record preserves the legacy circle port interface when absent. With `agency=true`, inputs add local light/minerals, cover, detritus, entanglement, local signal value/bearing and age. Outputs add development, repair, reproduction, neural growth/pruning, deployment/retraction, emission, burst, attack, escape and concealment. Every port remains identified as `module_id * 256 + channel`.

`organism.genome` remains immutable birth DNA. Optional `organism.expressed_brain` records lifetime topology, with its own topology version and stable surviving gene IDs. Live learned parameters and recurrent/plastic state remain in the coordinated neural checkpoint. `inherit_structure` is an explicit experimental inheritance switch.

Native checkpoints add `interactions` (filaments, signals, food flows and measured trophic/intake histories). Each grid cell can contain a `filaments` map from owner ID to bound material, disjoint from its edible `detritus`. The interaction geometry/material registry and cell inventories must agree exactly. Published A/B checkpoints without these optional fields still load. Intermediate development-only filament snapshots using the former reservation representation are not compatible with this final inventory representation.

Lightweight display frames add bounded ecosystem overlays and organism profiles. These are observations, not resumable models. Inspection/export commands go through the owner. Studies write telemetry and complete event streams outside the bounded native event buffer.

`world.subject` schema 1 stores origin metadata, birth organism, expressed learned graph, controller kind, state and SHA-256 of canonical content. The checksum detects changed content; it is not a publisher signature. `world.evaluation` records the frozen suite hash and subject hash. Evaluation never writes scores back into an ecological world.

## Revised ecological interfaces

The new circle phenotype opts into `social_senses` and `foraging_choices`. These preserve old port sets when absent. Social/local-feedback inputs use channels 30 through 47; alternative food inputs use 48 through 51, and output 124 selects between nearby and material-rich local food cues. Revised consumers have 46 inputs and 16 outputs. Food profitability here is harvestable material per distance, not a learned energy forecast.

Evolution configuration explicitly selects embodied ecology, founder age spread, social observations, learned-parameter inheritance, trace duration, exploration, temporal feedback and injury/reproduction feedback. Old checkpoints retain default legacy coefficients. Cell overrides may include funded `detritus` nutrient/energy. Small-lineage offspring reserves and tissue are calculated from the body target and remain parent-funded. Initial age offsets are part of the audited clock bounds.

Plastic checkpoints retain activity history when enabled and a private nine-parameter temporal-value estimator per live organism when selected. Newborn personal state resets. `plastic_sparse_ltc_v2` selects the custom conductance-based LTC update and round-trips through the same coordinated world/neural checkpoint; it is not an alias for the fixed-wiring official `ncps` model. Malformed activity/value arrays are rejected.

Selected display profiles include requested actions, executed quantities, energy before/after, targets, recent attackers and rejection reasons. Executed movement is distance, turning is absolute angle, feeding and damage are nutrient amounts, and reproduction is a successful-birth count; these are not interchangeable with normalized requests. Group metrics describe visible pigment-similar proximity clusters and do not establish learned cooperation.
