# First world rules: v0.1

Status: `circle_rules.json` (`circle-world-v0.3`) and `circle_habitat.json` define the active single-circle experiment. Legacy coefficients below remain for the independent reference and earlier controls. Circle mode adds bounded online plasticity, explicit learning costs, a maintenance reserve and learned-parameter inheritance; the laws below still govern all resource transfers.

The machine-readable coefficients are in [configs/world_rules_v0.json](configs/world_rules_v0.json). The reference implementation is [reference/ledger.py](reference/ledger.py), with the fixture in [examples/lifecycle_expected.json](examples/lifecycle_expected.json). The [native example](examples/native_lifecycle.py) checks the same expectations through Rust; its interface and limits are documented in [resource accounting](docs/resource_accounting.md). These coefficients are proposed normalized model parameters, not measured biological constants.

## 1. Scope and environment

The initial target is a bounded 2D habitat, 64 model-length units on each side, with a 64 x 64 environmental grid. Walls are closed to nutrient/material transfer. There is no wrapping, immigration, automatic food respawn, automatic organism replacement, or external mineral input in the baseline. External light is the sole replenishable energy source; its available quantity must be supplied explicitly by the world configuration.

The first organisms use one shared model with explicit capabilities. Controlled producer tissue can capture light and absorb minerals. Consumers can ingest accessible living tissue. Decomposition begins as a controlled aggregate process. These are initial capabilities, not permanent lineage classes. Evolving producers/recyclers, broader substrates, and full body modules follow the roadmap.

World updates target a fixed physics timestep of 1/60 model-time unit. Controllers decide every six physics ticks; environmental-field updates occur every six ticks. Display frequency is independent. These are proposed numerical settings to validate when the spatial world is implemented.

Neither ledger advances a global simulation clock, constructs the grid, solves diffusion or mechanics, or resolves multi-agent spatial claims. Calls are isolated authorized process windows. The caller provides contact/placement decisions and durations. The worked sequence is a transaction storyboard, not a full population simulation or a fixed-timestep run.

The separate `Habitat` now supplies fixed-duration producer windows and occupied-cell access to the Rust physiology functions. It uses stationary point positions, equal initial shares of each cell's available light/minerals, acquisition before maintenance, and maintenance before growth. Unused shares are not redistributed during that window. New incident light and expired unabsorbed light are recorded explicitly; cell inventories are authoritative and ledger pool totals are aggregates of them. See the [habitat contract](docs/resource_accounting.md#headless-producer-habitat) for initialization, scheduling, boundaries, and current exclusions. These controlled choices are numerical/model assumptions, not claims of natural ecological realism.

## 2. Units and disjoint compartments

| Symbol | Meaning | Rule |
| --- | --- | --- |
| L | Model length | A normalized spatial unit; no real-world length is claimed |
| T | Model time | Rates integrate over a declared positive duration, not wall-clock time |
| N | Limiting nutrient/building-material unit | Conserved across tracked pools; not a claim to model every chemical element |
| E | Energy unit | Stored chemically, stored mechanically, or dissipated; new energy enters only as absorbed external light |

Track these compartments exactly once:

- Available mineral N in the environment.
- Living structural tissue: bound N and embodied chemical E.
- Living reserves: stored N and usable chemical E.
- Detritus: dead tissue and organic waste, retaining remaining N/E.
- Mechanical E: tracked work/kinetic/potential storage where applicable.
- Cumulative dissipated heat E, unavailable to organisms in this model.

Mineral nutrients contain no usable energy. N is an abstract limiting currency; other carrier materials and detailed chemistry are implicit. The reference does not claim complete elemental or physical mass conservation. Tissue biomass/mass coupling will use this declared abstraction in the Rust body model.

Available external light is an input budget outside the initial chemical-energy inventory. Absorbing 10 E adds 10 E to the accounted input, even when only 8 E becomes usable reserve energy and 2 E immediately dissipates. Unabsorbed light is not counted as stored world energy.

Global audits, initially with absolute and relative tolerances of 1e-8:

```text
initial N = environmental mineral N + all living tissue/reserve N + detrital N

initial chemical E + initial mechanical E + cumulative absorbed light E
  = living tissue/reserve E + detrital E + tracked mechanical E + cumulative heat E
```

The reference starts with zero mechanical energy and no exports. Future imports/exports require explicit new ledger terms. Biological transfer errors and later physics-solver residuals are measured separately.

## 3. Organism state and viability

Each organism has a unique immutable identity, an inherited genome, tissue resources, reserve resources, age, parent identity, reproductive cooldown, alive/dead status, and an optional death cause. The reference genome contains capability flags and an immutable weight tuple solely to demonstrate parameter inheritance; it is not an executing ANN.

The v0 tissue relation is E_tissue = 2 * N_tissue. Initial organisms and all transfers must preserve it. Tissue below 1 N is nonviable. An organism at exactly 1 N can remain alive if it can fund maintenance. Mature organisms require age at least 2 T and tissue at least 6 N. Offspring begin with 2 N of tissue and age zero, so they start juvenile.

Reserve capacity scales with current tissue: at most 1 stored N and 5 stored E per tissue N. Overflow N returns to environmental minerals; overflow E becomes heat. This is explicit saturation, not unrecorded deletion. Tissue loss can reduce reserve capacity and cause the same accounted overflow. The early capacity coefficients are fixed physiology; evolving storage anatomy will replace them with costed module-derived capacities.

## 4. Resource transfer rules

Let x be a requested nutrient/tissue amount and dt the authorized positive process duration. All quantities must be finite and nonnegative; active requests must be positive. Invalid requests do not partially commit.

| Process | Preconditions | Transfers and limits |
| --- | --- | --- |
| Light capture | Living producer capability; available external light | Absorb at most 10 E/T * dt. Of absorbed light, 0.8 enters usable energy reserve and 0.2 becomes heat; reserve overflow also becomes heat |
| Mineral uptake | Living producer uptake capability; available mineral N | Transfer at most 1 N/T * dt from minerals to reserve N; overflow returns to minerals |
| Ingestion | Living consumer; another living target; authorized physical contact | Consume at most 2 tissue N/T * dt and never more than existing target tissue. Prepay digestion cost of 0.25 E per ingested N |
| Assimilation | An accepted bite | Remove x N and 2x E from target tissue. Assimilate 0.75 of each into reserves; the rest enters detritus. Do not also award the target's remaining reserves |
| Growth / tissue repair accounting | Sufficient stored N/E | Construct at most 1 tissue N/T * dt. For x N of new tissue, spend x reserve N and 3x reserve E; retain 2x E in tissue and dissipate x E as construction heat |
| Maintenance | Living tissue | Spend 0.1 E/(tissue N * T) * current tissue N * dt. Fully paid maintenance advances age by dt. If underfunded, dissipate the remaining reserve E, advance to the end of the authorized window, and die |
| Sensing accounting | Living organism; funded authorized sensing | Spend 0.02 E/T * dt as heat. The reference does not generate observations; functional sensor access is defined by the later body model |
| Actuation accounting | Living organism; prepaid requested energy | Of requested expenditure, 0.5 becomes tracked mechanical E and 0.5 becomes heat. There are no positions/velocities in the reference |
| Mechanical dissipation | Existing tracked mechanical E | Transfer the requested amount from mechanical E to heat; cannot dissipate more than exists |
| Tissue damage | Authorized tissue-loss event | Transfer lost tissue N/E to detritus. If remaining tissue is below 1 N, transfer all remaining tissue/reserves once and mark dead |
| Aggregate decomposition | Existing detritus; explicit fraction in (0, 1] | Process that fraction of the pool, limited by both 20 N/T and 100 E/T times dt. Return consumed N to minerals and dissipate consumed E as heat |

The two decomposition limits also handle energy-only organic waste without division by a zero nutrient amount. Nutrient-only waste can return minerals without creating energy. Later living recyclers retain explicitly accounted nutrients/energy in their own biomass and replace or partition the aggregate process.

Maintenance uses reserves only. There is no tissue catabolism or arbitrary maximum age in v0. Starvation can therefore occur while tissue still contains embodied energy; that energy enters detritus rather than automatically rescuing the organism. This is a declared simplifying policy, not a claim about universal biology.

The reference damage operation assumes a permitted tissue-loss event. It does not model a predator's attack mechanics or supply a free action to a controller. The later interaction/physics system must establish contact, capability, damage amount, and attacker/action cost before authorizing it.

## 5. Resource-funded birth and inherited weights

A parent must be alive, mature, past its reproductive cooldown, and able to fund a child at an authorized local placement. New IDs cannot reuse living or dead organism identities.

The first birth recipe is:

```text
child tissue:       2 N + 4 E
child reserves:     0.5 N + 5 E
construction heat:  2 E
parent debit:       2.5 reserve N + 11 reserve E
```

Parent debit, child creation, ancestry, construction heat, and cooldown commit together. The parent becomes eligible again no earlier than one additional T of age. The child starts age zero with inherited encoded parameters and capability flags. There is no full-network reinitialization, mutation, learned-weight assimilation, recurrent execution, or genotype-to-module development in this reference.

If a request lacks resources or placement, no child or heat is created and no parent debit occurs. No repeated free search for a favorable genome is performed. Later biologically failed development is a distinct resource-funded process; it must not be confused with rejecting an invalid transaction before development begins.

## 6. Mortality, ordering, and atomicity

Death is irreversible. A dead record retains identity/genome/lineage metadata but no tissue, reserves, sensing, feeding, growth, or reproductive activity. Repeated damage/death/feeding claims on that dead organism are rejected. Existing mechanical energy remains tracked until explicitly dissipated; death does not erase it.

The future fixed-timestep scheduler must authorize each organism/process window once, enforce shared feeding/uptake/placement budgets, and update all ages consistently. The intended initial order is: funded sensing/actions and physics/damage; light capture and uptake; accessible feeding; mandatory maintenance and mortality; growth/repair; reproduction; aggregate decomposition; event/audit completion. Newborns first act at their next eligible controller boundary. Storage saturation and threshold mortality occur within the relevant transaction.

Resource claims use a consistent decision snapshot and a documented seeded resolution policy. Conflicting mouths cannot each receive the same tissue. The current reference serializes explicit transactions to establish conservation/failure semantics; it does not claim to have implemented fair simultaneous resolution, a spatial solver, or the production scheduler. Repeated standalone calls represent separate authorized process windows, not a way for a production controller to bypass rate limits.

Every reference operation checks the prior ledger, tentatively updates state, audits the result, then records a committed event. If validation or the commit audit fails, resources, organism state, and event history roll back together. Negative/NaN/infinite inputs, impossible contacts, overdraws, excessive per-window rates, stale/dead IDs, and repeated births during cooldown are rejected.

## 7. Hand-calculated lifecycle example

The fixture deliberately uses 1 T process windows for readable arithmetic. It is not one physics tick and does not apply maintenance to every organism automatically.

Initial inventory: 100 N and 100 chemical E. Of N, 75 is free mineral, 15 belongs to the producer, and 10 belongs to the consumer. The producer starts with 20 tissue E + 30 reserve E; the consumer with 16 tissue E + 34 reserve E. External light budget is 10 E.

1. Producer absorbs 10 light E: gain 8 reserve E; dissipate 2 E.
2. Consumer ingests 2 producer tissue N / 4 E: assimilate 1.5 N / 3 E, leave 0.5 N / 1 E as detritus, and dissipate 0.5 reserve E for digestion.
3. Consumer grows 1 tissue N / 2 E: spend 1 reserve N / 3 reserve E; dissipate 1 E.
4. Consumer maintains 9 tissue N for 1 T: spend 0.9 E and age from 3 to 4 T.
5. Consumer funds the child recipe: transfer 2.5 N / 9 E into the child and dissipate 2 E. Parent retains 9 tissue N / 18 E and 21.6 reserve E.
6. Producer loses all 8 remaining tissue N to damage and dies: its remaining resources join waste, producing 13.5 detrital N / 55 E in total.
7. Aggregate decomposition processes that pool: return 13.5 N to minerals and dissipate 55 E.

Final independent expectation: 88.5 free mineral N + 11.5 living N = 100 N. Living chemical E is 48.6; heat is 61.4; accounted E is 110, equal to initial 100 E plus absorbed light 10 E. The consumer and juvenile child are alive; the producer is dead. These are fixture outcomes, not a claim of ecological equilibrium or intelligence.

## 8. Run and acceptance checks

From the project root:

```powershell
python -m reference.worked_example --output artifacts/lifecycle.json
python -m unittest discover -s tests -p test_resource_ledger.py -v
```

The worked example compares every snapshot with separately written expected values. The tests cover conservation, resource saturation, upfront digestion, finite light, growth, maturity/cooldown, funded inheritance, starvation, damage, one-time death, partial/energy-only decomposition, mechanical energy, invalid inputs, rollback, and a seeded mixed-transaction exercise.

These checks establish agreement between the contract and transaction reference. They do not validate a spatial simulator, neural learner, genetic operators, or viewer. Revised biological rules require a new model/configuration identity before experiments are compared.

## Current reference clarification

The active `circle-world-v0.3` reference has a 40 x 22.5 domain, 40 x 23 resource cells, 1 initial mineral N per cell, conservative mineral diffusion, bounded useful ingestion, and a declared 15% inaccessible producer grazing residual. This is a particular abstract ecological model, not calibrated natural biology. The computational ceiling is 512 organisms and is reported as a constraint when reached. Old checkpoints retain their own settings. Growth, repair and reproduction are still automatic until the neural-action interface is expanded.
