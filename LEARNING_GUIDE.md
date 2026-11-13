# World: learning guide and readiness checks

Personal baseline: comfortable with Python and PyTorch; new to Rust. This is a project-specific preparation path, not a requirement to complete several broad courses. A runnable resource-ledger reference for Exercise A and encoded-weight inheritance checks now exist in [reference/](reference/); the remaining exercises guide future work. Implementation of an example does not establish the learner's personal readiness, and no checklist has been marked complete on the user's behalf.

Read alongside [PLAN.md](PLAN.md), [CORE_PLAYERS.md](CORE_PLAYERS.md), and [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md).

## 1. What to learn before starting the core

Prioritize enough practical Rust to own simple state, enough model reasoning to balance a birth/death transaction, and enough interoperability to pass one batch safely. Your existing Python/PyTorch knowledge lets you learn these through small experiments immediately. Rendering, crossover, sophisticated plasticity, and GPU kernels come later.

| Priority | Topic | Required practical depth | Demonstration of readiness |
| --- | --- | --- | --- |
| First | Rust ownership and borrowing | Values, moves, references, mutable borrowing, Vec/slices, explicit Clone | Update and transfer reserves without accidental copying or simultaneous conflicting mutation |
| First | Rust data modeling | Structs, enums, Option, Result, match, modules, simple traits/generics | Represent organism state and return a meaningful failed-transfer result |
| First | Cargo and checks | Build/run/test, release builds, fmt, clippy, dependency/lock files | Build a small library, run its tests, and understand its compiler diagnostics |
| First | Evolutionary model | Genome/phenotype, inheritance, mutation, ecological selection, lifetime learning | Trace exactly what a child inherits and what resets |
| First | Resource/units model | Material compartments, energy input/dissipation, finite rates, construction and maintenance | Balance one ingestion, growth, reproduction, death, and decomposition example |
| Before the bridge | PyO3/maturin and arrays | Native extension build, shapes/dtypes, ownership/lifetime, copies/views | Call a Rust batch operation from Python and prove that IDs and buffers remain valid |
| Before active evolution | Numerical simulation | Fixed ticks, rate x timestep, finite values, tolerances, update order | Explain how changing timestep can change or destabilize an update |
| Before adaptation claims | Experiments/statistics | Independent runs, controls, drift, confounding, uncertainty, censoring | Design a mutation-on/off comparison without selecting only successful seeds |

Do not treat unfamiliar vocabulary as a reason to postpone all work. Complete a small exercise, inspect the result, then return to the relevant documentation.

## 2. Rust path for a Python/PyTorch developer

Use [The Rust Programming Language](https://doc.rust-lang.org/book/) as the main reference. Focus on basic syntax/control flow, ownership, structs, enums/pattern matching, modules, collections, errors, traits/generics/lifetimes, and tests. These topics are the proposed curriculum; you do not need every advanced feature before making the first library.

Learn to answer these questions in code:

- Who owns this organism collection, genome, buffer, and checkpoint data?
- Does an assignment move a value, copy a simple value, or intentionally clone owned data?
- When can a slice be borrowed for reading, and when is exclusive mutable access required?
- How can stable IDs refer to organisms without storing fragile pointers into a Vec that may grow?
- How do Option and Result express missing organisms, invalid actions, and failed transfers?
- Which errors indicate a model-contract violation, and which are normal biological outcomes?
- What does a release build measure compared with a debug build?

Start with ordinary structs, enums, Vec, slices, and ID-based lookup. Learn lifetime annotations as needed to understand references; initially prefer owned batch snapshots to complicated shared borrows. Avoid choosing unsafe Rust, custom allocators, or advanced pointer structures as prerequisites for the simulation.

Practice commands: cargo check, cargo test, cargo fmt, cargo clippy, and cargo run --release. Use Git to review small changes and retain exact dependency locks for an experiment. A benchmark should record its build profile and configuration.

On Windows, follow the [official Rust installation instructions](https://doc.rust-lang.org/book/ch01-01-installation.html), including the native compiler/linker prerequisites for the chosen target. Keep Python and native builds on a compatible architecture. Setting up the toolchain is a preparation task; this guide does not claim it is already installed.

## 3. Rust/Python boundary

Read the [PyO3 guide](https://pyo3.rs/) for exposing Rust functions/classes to Python and the [maturin guide](https://www.maturin.rs/) for building a mixed package. Start with one extension function and a clean local environment rather than integrating the whole ecosystem at once.

Understand:

- Extension modules, Cargo versus pyproject configuration, and rebuilding native changes.
- Array shape, dtype, contiguous layout, strides, and explicit IDs in batches.
- Ownership of returned arrays and the difference between a snapshot copy and a borrowed view.
- Why a view must not outlive its storage, and why resizing or simultaneous mutation is dangerous.
- Where native code can work without holding Python's interpreter lock, and why it must not access Python objects in that region.
- Why batching reduces boundary calls but does not eliminate all copying or device synchronization.
- How Rust errors become clear Python exceptions without partially committed world state.

Review [NumPy's quickstart](https://numpy.org/doc/stable/user/quickstart.html) for shape, dtype, indexing, and copies/views as needed. CPU tensors constructed with [torch.from_numpy](https://docs.pytorch.org/docs/2.15/generated/torch.from_numpy.html) can share NumPy storage; this is not permission to let Rust mutate or free that storage while a tensor uses it.

You are ready for the real bridge when a multi-organism batch can be reordered safely by ID, invalid shapes/versions are rejected, and buffer ownership is explicit. A scalar addition extension is a setup check, not enough to establish the simulation interface.

## 4. Evolution and genetic algorithms

Learn or review these distinctions:

| Distinction | Meaning in this project |
| --- | --- |
| Genome vs phenotype | Inherited construction instructions versus the actual developed body/brain |
| Neural state vs learning | Changing activations/memory versus changing weights or expressed structure |
| Learning vs evolution | Changes during one lifetime versus inherited differences across descendants |
| Survival vs evolutionary persistence | Staying alive versus leaving descendants under the environment's constraints |
| Mutation vs crossover | Random inherited edits versus recombining compatible parental genes |
| Selection vs drift | Systematic reproductive differences versus chance changes in a finite population |
| Analysis cluster vs species | A grouping used to inspect data versus a biological/reproductive claim |

Review [OpenStax: Understanding Evolution](https://openstax.org/books/biology-2e/pages/18-1-understanding-evolution) for the biological foundation, then read the representation/mutation ideas in the [original NEAT paper](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf). Focus on historical gene identity, changing topology, and viable incremental variation. NEAT's full fitness sharing and population selection are not our ecological selection mechanism.

Understand representation validity, biological viability, mutation distributions, expected mutation load, structural mutation frequency, gene alignment, and connected body graphs. Begin with mutation-only asexual reproduction; crossover is not needed for the first working evolutionary experiment.

Know what the birth transaction does: copy inherited genome, mutate, validate representation, fund development, commit parent/child resources, record ancestry, and construct a controller. Learned weights and recurrent memory reset according to the declared inheritance rule. Avoid giving every organism an extra hand-written fitness score because that is what a textbook GA example does.

## 5. Ecology, physiology, and model assumptions

Learn resource limitation, producers/consumers/decomposers, trophic transfers, population growth under finite resources, and how resource access and reproduction interact. Start with [OpenStax: Energy Flow through Ecosystems](https://openstax.org/books/biology-2e/pages/46-2-energy-flow-through-ecosystems).

For our abstract model, be able to identify the source and destination of every nutrient and energy transfer. Explain why light does not replace material needed for tissue, why eating must remove substrate, why offspring require parental investment, and why decomposition partitions resources rather than creating them. The model uses simplified currencies and declared coefficients; it does not reproduce full chemistry.

Understand constitutive physiology versus neural behavior. A zero-neuron organism can use modeled passive uptake and threshold reproduction without receiving a hidden foraging controller. Its capabilities still require anatomy and resource costs.

Before adding a new trait, specify its benefit and coupled costs. Avoid independently mutable genes that increase speed, sensing, assimilation, or neural updates with no construction, maintenance, capacity, or operating tradeoff. Costs are model assumptions to test, not values we may label natural merely because the result looks plausible.

## 6. Mathematics and numerical simulation

Given your background, review gaps rather than restarting all mathematics:

- Linear algebra and graphs: vectors, matrix dimensions, sparse edge lists, adjacency, recurrent state, connected components.
- Probability: seeded randomness, Bernoulli mutation events, perturbation distributions, expected mutation counts, variance, finite-population chance.
- Calculus/dynamics: rates, integration over a timestep, feedback, stability, saturation, and conservation.
- Statistics: independent units, effect sizes, uncertainty intervals, paired comparisons, multiple-condition exploration, and held-out evaluation.
- Numerical computing: floating-point error, tolerances, NaN/infinity, nonnegative pools, reduction order, and why a seed alone is not a cross-device reproducibility guarantee.

You should be able to write a fixed-timestep update and identify when a large dt could consume more than an available pool. Learn the stability condition for the actual diffusion/integration method when implementing it; a general familiarity with differential equations does not establish that an arbitrary update is stable.

Mechanical work matters once physics becomes more detailed: actuator expenditure, stored kinetic/potential energy, and dissipated heat must not be double-counted. Track numerical solver residuals separately from deliberate biological inefficiency.

Advanced PDE theory, full fluid mechanics, and detailed biochemistry are not prerequisites for the initial resource cycle.

## 7. PyTorch topics specific to this project

Your existing knowledge is the starting point. Review only unfamiliar items using [PyTorch's official learning material](https://docs.pytorch.org/tutorials/beginner/basics/intro.html) and the targeted documentation below:

- Recurrent hidden state belongs to an organism ID and must not leak between organisms when batches change.
- Heterogeneous graph packing, masks, and padding affect compute but must not grant extra biological capability or cost padded neurons.
- Local plasticity updates can run without autograd; gradient-based experiments require an explicitly defined loss/objective.
- The proposed controller is a sparse leaky recurrent rate network. Later plasticity separates inherited birth weights from acquired offsets and uses local eligibility traces plus bounded network-controlled modulation. Review these mechanisms when reaching M6; useful plasticity is an experimental outcome, not an automatic consequence of the rule.
- LTC/CfC liquid-network candidates change input-dependent temporal dynamics; that is distinct from learning new weights. All candidates inherit encoded parameters. Review inherited weights versus transient state and the separately configured acquired-weight inheritance experiment in PLAN.md.
- Stateful learning and truncated histories require deliberate detach/reset boundaries; avoid retaining a lifetime-long graph accidentally.
- Replacing/resizing parameters after structural edits requires explicit weight/state mapping and compatible optimizer-state handling.
- CPU/GPU comparison must include observation/action transfer, topology churn, packing, and completed device work.
- Saved controller state includes learned parameters and recurrent state; optimizer/history state is included where the learning method needs it.

Useful references: [autograd](https://docs.pytorch.org/docs/stable/autograd.html), [optimizers](https://docs.pytorch.org/docs/stable/optim.html), [CUDA semantics](https://docs.pytorch.org/docs/main/notes/cuda.html), and [reproducibility](https://docs.pytorch.org/docs/2.15/notes/randomness.html). Read these around a concrete experiment rather than treating every backend feature as a prerequisite.

## 8. Learn when the relevant milestone arrives

| Topic | When to learn it | Enough for the first use |
| --- | --- | --- |
| Bevy ECS and egui | M1 observer | Components, systems, schedules, camera/input, snapshot rendering, inspection |
| Rapier | M3 or when contacts/joints are necessary | Bodies, colliders, contacts, joints, timestep, relevant serialization and determinism limits |
| Advanced neuroevolution | M4 | Structural edits, historical IDs, recurrent semantics, graph validity |
| Ecological networks | M5 | Resource-flow records, trophic summaries, persistence/extinction analysis |
| Plasticity and learning-rule evolution | M6 | A bounded local rule, permitted internal feedback, controls, state reset |
| Sexual reproduction/crossover | After asexual stability | Gene alignment, mating compatibility, parental investment, conflict handling |
| GPU optimization/custom kernels | M7 after profiling | Memory layout, batching, synchronization, correctness, end-to-end timing |
| Full 3D | When spatial scope changes | 3D geometry, sensing, body dynamics, resource representation, transfer experiments |

Use the [Bevy introduction](https://bevy.org/learn/quick-start/introduction/) and [Rapier Rust guide](https://rapier.rs/docs/user_guides/rust/getting_started/) at those milestones. The chosen engine versions and plugins need compatible pins; you do not need to learn advanced shaders to inspect a resource cycle.

## 9. Preparation exercises in recommended order

### Exercise A: a resource ledger using familiar Python

Start with [WORLD_RULES.md](WORLD_RULES.md) and run [reference/worked_example.py](reference/worked_example.py). Compare each operation with [the independent hand-calculated fixture](examples/lifecycle_expected.json), then make a prediction before changing a parameter. Keep the default fixture unchanged when exploring a variant so you can see which original expectation stops applying.

Represent two organisms and a detritus/mineral pool. Perform feeding, reserve use, tissue growth, one funded birth, death, and decomposition. Write down expected totals before execution. Keep nutrient material and usable energy separate; account dissipation explicitly.

Ready when every transfer balances within a declared tolerance and attempting to spend the same food or birth investment twice fails. This exercise verifies model reasoning without adding Rust difficulty.

### Exercise B: a small Rust state library

Implement the same reserve/tissue structures, transfer function, organism IDs, and lifecycle enum in safe Rust. Return Result for invalid requests. Use Vec-based storage and explicit IDs. Add meaningful tests for valid transfer, insufficient reserves, one-time death, and parent/child inventory.

Ready when you can explain ownership/borrowing and failure behavior, not just make the compiler accept copied data. Keep the exercise independent of Bevy, Rapier, and Python bindings.

### Exercise C: a batched native/PyTorch round trip

Expose a small Rust world through PyO3/maturin. Publish observations for several organism IDs, compute actions in PyTorch, submit one action batch, and advance fixed ticks. Reorder the batch intentionally, remove one ID, and introduce a stale topology version to check rejection paths.

Ready when shape/dtype/ID checks work, storage remains valid, and no action can accidentally target a different organism. Measure the batch boundary rather than promising zero overhead.

### Exercise D: one tiny recurrent brain

Build a simple reference recurrent update and a batched PyTorch equivalent. Test zero-controller physiology separately, one active controller, and multiple differently sized controllers. Save and restore a hidden state; introduce a new child with a fresh state.

Ready when batched and reference results agree within the declared tolerance and no state crosses organism boundaries. This establishes execution correctness, not intelligent behavior.

### Exercise E: inheritance and mutation

Create a genome with one scalar body trait and initial neural weights. Produce clone and mutated offspring from seeded streams. Verify immutable parent genomes, stable gene IDs, reset acquired state, and the observed distribution of edits over many samples. Track expected versus realized mutations as genome length changes.

Ready when mutation-disabled births are inherited clones and mutation effects are visible without a hidden intelligence reward. Add topology edits only after scalar inheritance is correct.

### Exercise F: controlled experiment and continuation

Write a protocol comparing two configurations over independent world seeds. Include failed/extinct outcomes, an observation cutoff, and a relevant control. Save a world plus neural state mid-run and compare resumed continuation with uninterrupted execution in the controlled CPU setup.

Ready when you can reproduce the run and explain what result would contradict your hypothesis. A visually interesting single run does not satisfy this exercise.

## 10. Readiness checklist before going ahead

Before the first core implementation:

- [ ] I can explain who owns world state, inherited genomes, and expressed neural state.
- [ ] I can implement/read simple Rust structs, enums, Vec/slices, references, and Result-returning functions.
- [ ] I can distinguish moves, copies, clones, and borrowed array views.
- [ ] I can trace nutrient and energy budgets through a funded birth and death.
- [ ] I can distinguish survival, reproduction, inherited evolution, neural memory, and learning.
- [ ] I know the initial model is a simplified 2D ecology with declared assumptions.

Before enabling evolution:

- [ ] The scalar inheritance/mutation path is understood and tested, including mutation-disabled controls.
- [ ] I can explain how genome size changes mutation load and why malformed genomes differ from nonviable offspring.
- [ ] Observation/action ports, recurrent update order, and IDs have explicit contracts.
- [ ] The world advances in fixed ticks independently of display frequency.
- [ ] Resource costs and failed-development/death outcomes cannot create free resources.

Before claiming adaptation or intelligence:

- [ ] I can design independent-seed studies and appropriate controls rather than relying on a showcase run.
- [ ] I know why neuron count, body size, or survival duration alone does not establish learning or general intelligence.
- [ ] I can separate inherited capability, recurrent memory, plasticity, and body advantages in a comparison.
- [ ] I can explain numerical tolerances, timestep sensitivity, and CPU/GPU reproducibility limits.
- [ ] I know which observations are allowed and how to prevent privileged world-state leakage.
- [ ] I can reproduce a checkpointed experiment and keep final test worlds separate from selection/tuning.

## 11. Your immediate learning order

Start with Exercise A while learning practical Rust ownership/data modeling. Then complete Exercise B and the PyO3/maturin round trip in Exercise C. Those establish readiness for the first real core. Continue with recurrent execution, inheritance, and experimental controls as the matching milestones are built.

Skip general Python/PyTorch beginner courses unless an exercise exposes a gap. Defer advanced Rust metaprogramming, unsafe optimization, distributed training, custom CUDA/Triton kernels, elaborate rendering, detailed neuroscience, and full 3D physics until the project provides a measured reason to use them. Progress by demonstrated competence and working artifacts rather than an invented calendar estimate.


## Review the inherited-body/brain implementation

Read `agents.rs` for local inputs/strict action and learned-inheritance validation, `circle_world.rs` for contact movement/feeding, `evolution.rs` for resource-funded development/births, `plastic.py` for lifetime weight learning and stable state migration, and `ecosystem.py`/`control.py` for complete saves and lightweight observation. Compare the active plastic controller with `evolving.py`, the preserved fixed-weight reference. Capacity changes at births; lifetime neural growth remains future work.

Practice tracing one input/edge/output by organism and gene ID, one rejected batch without memory changes, one funded child whose inherited parameters match its parent, and one local injury/repair with balanced resources. Explain the distinction between physical growth, birth-time topology mutation, recurrent memory, and lifetime learning. Lifetime weight plasticity now executes in the circle mode; lifetime neural growth remains future work.

For experiments, inspect the clone/brain-only/body-only/combined configurations and all seed results, then the held-out ancestral/descendant comparisons. A changed outcome is not automatically an improvement or evidence of intelligence. For performance, read the component profiler before choosing GPU, precision, or interface changes; the current float64/JSON/full-audit path is a reference baseline.
