# Project walkthrough

Read one file at a time. Each expandable entry starts with its project role, then separates imports, classes/structures, methods, standalone functions, and useful syntax. Open a function to see a short real code excerpt and why it exists.

Excerpts are copied from the linked source, not rewritten examples. Long bodies are deliberately shortened; the line link opens the complete file. Methods are grouped under their own class or Rust type. Shared files show current contents, including additions made later than their original group.

**Maintenance:** update this same walkthrough after each completed capability, including changed shared files. Keep code excerpts and file coverage current; do not create a separate explanation document for every file.

**Current state:** the default has simple circles, bounded feeding/development, conservative mineral diffusion, online recurrent parameter learning, inherited variation, persistent diagnostic capture, full-window observation and complete learning continuation. The reference declares protected grazing tissue. Legacy experiments remain available; current outcomes and limits are in artifacts/setup/ecosystem-ab-verification.json.

**Remaining expansion:** social communication/group hunting, differentiated bodies and diets, living recyclers, lifetime neural topology growth, large-population scaling, general evaluation, and release remain. The parts-scavenging robot mode stays deferred. A local learned feeding result does not establish general intelligence.

**Reading Rust:** `fn` defines a function, `struct` defines a record, `enum` lists alternatives, and `impl` groups methods. `pub` makes an item public; `Self` means the current type. `&self` reads an object, `&mut self` can change it, `Result` carries success/error, and `?` returns an error immediately. `Vec` is a growable list; `Option` represents a value that may be absent.

**Units:** N = nutrient, E = energy, T = model time. These are normalized model units.

## Step 1: Rules and Python reference

<details><summary><code>PLAN.md</code></summary>

**Purpose:** The overall design: survival, inherited variation, functional bodies/brains, diverse food webs, local social communication, injury/repair, and a readable navigable world.

[Open the file](C:/Thought/Projects/World/PLAN.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>CORE_PLAYERS.md</code></summary>

**Purpose:** Defines producer/grazer/predator/scavenger/recycler founder designs, independent brains and social signals, injury/repair, resource pools, and experiment responsibilities.

[Open the file](C:/Thought/Projects/World/CORE_PLAYERS.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>IMPLEMENTATION_ROADMAP.md</code></summary>

**Purpose:** The ordered build plan and acceptance checks, including where diagnostics, differentiated organisms, damage, social hunting, and scalable observation enter the remaining work.

[Open the file](C:/Thought/Projects/World/IMPLEMENTATION_ROADMAP.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>LEARNING_GUIDE.md</code></summary>

**Purpose:** The preparation guide for a Python/PyTorch developer learning Rust and simulation design.

[Open the file](C:/Thought/Projects/World/LEARNING_GUIDE.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>WORLD_RULES.md</code></summary>

**Purpose:** The shared biological rules and resource-conservation contract used to judge implementations.

[Open the file](C:/Thought/Projects/World/WORLD_RULES.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>configs/world_rules_v0.json</code></summary>

**Purpose:** The actual coefficient values read by both ledgers; spatial settings are reserved for later integration.

[Open the file](C:/Thought/Projects/World/configs/world_rules_v0.json)

[Source lines 1–6](C:/Thought/Projects/World/configs/world_rules_v0.json:1)

```json
{
  "schema_version": 1,
  "model_id": "world-rules-v0.1",
  "world_width": 64.0,
  "world_height": 64.0,
  "grid_width": 64,
```

These objects/lists supply configuration or expected data to the validated code. Booleans are `true`/`false`; each named field has one value.

</details>

<details><summary><code>reference/__init__.py</code></summary>

**Purpose:** Marks and describes the independent Python reference package.

[Open the file](C:/Thought/Projects/World/reference/__init__.py)

[Source lines 1–1](C:/Thought/Projects/World/reference/__init__.py:1)

```python
"""Small executable model-contract reference; not the production simulator."""
```

These declarations describe the package or expose neighboring modules; no independent runtime algorithm is defined here.

</details>

<details><summary><code>reference/rules.py</code></summary>

**Purpose:** Loads the coefficient file and prevents invalid rules from entering the Python reference.

[Open the file](C:/Thought/Projects/World/reference/rules.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from dataclasses import dataclass, fields
import json
import math
from pathlib import Path
```

- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `json`: conversion between dictionaries and the native JSON interface.
- `math`: independent finite-value checks and inventory sums in tests.
- `pathlib`: configuration and output paths.

**Class: `WorldRules`**

One validated rule record shared by all operations, so coefficients stay consistent.

[Source lines 16–23](C:/Thought/Projects/World/reference/rules.py:16)

```python
@dataclass(frozen=True)
class WorldRules:
    schema_version: int
    model_id: str
    world_width: float
    world_height: float
    grid_width: int
    grid_height: int
```

**Methods — open the one you are reading**

<details><summary><code>__post_init__()</code></summary>

Validate schema, positive coefficients, fractions, integer settings, and newborn viability/capacities.

[Source lines 54–59](C:/Thought/Projects/World/reference/rules.py:54)

```python
    def __post_init__(self) -> None:
        if type(self.schema_version) is not int or self.schema_version != 1:
            raise ValueError("Only world-rules schema_version 1 is supported")
        if not isinstance(self.model_id, str) or not self.model_id.strip():
            raise ValueError("model_id must be a nonempty string")
        for field in fields(self):
```

</details>

**Standalone functions**

<details><summary><code>finite_nonnegative()</code></summary>

Reject Booleans, nonnumeric/nonfinite/negative inputs and optionally zero.

[Source lines 9–13](C:/Thought/Projects/World/reference/rules.py:9)

```python
def finite_nonnegative(value: float, name: str, *, positive: bool = False) -> None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{name} must be a finite number")
    if not math.isfinite(value) or value < 0 or (positive and value == 0):
        raise ValueError(f"{name} must be finite and {'positive' if positive else 'nonnegative'}")
```

</details>

<details><summary><code>load_rules()</code></summary>

Read the coefficient JSON and construct the validated rule record.

[Source lines 79–84](C:/Thought/Projects/World/reference/rules.py:79)

```python
def load_rules(path: str | Path | None = None) -> WorldRules:
    source = Path(path) if path is not None else Path(__file__).resolve().parents[1] / "configs/world_rules_v0.json"
    payload = json.loads(source.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("Rules must be a JSON object")
    return WorldRules(**payload)
```

</details>

**Useful syntax in this file**

- `@dataclass` builds record constructors; `frozen=True` prevents normal reassignment, and `slots=True` fixes instance fields.

</details>

<details><summary><code>reference/ledger.py</code></summary>

**Purpose:** A small independent model for checking resource transfers, inheritance, mortality, and rollback.

[Open the file](C:/Thought/Projects/World/reference/ledger.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from dataclasses import dataclass, replace
from functools import wraps
from inspect import signature
import math
from types import MappingProxyType
from typing import Mapping
from .rules import WorldRules, finite_nonnegative
```

- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `functools`: preserving decorated function metadata.
- `inspect`: capturing method arguments.
- `math`: independent finite-value checks and inventory sums in tests.
- `types`: read-only mapping access.
- `typing`: type annotations.
- `.rules`: the shared Python rule record and finite-number validator.

**Class: `InvariantViolation`**

A dedicated error for a broken accounting condition, distinct from an ordinary invalid request.

[Source lines 17–18](C:/Thought/Projects/World/reference/ledger.py:17)

```python
class InvariantViolation(RuntimeError):
    """The complete inventory no longer matches its declared sources."""
```

**Class: `Resources`**

A nutrient/energy pair used for tissue, reserves, and detritus; checked arithmetic prevents overspending.

[Source lines 21–24](C:/Thought/Projects/World/reference/ledger.py:21)

```python
@dataclass(frozen=True)
class Resources:
    nutrient: float = 0.0
    energy: float = 0.0
```

**Methods — open the one you are reading**

<details><summary><code>__post_init__()</code></summary>

Validate both resource amounts immediately so invalid records cannot be constructed.

[Source lines 26–28](C:/Thought/Projects/World/reference/ledger.py:26)

```python
    def __post_init__(self) -> None:
        finite_nonnegative(self.nutrient, "nutrient")
        finite_nonnegative(self.energy, "energy")
```

</details>

<details><summary><code>plus()</code></summary>

Return validated resource sums.

[Source lines 30–31](C:/Thought/Projects/World/reference/ledger.py:30)

```python
    def plus(self, other: "Resources") -> "Resources":
        return Resources(self.nutrient + other.nutrient, self.energy + other.energy)
```

</details>

<details><summary><code>minus()</code></summary>

Reject overdrawing either compartment and return the remainder.

[Source lines 33–36](C:/Thought/Projects/World/reference/ledger.py:33)

```python
    def minus(self, other: "Resources") -> "Resources":
        if other.nutrient > self.nutrient or other.energy > self.energy:
            raise ValueError("Insufficient resources")
        return Resources(self.nutrient - other.nutrient, self.energy - other.energy)
```

</details>

**Class: `Genome`**

Inherited capability flags and encoded weights. These parameters are data; no neural controller executes here.

[Source lines 39–44](C:/Thought/Projects/World/reference/ledger.py:39)

```python
@dataclass(frozen=True)
class Genome:
    photosynthetic: bool = False
    can_ingest: bool = False
    # Inherited parameter fixture, not an executing neural network.
    initial_weights: tuple[float, ...] = ()
```

**Methods — open the one you are reading**

<details><summary><code>__post_init__()</code></summary>

Require real capability Booleans and an immutable tuple of finite inherited parameters.

[Source lines 46–51](C:/Thought/Projects/World/reference/ledger.py:46)

```python
    def __post_init__(self) -> None:
        if type(self.photosynthetic) is not bool or type(self.can_ingest) is not bool:
            raise ValueError("Capabilities must be booleans")
        if not isinstance(self.initial_weights, tuple):
            raise ValueError("Inherited weights must be an immutable tuple")
        for weight in self.initial_weights:
```

</details>

**Class: `Organism`**

The changing individual state, kept separate from its inherited blueprint.

[Source lines 56–63](C:/Thought/Projects/World/reference/ledger.py:56)

```python
@dataclass(frozen=True)
class Organism:
    organism_id: str
    genome: Genome
    tissue: Resources
    reserves: Resources
    age: float = 0.0
    alive: bool = True
```

**Methods — open the one you are reading**

<details><summary><code>__post_init__()</code></summary>

Validate identity, component types, age, cooldown, and supported death causes.

[Source lines 68–73](C:/Thought/Projects/World/reference/ledger.py:68)

```python
    def __post_init__(self) -> None:
        if not isinstance(self.organism_id, str) or not self.organism_id:
            raise ValueError("Organism ID must be a nonempty string")
        if not isinstance(self.genome, Genome) or not isinstance(self.tissue, Resources) or not isinstance(self.reserves, Resources):
            raise ValueError("Organism requires typed genome, tissue and reserve records")
        if type(self.alive) is not bool:
```

</details>

<details><summary><code>mature()</code></summary>

Require living state, minimum individual age, and minimum tissue for reproduction.

[Source lines 82–84](C:/Thought/Projects/World/reference/ledger.py:82)

```python
    def mature(self, rules: WorldRules) -> bool:
        return (self.alive and self.age >= rules.minimum_reproduction_age
                and self.tissue.nutrient >= rules.maturity_tissue_nutrient)
```

</details>

**Class: `Ledger`**

The independent Python implementation used to check the native resource model.

[Source lines 107–107](C:/Thought/Projects/World/reference/ledger.py:107)

```python
class Ledger:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Create this object and store the state its other methods need.

[Source lines 108–113](C:/Thought/Projects/World/reference/ledger.py:108)

```python
    def __init__(self, rules: WorldRules, *, mineral_nutrient: float,
                 organisms: list[Organism], initial_nutrient: float,
                 initial_energy: float, light_available: float = 0.0,
                 detritus: Resources = Resources()):
        self.rules = rules
        for name, value in (("mineral_nutrient", mineral_nutrient), ("initial_nutrient", initial_nutrient),
```

</details>

<details><summary><code>organisms()</code></summary>

Expose immutable organism records or the combined committed/proposed audit view.

[Source lines 131–133](C:/Thought/Projects/World/reference/ledger.py:131)

```python
    def organisms(self) -> Mapping[str, Organism]:
        # Snapshot readers must not observe tentative writes.
        return MappingProxyType(dict(self._organisms))
```

</details>

<details><summary><code>events()</code></summary>

Return protected access to committed event history.

[Source lines 136–138](C:/Thought/Projects/World/reference/ledger.py:136)

```python
    def events(self) -> tuple[dict, ...]:
        # Keep callers from mutating committed history.
        return tuple({**event, "arguments": dict(event["arguments"])} for event in self._events)
```

</details>

<details><summary><code>_checkpoint()</code></summary>

Save all changing reference-ledger state and history length.

[Source lines 140–142](C:/Thought/Projects/World/reference/ledger.py:140)

```python
    def _checkpoint(self):
        return (dict(self._organisms), self._mineral, self._detritus, self._light_available,
                self._absorbed_light, self._heat, self._mechanical, len(self._events))
```

</details>

<details><summary><code>_restore()</code></summary>

Restore the checkpoint and remove tentative history.

[Source lines 144–147](C:/Thought/Projects/World/reference/ledger.py:144)

```python
    def _restore(self, state) -> None:
        (self._organisms, self._mineral, self._detritus, self._light_available,
         self._absorbed_light, self._heat, self._mechanical, event_count) = state
        del self._events[event_count:]
```

</details>

<details><summary><code>_living()</code></summary>

Find a living record or reject unknown/dead identity.

[Source lines 149–154](C:/Thought/Projects/World/reference/ledger.py:149)

```python
    def _living(self, organism_id: str) -> Organism:
        if organism_id not in self._organisms:
            raise ValueError(f"Unknown organism: {organism_id}")
        organism = self._organisms[organism_id]
        if not organism.alive:
            raise ValueError(f"Organism is dead: {organism_id}")
```

</details>

<details><summary><code>_rate_check()</code></summary>

Require positive amount/duration and enforce amount <= rate times duration.

[Source lines 157–161](C:/Thought/Projects/World/reference/ledger.py:157)

```python
    def _rate_check(self, amount: float, duration: float, rate: float) -> None:
        finite_nonnegative(amount, "amount", positive=True)
        finite_nonnegative(duration, "duration", positive=True)
        if amount > rate * duration:
            raise ValueError("Requested amount exceeds this integration window's process capacity")
```

</details>

<details><summary><code>_set_reserves()</code></summary>

Apply tissue-based capacities; excess nutrient returns to minerals and energy to heat.

[Source lines 163–168](C:/Thought/Projects/World/reference/ledger.py:163)

```python
    def _set_reserves(self, organism: Organism, reserves: Resources) -> Organism:
        n_cap = organism.tissue.nutrient * self.rules.reserve_nutrient_capacity_per_tissue_nutrient
        e_cap = organism.tissue.nutrient * self.rules.reserve_energy_capacity_per_tissue_nutrient
        # Account for overflow as mineral return or heat.
        self._mineral += max(0.0, reserves.nutrient - n_cap)
        self._heat += max(0.0, reserves.energy - e_cap)
```

</details>

<details><summary><code>_debit_energy()</code></summary>

Return an updated organism copy with less reserve energy.

[Source lines 171–173](C:/Thought/Projects/World/reference/ledger.py:171)

```python
    def _debit_energy(self, organism: Organism, amount: float) -> Organism:
        finite_nonnegative(amount, "energy", positive=True)
        return replace(organism, reserves=organism.reserves.minus(Resources(energy=amount)))
```

</details>

<details><summary><code>_die()</code></summary>

Transfer remaining tissue/reserves once to detritus and store an empty dead record.

[Source lines 175–178](C:/Thought/Projects/World/reference/ledger.py:175)

```python
    def _die(self, organism: Organism, cause: str) -> None:
        self._detritus = self._detritus.plus(organism.tissue).plus(organism.reserves)
        self._organisms[organism.organism_id] = replace(
            organism, tissue=Resources(), reserves=Resources(), alive=False, death_cause=cause)
```

</details>

<details><summary><code>_after_tissue_loss()</code></summary>

Kill below viability or resize reserves to the remaining tissue.

[Source lines 180–184](C:/Thought/Projects/World/reference/ledger.py:180)

```python
    def _after_tissue_loss(self, organism: Organism, cause: str) -> None:
        if organism.tissue.nutrient < self.rules.minimum_viable_tissue_nutrient:
            self._die(organism, cause)
        else:
            self._organisms[organism.organism_id] = self._set_reserves(organism, organism.reserves)
```

</details>

<details><summary><code>audit()</code></summary>

Check valid organism/resource state and conserved nutrient/energy budgets.

[Source lines 186–191](C:/Thought/Projects/World/reference/ledger.py:186)

```python
    def audit(self) -> dict[str, float]:
        n_live = math.fsum(o.tissue.nutrient + o.reserves.nutrient for o in self._organisms.values())
        e_live = math.fsum(o.tissue.energy + o.reserves.energy for o in self._organisms.values())
        nutrient = math.fsum((self._mineral, self._detritus.nutrient, n_live))
        energy = math.fsum((self._detritus.energy, e_live, self._heat, self._mechanical))
        expected_energy = self._initial_energy + self._absorbed_light
```

</details>

<details><summary><code>summary()</code></summary>

Return the audited aggregate resource inventory.

[Source lines 219–224](C:/Thought/Projects/World/reference/ledger.py:219)

```python
    def summary(self, label: str) -> dict:
        totals = self.audit()
        return {"label": label, "mineral_nutrient": self._mineral,
                "living_nutrient": math.fsum(o.tissue.nutrient + o.reserves.nutrient for o in self._organisms.values()),
                "living_energy": math.fsum(o.tissue.energy + o.reserves.energy for o in self._organisms.values()),
                "detritus_nutrient": self._detritus.nutrient, "detritus_energy": self._detritus.energy,
```

</details>

<details><summary><code>photosynthesize()</code></summary>

Spend available light and divide captured input between reserves and heat.

[Source lines 231–236](C:/Thought/Projects/World/reference/ledger.py:231)

```python
    def photosynthesize(self, organism_id: str, light_energy: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        if not organism.genome.photosynthetic:
            raise ValueError("Organism has no light-capture capability")
        self._rate_check(light_energy, duration, self.rules.max_absorbed_light_per_time)
        if light_energy > self._light_available:
```

</details>

<details><summary><code>absorb()</code></summary>

Transfer existing minerals into capable reserves within processing/storage limits.

[Source lines 245–250](C:/Thought/Projects/World/reference/ledger.py:245)

```python
    def absorb(self, organism_id: str, nutrient: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        self._rate_check(nutrient, duration, self.rules.max_absorption_nutrient_per_time)
        if not organism.genome.photosynthetic:
            raise ValueError("No passive mineral-uptake capability in this v0 configuration")
        if nutrient > self._mineral:
```

</details>

<details><summary><code>feed()</code></summary>

Validate contact/capability; prepay digestion; remove target tissue; split assimilation/waste; check viability.

[Source lines 256–261](C:/Thought/Projects/World/reference/ledger.py:256)

```python
    def feed(self, consumer_id: str, target_id: str, tissue_nutrient: float,
             *, contact: bool, duration: float = 1.0) -> None:
        consumer, target = self._living(consumer_id), self._living(target_id)
        if consumer_id == target_id or not consumer.genome.can_ingest or contact is not True:
            raise ValueError("Feeding requires ingestion capability and authorized contact with another organism")
        self._rate_check(tissue_nutrient, duration, self.rules.max_ingestion_nutrient_per_time)
```

</details>

<details><summary><code>grow()</code></summary>

Spend nutrient and energy reserves on new tissue and construction heat.

[Source lines 273–278](C:/Thought/Projects/World/reference/ledger.py:273)

```python
    def grow(self, organism_id: str, tissue_nutrient: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        self._rate_check(tissue_nutrient, duration, self.rules.max_growth_nutrient_per_time)
        embodied = tissue_nutrient * self.rules.tissue_energy_per_nutrient
        heat = tissue_nutrient * self.rules.construction_heat_per_nutrient
        reserves = organism.reserves.minus(Resources(tissue_nutrient, embodied + heat))
```

</details>

<details><summary><code>maintain()</code></summary>

Pay tissue-based maintenance, advance individual age, and commit starvation if underfunded.

[Source lines 284–289](C:/Thought/Projects/World/reference/ledger.py:284)

```python
    def maintain(self, organism_id: str, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        finite_nonnegative(duration, "duration", positive=True)
        required = organism.tissue.nutrient * self.rules.maintenance_per_tissue_nutrient_per_time * duration
        paid = min(required, organism.reserves.energy)
        updated = replace(organism, age=organism.age + duration,
```

</details>

<details><summary><code>reproduce()</code></summary>

Check maturity/cooldown/new ID/placement, fund the child entirely from parent reserves, and clone the genome.

[Source lines 298–303](C:/Thought/Projects/World/reference/ledger.py:298)

```python
    def reproduce(self, parent_id: str, child_id: str, *, placement_available: bool) -> Organism:
        parent = self._living(parent_id)
        if not isinstance(child_id, str) or not child_id or child_id in self._organisms:
            raise ValueError("Child ID must be new and nonempty")
        if not parent.mature(self.rules) or parent.age < parent.next_reproduction_age or placement_available is not True:
            raise ValueError("Reproduction requires maturity, cooldown completion and authorized placement")
```

</details>

<details><summary><code>damage()</code></summary>

Transfer destroyed tissue to detritus and check remaining viability/capacity.

[Source lines 318–323](C:/Thought/Projects/World/reference/ledger.py:318)

```python
    def damage(self, organism_id: str, tissue_nutrient: float) -> None:
        organism = self._living(organism_id)
        finite_nonnegative(tissue_nutrient, "damaged tissue", positive=True)
        lost = Resources(tissue_nutrient, tissue_nutrient * self.rules.tissue_energy_per_nutrient)
        remaining = organism.tissue.minus(lost)
        self._detritus = self._detritus.plus(lost)
```

</details>

<details><summary><code>decompose()</code></summary>

Process existing detritus within separate nutrient/energy rate caps; minerals and heat receive outputs.

[Source lines 327–332](C:/Thought/Projects/World/reference/ledger.py:327)

```python
    def decompose(self, *, fraction: float, duration: float = 1.0) -> None:
        finite_nonnegative(fraction, "fraction", positive=True)
        finite_nonnegative(duration, "duration", positive=True)
        if fraction > 1 or self._detritus == Resources():
            raise ValueError("Decomposition requires a fraction at most 1 of existing detritus")
        consumed = (self._detritus if fraction == 1 else
```

</details>

<details><summary><code>sense()</code></summary>

Spend sensing energy as heat.

[Source lines 342–347](C:/Thought/Projects/World/reference/ledger.py:342)

```python
    def sense(self, organism_id: str, *, duration: float = 1.0) -> None:
        finite_nonnegative(duration, "duration", positive=True)
        organism = self._living(organism_id)
        cost = self.rules.sensor_energy_per_time * duration
        self._organisms[organism_id] = self._debit_energy(organism, cost)
        self._heat += cost
```

</details>

<details><summary><code>actuate()</code></summary>

Spend reserves on mechanical energy plus heat.

[Source lines 350–355](C:/Thought/Projects/World/reference/ledger.py:350)

```python
    def actuate(self, organism_id: str, energy: float) -> None:
        organism = self._living(organism_id)
        self._organisms[organism_id] = self._debit_energy(organism, energy)
        work = energy * self.rules.movement_efficiency
        self._mechanical += work
        self._heat += energy - work
```

</details>

<details><summary><code>dissipate_motion()</code></summary>

Transfer existing mechanical energy to heat.

[Source lines 358–363](C:/Thought/Projects/World/reference/ledger.py:358)

```python
    def dissipate_motion(self, energy: float) -> None:
        finite_nonnegative(energy, "mechanical dissipation", positive=True)
        if energy > self._mechanical:
            raise ValueError("Cannot dissipate unavailable mechanical energy")
        self._mechanical -= energy
        self._heat += energy
```

</details>

**Standalone functions**

<details><summary><code>transaction()</code></summary>

Wrap the reference operation with audit, checkpoint, receipt, and rollback.

[Source lines 87–92](C:/Thought/Projects/World/reference/ledger.py:87)

```python
def transaction(method):
    """Commit once, or restore every resource/organism/event field on failure."""
    @wraps(method)
    def checked(self, *args, **kwargs):
        self.audit()
        before = self._checkpoint()
```

</details>

**Nested helpers**

These functions live inside another function or method; they are not additional public entry points.

Inside `transaction`:

<details><summary><code>checked() — nested helper</code></summary>

Audit, checkpoint, execute, audit, append receipt; restore and re-raise on failure.

[Source lines 90–95](C:/Thought/Projects/World/reference/ledger.py:90)

```python
    def checked(self, *args, **kwargs):
        self.audit()
        before = self._checkpoint()
        try:
            result = method(self, *args, **kwargs)
            self.audit()
```

</details>

Inside `Ledger.audit`:

<details><summary><code>close() — nested helper</code></summary>

Compare finite values using the larger of absolute and relative numerical tolerance.

[Source lines 196–198](C:/Thought/Projects/World/reference/ledger.py:196)

```python
        def close(a, b):
            return math.isclose(a, b, rel_tol=self.rules.relative_tolerance,
                                abs_tol=self.rules.absolute_tolerance)
```

</details>

**Useful syntax in this file**

- `@dataclass` builds record constructors; `frozen=True` prevents normal reassignment, and `slots=True` fixes instance fields.
- `@transaction` applies the shared audit/checkpoint/rollback wrapper to an operation.
- `@property` exposes a method as a read-only attribute.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>reference/worked_example.py</code></summary>

**Purpose:** Runs a prescribed lifecycle and compares eight results with independent expected values.

[Open the file](C:/Thought/Projects/World/reference/worked_example.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
from dataclasses import asdict
import json
import math
from pathlib import Path
from .ledger import Genome, Ledger, Organism, Resources
from .rules import WorldRules, load_rules
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `json`: conversion between dictionaries and the native JSON interface.
- `math`: independent finite-value checks and inventory sums in tests.
- `pathlib`: configuration and output paths.
- `.ledger`: the independent reference organisms, resources, genomes, and ledger.
- `.rules`: the shared Python rule record and finite-number validator.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 13–13](C:/Thought/Projects/World/reference/worked_example.py:13)

```python
FIXTURE_PATH = Path(__file__).resolve().parents[1] / "examples/lifecycle_expected.json"
```

**Standalone functions**

<details><summary><code>load_fixture()</code></summary>

Read fixture JSON; the installed package helper also constructs the native world.

[Source lines 16–17](C:/Thought/Projects/World/reference/worked_example.py:16)

```python
def load_fixture() -> dict:
    return json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
```

</details>

<details><summary><code>build_reference_world()</code></summary>

Adapt the independent expected fixture into a declared-budget reference ledger.

[Source lines 20–25](C:/Thought/Projects/World/reference/worked_example.py:20)

```python
def build_reference_world(rules: WorldRules | None = None) -> Ledger:
    rules = rules or load_rules()
    fixture = load_fixture()
    if fixture["model_id"] != rules.model_id:
        raise ValueError("Reference fixture and model rule versions differ")
    organisms = [Organism(
```

</details>

<details><summary><code>check_expected()</code></summary>

Check identical fields and expected numbers with configured floating-point tolerances.

[Source lines 36–41](C:/Thought/Projects/World/reference/worked_example.py:36)

```python
def check_expected(actual: dict, expected: dict, rules: WorldRules) -> None:
    if actual.keys() != expected.keys():
        raise AssertionError("Worked-example snapshot fields differ from the hand-calculated fixture")
    for key, value in expected.items():
        if isinstance(value, (float, int)):
            matches = math.isclose(actual[key], value, rel_tol=rules.relative_tolerance,
```

</details>

<details><summary><code>run_example()</code></summary>

Run seven prescribed operations, collect eight summaries, and check independent expected results.

[Source lines 49–54](C:/Thought/Projects/World/reference/worked_example.py:49)

```python
def run_example() -> tuple[Ledger, list[dict]]:
    world = build_reference_world()
    actions = [
        ("photosynthesis", lambda: world.photosynthesize("producer", 10.0)),
        ("feeding", lambda: world.feed("consumer", "producer", 2.0, contact=True)),
        ("growth", lambda: world.grow("consumer", 1.0)),
```

</details>

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 72–77](C:/Thought/Projects/World/reference/worked_example.py:72)

```python
def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Optional JSON trace destination")
    args = parser.parse_args()
    world, snapshots = run_example()
    for row in snapshots:
```

</details>

**Useful syntax in this file**

- `allow_nan=False` prevents nonfinite numbers from being encoded as JSON.

</details>

<details><summary><code>examples/lifecycle_expected.json</code></summary>

**Purpose:** Initial organisms and separately calculated expected resource inventories.

[Open the file](C:/Thought/Projects/World/examples/lifecycle_expected.json)

[Source lines 1–6](C:/Thought/Projects/World/examples/lifecycle_expected.json:1)

```json
{
  "model_id": "world-rules-v0.1",
  "description": "Hand-calculated transaction storyboard, not an autonomous world run.",
  "initial_inventory": {
    "nutrient": 100.0,
    "chemical_energy": 100.0,
```

These objects/lists supply configuration or expected data to the validated code. Booleans are `true`/`false`; each named field has one value.

</details>

<details><summary><code>tests/test_resource_ledger.py</code></summary>

**Purpose:** Checks the reference rules, calculations, boundary conditions, and rollback.

[Open the file](C:/Thought/Projects/World/tests/test_resource_ledger.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from dataclasses import FrozenInstanceError, replace
import math
import random
import unittest
from reference.ledger import Genome, InvariantViolation, Ledger, Organism, Resources
from reference.rules import load_rules
from reference.worked_example import build_reference_world, check_expected, load_fixture, run_example
```

- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `math`: independent finite-value checks and inventory sums in tests.
- `random`: repeatable pseudorandom test inputs.
- `unittest`: fresh test cases, assertions, and expected exceptions.
- `reference.ledger`: matching project types and validation used by this file.
- `reference.rules`: matching project types and validation used by this file.
- `reference.worked_example`: matching project types and validation used by this file.

**Class: `ResourceContractTests`**

A test group that constructs fresh fixtures and checks related behavior.

[Source lines 13–13](C:/Thought/Projects/World/tests/test_resource_ledger.py:13)

```python
class ResourceContractTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>setUp()</code></summary>

Create fresh state before each test.

[Source lines 14–16](C:/Thought/Projects/World/tests/test_resource_ledger.py:14)

```python
    def setUp(self):
        self.rules = load_rules()
        self.world = build_reference_world()
```

</details>

<details><summary><code>assertUnchangedFailure()</code></summary>

Require the expected exception and compare complete state before/after.

[Source lines 18–23](C:/Thought/Projects/World/tests/test_resource_ledger.py:18)

```python
    def assertUnchangedFailure(self, operation, exception=ValueError):
        before = self.world.summary("state"), dict(self.world.organisms), self.world.events
        with self.assertRaises(exception):
            operation()
        after = self.world.summary("state"), dict(self.world.organisms), self.world.events
        self.assertEqual(before, after)
```

</details>

<details><summary><code>make_single()</code></summary>

Construct a balanced reference ledger for a particular boundary case.

[Source lines 25–30](C:/Thought/Projects/World/tests/test_resource_ledger.py:25)

```python
    def make_single(self, *, tissue=8.0, energy=20.0, reserve_nutrient=5.0,
                    age=3.0, genome=None, mineral=10.0, light=0.0, detritus=Resources()):
        organism = Organism("solo", genome or Genome(can_ingest=True),
                            Resources(tissue, tissue * self.rules.tissue_energy_per_nutrient),
                            Resources(reserve_nutrient, energy), age=age)
        return Ledger(self.rules, mineral_nutrient=mineral, organisms=[organism],
```

</details>

<details><summary><code>test_full_lifecycle_matches_independently_written_expected_snapshots()</code></summary>

Checks full lifecycle matches independently written expected snapshots.

[Source lines 35–40](C:/Thought/Projects/World/tests/test_resource_ledger.py:35)

```python
    def test_full_lifecycle_matches_independently_written_expected_snapshots(self):
        world, snapshots = run_example()
        self.assertEqual(len(snapshots), 8)
        for actual, expected in zip(snapshots, load_fixture()["expected_snapshots"], strict=True):
            check_expected(actual, expected, self.rules)
        self.assertAlmostEqual(world.summary("final")["mineral_nutrient"], 88.5)
```

</details>

<details><summary><code>test_initial_inventories_are_checked_instead_of_inferred()</code></summary>

Checks initial inventories are checked instead of inferred.

[Source lines 44–47](C:/Thought/Projects/World/tests/test_resource_ledger.py:44)

```python
    def test_initial_inventories_are_checked_instead_of_inferred(self):
        with self.assertRaises(InvariantViolation):
            Ledger(self.rules, mineral_nutrient=75.0, organisms=list(self.world.organisms.values()),
                   initial_nutrient=101.0, initial_energy=100.0)
```

</details>

<details><summary><code>test_duplicate_initial_ids_rejected()</code></summary>

Checks duplicate initial ids rejected.

[Source lines 49–53](C:/Thought/Projects/World/tests/test_resource_ledger.py:49)

```python
    def test_duplicate_initial_ids_rejected(self):
        organism = self.world.organisms["producer"]
        with self.assertRaises(ValueError):
            Ledger(self.rules, mineral_nutrient=0, organisms=[organism, organism],
                   initial_nutrient=30, initial_energy=100)
```

</details>

<details><summary><code>test_tissue_energy_density_is_part_of_the_contract()</code></summary>

Checks tissue energy density is part of the contract.

[Source lines 55–58](C:/Thought/Projects/World/tests/test_resource_ledger.py:55)

```python
    def test_tissue_energy_density_is_part_of_the_contract(self):
        bad = replace(self.world.organisms["producer"], tissue=Resources(10.0, 21.0))
        with self.assertRaises(InvariantViolation):
            Ledger(self.rules, mineral_nutrient=0, organisms=[bad], initial_nutrient=15, initial_energy=51)
```

</details>

<details><summary><code>test_light_has_a_finite_external_budget_and_capability()</code></summary>

Checks light has a finite external budget and capability.

[Source lines 60–65](C:/Thought/Projects/World/tests/test_resource_ledger.py:60)

```python
    def test_light_has_a_finite_external_budget_and_capability(self):
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("consumer", 1))
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 11))
        self.world.photosynthesize("producer", 10)
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 1))
        self.assertAlmostEqual(self.world.organisms["producer"].reserves.energy, 38)
```

</details>

<details><summary><code>test_energy_storage_saturation_is_explicit_heat()</code></summary>

Checks energy storage saturation is explicit heat.

[Source lines 67–71](C:/Thought/Projects/World/tests/test_resource_ledger.py:67)

```python
    def test_energy_storage_saturation_is_explicit_heat(self):
        self.world = self.make_single(tissue=8, energy=39, genome=Genome(photosynthetic=True), light=10)
        self.world.photosynthesize("solo", 10)
        self.assertAlmostEqual(self.world.organisms["solo"].reserves.energy, 40)
        self.assertAlmostEqual(self.world.summary("after")["heat"], 9)
```

</details>

<details><summary><code>test_absorption_returns_overflow_to_minerals()</code></summary>

Checks absorption returns overflow to minerals.

[Source lines 73–77](C:/Thought/Projects/World/tests/test_resource_ledger.py:73)

```python
    def test_absorption_returns_overflow_to_minerals(self):
        self.world = self.make_single(tissue=8, reserve_nutrient=8, genome=Genome(photosynthetic=True))
        self.world.absorb("solo", 1)
        self.assertEqual(self.world.organisms["solo"].reserves.nutrient, 8)
        self.assertEqual(self.world.summary("after")["mineral_nutrient"], 10)
```

</details>

<details><summary><code>test_absorption_requires_available_material_and_rate_capacity()</code></summary>

Checks absorption requires available material and rate capacity.

[Source lines 79–83](C:/Thought/Projects/World/tests/test_resource_ledger.py:79)

```python
    def test_absorption_requires_available_material_and_rate_capacity(self):
        self.assertUnchangedFailure(lambda: self.world.absorb("producer", 2))
        self.assertUnchangedFailure(lambda: self.world.absorb("consumer", 1))
        self.world = self.make_single(mineral=0, genome=Genome(photosynthetic=True))
        self.assertUnchangedFailure(lambda: self.world.absorb("solo", 1))
```

</details>

<details><summary><code>test_feeding_removes_substrate_and_charges_digestion()</code></summary>

Checks feeding removes substrate and charges digestion.

[Source lines 85–90](C:/Thought/Projects/World/tests/test_resource_ledger.py:85)

```python
    def test_feeding_removes_substrate_and_charges_digestion(self):
        self.world.feed("consumer", "producer", 2, contact=True)
        self.assertEqual(self.world.organisms["producer"].tissue, Resources(8, 16))
        self.assertEqual(self.world.organisms["consumer"].reserves, Resources(3.5, 36.5))
        row = self.world.summary("after")
        self.assertEqual((row["detritus_nutrient"], row["detritus_energy"], row["heat"]), (0.5, 1, 0.5))
```

</details>

<details><summary><code>test_contact_self_feeding_and_excess_bites_are_rejected()</code></summary>

Checks contact self feeding and excess bites are rejected.

[Source lines 92–96](C:/Thought/Projects/World/tests/test_resource_ledger.py:92)

```python
    def test_contact_self_feeding_and_excess_bites_are_rejected(self):
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=False))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "consumer", 1, contact=True))
        self.assertUnchangedFailure(lambda: self.world.feed("producer", "consumer", 1, contact=True))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 3, contact=True))
```

</details>

<details><summary><code>test_already_eaten_tissue_cannot_be_claimed_twice()</code></summary>

Checks already eaten tissue cannot be claimed twice.

[Source lines 98–101](C:/Thought/Projects/World/tests/test_resource_ledger.py:98)

```python
    def test_already_eaten_tissue_cannot_be_claimed_twice(self):
        self.world.feed("consumer", "producer", 10, contact=True, duration=5)
        self.assertFalse(self.world.organisms["producer"].alive)
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=True))
```

</details>

<details><summary><code>test_unaffordable_digestion_leaves_food_untouched()</code></summary>

Checks unaffordable digestion leaves food untouched.

[Source lines 103–108](C:/Thought/Projects/World/tests/test_resource_ledger.py:103)

```python
    def test_unaffordable_digestion_leaves_food_untouched(self):
        hungry = replace(self.world.organisms["consumer"], reserves=Resources(2, 0.1))
        producer = self.world.organisms["producer"]
        self.world = Ledger(self.rules, mineral_nutrient=75, organisms=[producer, hungry],
                            initial_nutrient=100, initial_energy=66.1, light_available=10)
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 2, contact=True))
```

</details>

<details><summary><code>test_growth_pays_both_embodied_energy_and_construction_loss()</code></summary>

Checks growth pays both embodied energy and construction loss.

[Source lines 110–115](C:/Thought/Projects/World/tests/test_resource_ledger.py:110)

```python
    def test_growth_pays_both_embodied_energy_and_construction_loss(self):
        self.world.grow("consumer", 1)
        self.assertEqual(self.world.organisms["consumer"].tissue, Resources(9, 18))
        self.assertEqual(self.world.organisms["consumer"].reserves, Resources(1, 31))
        self.assertEqual(self.world.summary("after")["heat"], 1)
        self.assertUnchangedFailure(lambda: self.world.grow("consumer", 2))
```

</details>

<details><summary><code>test_growth_cannot_use_light_as_material()</code></summary>

Checks growth cannot use light as material.

[Source lines 117–121](C:/Thought/Projects/World/tests/test_resource_ledger.py:117)

```python
    def test_growth_cannot_use_light_as_material(self):
        self.world = self.make_single(reserve_nutrient=0, energy=30,
                                       genome=Genome(photosynthetic=True), light=10)
        self.world.photosynthesize("solo", 10)
        self.assertUnchangedFailure(lambda: self.world.grow("solo", 1))
```

</details>

<details><summary><code>test_maintenance_exact_funding_and_age()</code></summary>

Checks maintenance exact funding and age.

[Source lines 123–128](C:/Thought/Projects/World/tests/test_resource_ledger.py:123)

```python
    def test_maintenance_exact_funding_and_age(self):
        self.world = self.make_single(energy=0.8)
        self.world.maintain("solo")
        self.assertTrue(self.world.organisms["solo"].alive)
        self.assertEqual(self.world.organisms["solo"].reserves.energy, 0)
        self.assertEqual(self.world.organisms["solo"].age, 4)
```

</details>

<details><summary><code>test_starvation_does_not_use_tissue_energy_as_free_maintenance()</code></summary>

Checks starvation does not use tissue energy as free maintenance.

[Source lines 132–137](C:/Thought/Projects/World/tests/test_resource_ledger.py:132)

```python
    def test_starvation_does_not_use_tissue_energy_as_free_maintenance(self):
        self.world = self.make_single(energy=0.2)
        self.world.maintain("solo")
        organism = self.world.organisms["solo"]
        self.assertFalse(organism.alive)
        self.assertEqual(organism.death_cause, "starvation")
```

</details>

<details><summary><code>test_birth_is_funded_and_preserves_inherited_weights()</code></summary>

Checks birth is funded and preserves inherited weights.

[Source lines 144–149](C:/Thought/Projects/World/tests/test_resource_ledger.py:144)

```python
    def test_birth_is_funded_and_preserves_inherited_weights(self):
        self.world = self.make_single(genome=Genome(can_ingest=True, initial_weights=(0.25, -0.5)))
        before = self.world.organisms["solo"]
        child = self.world.reproduce("solo", "baby", placement_available=True)
        parent = self.world.organisms["solo"]
        self.assertEqual(parent.reserves, Resources(2.5, 9))
```

</details>

<details><summary><code>test_birth_checks_maturity_placement_ids_and_cooldown()</code></summary>

Checks birth checks maturity placement ids and cooldown.

[Source lines 159–164](C:/Thought/Projects/World/tests/test_resource_ledger.py:159)

```python
    def test_birth_checks_maturity_placement_ids_and_cooldown(self):
        self.world = self.make_single(age=1)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.world = self.make_single(tissue=5)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.world = self.make_single()
```

</details>

<details><summary><code>test_unfunded_birth_cannot_create_resources_or_partial_child()</code></summary>

Checks unfunded birth cannot create resources or partial child.

[Source lines 171–176](C:/Thought/Projects/World/tests/test_resource_ledger.py:171)

```python
    def test_unfunded_birth_cannot_create_resources_or_partial_child(self):
        self.world = self.make_single(energy=10)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
        self.assertNotIn("baby", self.world.organisms)
        self.world = self.make_single(reserve_nutrient=2)
        self.assertUnchangedFailure(lambda: self.world.reproduce("solo", "baby", placement_available=True))
```

</details>

<details><summary><code>test_damage_transfers_all_resources_once_when_vital_tissue_fails()</code></summary>

Checks damage transfers all resources once when vital tissue fails.

[Source lines 178–183](C:/Thought/Projects/World/tests/test_resource_ledger.py:178)

```python
    def test_damage_transfers_all_resources_once_when_vital_tissue_fails(self):
        self.world.damage("producer", 10)
        row = self.world.summary("after")
        self.assertEqual(row["detritus_nutrient"], 15)
        self.assertEqual(row["detritus_energy"], 50)
        self.assertUnchangedFailure(lambda: self.world.damage("producer", 1))
```

</details>

<details><summary><code>test_exact_vital_threshold_is_alive_and_lower_is_dead()</code></summary>

Checks exact vital threshold is alive and lower is dead.

[Source lines 185–190](C:/Thought/Projects/World/tests/test_resource_ledger.py:185)

```python
    def test_exact_vital_threshold_is_alive_and_lower_is_dead(self):
        self.world.damage("producer", 9)
        self.assertTrue(self.world.organisms["producer"].alive)
        self.assertEqual(self.world.organisms["producer"].tissue.nutrient, 1)
        self.assertEqual(self.world.organisms["producer"].reserves, Resources(1, 5))
        self.world.damage("producer", 0.5)
```

</details>

<details><summary><code>test_decomposition_handles_partial_and_complete_substrate()</code></summary>

Checks decomposition handles partial and complete substrate.

[Source lines 193–198](C:/Thought/Projects/World/tests/test_resource_ledger.py:193)

```python
    def test_decomposition_handles_partial_and_complete_substrate(self):
        self.world.damage("producer", 10)
        self.world.decompose(fraction=0.5)
        row = self.world.summary("partial")
        self.assertEqual(row["detritus_nutrient"], 7.5)
        self.assertEqual(row["detritus_energy"], 25)
```

</details>

<details><summary><code>test_decomposition_supports_energy_only_waste_and_limits_processing()</code></summary>

Checks decomposition supports energy only waste and limits processing.

[Source lines 204–209](C:/Thought/Projects/World/tests/test_resource_ledger.py:204)

```python
    def test_decomposition_supports_energy_only_waste_and_limits_processing(self):
        self.world = self.make_single(detritus=Resources(0, 4))
        self.world.decompose(fraction=0.5)
        self.assertEqual(self.world.summary("after")["detritus_energy"], 2)
        self.assertEqual(self.world.summary("after")["heat"], 2)
        self.assertUnchangedFailure(lambda: self.world.decompose(fraction=1, duration=0.001))
```

</details>

<details><summary><code>test_movement_tracks_mechanical_storage_before_dissipation()</code></summary>

Checks movement tracks mechanical storage before dissipation.

[Source lines 212–217](C:/Thought/Projects/World/tests/test_resource_ledger.py:212)

```python
    def test_movement_tracks_mechanical_storage_before_dissipation(self):
        self.world.actuate("consumer", 4)
        row = self.world.summary("moving")
        self.assertEqual(row["mechanical_energy"], 2)
        self.assertEqual(row["heat"], 2)
        self.assertEqual(self.world.organisms["consumer"].reserves.energy, 30)
```

</details>

<details><summary><code>test_sensing_spends_energy_and_dead_organisms_cannot_act()</code></summary>

Checks sensing spends energy and dead organisms cannot act.

[Source lines 224–229](C:/Thought/Projects/World/tests/test_resource_ledger.py:224)

```python
    def test_sensing_spends_energy_and_dead_organisms_cannot_act(self):
        self.world.sense("consumer")
        self.assertAlmostEqual(self.world.organisms["consumer"].reserves.energy, 33.98)
        self.world.damage("consumer", 8)
        self.assertUnchangedFailure(lambda: self.world.sense("consumer"))
        self.assertUnchangedFailure(lambda: self.world.actuate("consumer", 1))
```

</details>

<details><summary><code>test_invalid_numbers_and_durations_cannot_mutate_inventory()</code></summary>

Checks invalid numbers and durations cannot mutate inventory.

[Source lines 233–238](C:/Thought/Projects/World/tests/test_resource_ledger.py:233)

```python
    def test_invalid_numbers_and_durations_cannot_mutate_inventory(self):
        for bad in (-1.0, math.nan, math.inf, -math.inf, True):
            with self.subTest(value=bad):
                self.assertUnchangedFailure(lambda: self.world.actuate("consumer", bad))
                self.assertUnchangedFailure(lambda: self.world.grow("consumer", bad))
                self.assertUnchangedFailure(lambda: self.world.maintain("consumer", duration=bad))
```

</details>

<details><summary><code>test_immutable_public_records_and_defensive_history()</code></summary>

Checks immutable public records and defensive history.

[Source lines 241–246](C:/Thought/Projects/World/tests/test_resource_ledger.py:241)

```python
    def test_immutable_public_records_and_defensive_history(self):
        held_snapshot = self.world.organisms
        with self.assertRaises(TypeError):
            self.world.organisms["other"] = self.world.organisms["producer"]
        with self.assertRaises(FrozenInstanceError):
            self.world.organisms["consumer"].age = 100
```

</details>

<details><summary><code>test_audit_failure_rolls_back_even_after_tentative_writes()</code></summary>

Checks audit failure rolls back even after tentative writes.

[Source lines 253–258](C:/Thought/Projects/World/tests/test_resource_ledger.py:253)

```python
    def test_audit_failure_rolls_back_even_after_tentative_writes(self):
        original = self.world.audit
        before = self.world.summary("state"), dict(self.world.organisms), self.world.events
        calls = 0
        def fail_on_commit():
            nonlocal calls
```

</details>

<details><summary><code>test_dead_body_does_not_destroy_existing_mechanical_energy()</code></summary>

Checks dead body does not destroy existing mechanical energy.

[Source lines 272–277](C:/Thought/Projects/World/tests/test_resource_ledger.py:272)

```python
    def test_dead_body_does_not_destroy_existing_mechanical_energy(self):
        self.world.actuate("consumer", 4)
        self.world.damage("consumer", 8)
        self.assertEqual(self.world.summary("after death")["mechanical_energy"], 2)
        self.world.dissipate_motion(2)
        self.assertEqual(self.world.summary("after dissipation")["mechanical_energy"], 0)
```

</details>

<details><summary><code>test_rate_limits_scale_with_the_authorized_integration_window()</code></summary>

Checks rate limits scale with the authorized integration window.

[Source lines 280–285](C:/Thought/Projects/World/tests/test_resource_ledger.py:280)

```python
    def test_rate_limits_scale_with_the_authorized_integration_window(self):
        self.assertUnchangedFailure(lambda: self.world.photosynthesize("producer", 2, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.feed("consumer", "producer", 1, contact=True, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.grow("consumer", 0.2, duration=0.1))
        self.assertUnchangedFailure(lambda: self.world.absorb("producer", 0.2, duration=0.1))
        self.world.grow("consumer", 0.1, duration=0.1)
```

</details>

<details><summary><code>test_seeded_mixed_transactions_preserve_inventory_or_fail_atomically()</code></summary>

Checks seeded mixed transactions preserve inventory or fail atomically.

[Source lines 288–293](C:/Thought/Projects/World/tests/test_resource_ledger.py:288)

```python
    def test_seeded_mixed_transactions_preserve_inventory_or_fail_atomically(self):
        rng = random.Random(73)
        for _ in range(200):
            operation = rng.choice((
                lambda: self.world.photosynthesize("producer", rng.uniform(0.01, 3)),
                lambda: self.world.absorb("producer", rng.uniform(0.01, 0.5)),
```

</details>

**Class: `RuleValidationTests`**

A test group that constructs fresh fixtures and checks related behavior.

[Source lines 312–312](C:/Thought/Projects/World/tests/test_resource_ledger.py:312)

```python
class RuleValidationTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>test_invalid_rules_rejected()</code></summary>

Checks invalid rules rejected.

[Source lines 313–318](C:/Thought/Projects/World/tests/test_resource_ledger.py:313)

```python
    def test_invalid_rules_rejected(self):
        rules = load_rules()
        for update in ({"schema_version": 2}, {"physics_dt": 0}, {"grid_width": 2.5},
                       {"energy_assimilation_fraction": 1.1}, {"movement_efficiency": math.nan},
                       {"offspring_tissue_nutrient": 0.5}, {"offspring_reserve_energy": 100},
                       {"offspring_reserve_nutrient": 3}, {"model_id": ""}):
```

</details>

<details><summary><code>test_resources_and_genomes_reject_invalid_values()</code></summary>

Checks resources and genomes reject invalid values.

[Source lines 322–327](C:/Thought/Projects/World/tests/test_resource_ledger.py:322)

```python
    def test_resources_and_genomes_reject_invalid_values(self):
        for operation in (lambda: Resources(-1, 0), lambda: Resources(0, math.nan),
                          lambda: Genome(initial_weights=(math.inf,)),
                          lambda: Genome(initial_weights=[0.1]), lambda: Genome(photosynthetic=1)):
            with self.assertRaises(ValueError):
                operation()
```

</details>

**Nested helpers**

These functions live inside another function or method; they are not additional public entry points.

Inside `ResourceContractTests.test_audit_failure_rolls_back_even_after_tentative_writes`:

<details><summary><code>fail_on_commit() — nested helper</code></summary>

Inject a final-audit failure after tentative writes to test rollback.

[Source lines 257–262](C:/Thought/Projects/World/tests/test_resource_ledger.py:257)

```python
        def fail_on_commit():
            nonlocal calls
            calls += 1
            if calls == 2:
                raise InvariantViolation("Injected commit audit failure")
            return original()
```

</details>

</details>

## Step 2: Build environment and native package

<details><summary><code>README.md</code></summary>

**Purpose:** The starting point for running the project and understanding what currently works.

[Open the file](C:/Thought/Projects/World/README.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>DEPENDENCIES.md</code></summary>

**Purpose:** Records installed dependencies, prerequisites, build commands, and verification evidence.

[Open the file](C:/Thought/Projects/World/DEPENDENCIES.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>.gitignore</code></summary>

**Purpose:** Keeps generated environments, binaries, caches, and experiment output out of normal Git tracking.

[Open the file](C:/Thought/Projects/World/.gitignore)

[Source lines 1–6](C:/Thought/Projects/World/.gitignore:1)

```text
__pycache__/
*.py[cod]
.venv/
artifacts/
target/
*.pyd
```

The declared fields are read by the project or its build tools. JSON booleans are `true`/`false`; lockfiles pin exact dependency versions and checksums.

</details>

<details><summary><code>rust-toolchain.toml</code></summary>

**Purpose:** Selects the project Rust compiler, formatter, and lint checker.

[Open the file](C:/Thought/Projects/World/rust-toolchain.toml)

[Source lines 1–4](C:/Thought/Projects/World/rust-toolchain.toml:1)

```json
[toolchain]
channel = "1.97.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

Section headers group settings; `key = value` assigns them. `.workspace = true` inherits the shared root setting. These values configure build tools.

</details>

<details><summary><code>Cargo.toml</code></summary>

**Purpose:** Connects our Rust crates and shares package metadata and dependency versions.

[Open the file](C:/Thought/Projects/World/Cargo.toml)

[Source lines 1–8](C:/Thought/Projects/World/Cargo.toml:1)

```toml
[workspace]
members = ["crates/world_core", "crates/world_bindings", "crates/world_genetics", "crates/world_viewer"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.97.1"
```

This is declarative data, so it has no runtime imports, classes, or functions. `workspace = true` inherits the reviewed package/dependency setting from the root manifest. The observer adds rendering dependencies without putting them into the headless core.

**Development performance**

These Cargo package overrides optimize simulation, genetics, JSON exchange and viewer work while retaining the cached development build. They change compilation settings rather than model coefficients or biological timesteps.

[Source lines 26–39](C:/Thought/Projects/World/Cargo.toml:26)

```toml
[profile.dev.package.world_core]
opt-level = 3

[profile.dev.package.world_genetics]
opt-level = 3

[profile.dev.package.world_viewer]
opt-level = 2

[profile.dev.package.serde_json]
opt-level = 3

[profile.dev.package.zmij]
opt-level = 3
```

</details>

<details><summary><code>Cargo.lock</code></summary>

**Purpose:** Records the exact resolved Cargo dependencies so builds use the reviewed versions.

[Open the file](C:/Thought/Projects/World/Cargo.lock)

[Source lines 1–6](C:/Thought/Projects/World/Cargo.lock:1)

```toml
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "accesskit"
```

The declared fields are read by the project or its build tools. JSON booleans are `true`/`false`; lockfiles pin exact dependency versions and checksums.

</details>

<details><summary><code>crates/world_core/Cargo.toml</code></summary>

**Purpose:** Defines the Python-independent library that owns native rules and calculations.

[Open the file](C:/Thought/Projects/World/crates/world_core/Cargo.toml)

[Source lines 1–6](C:/Thought/Projects/World/crates/world_core/Cargo.toml:1)

```json
[package]
name = "world_core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
```

Section headers group settings; `key = value` assigns them. `.workspace = true` inherits the shared root setting. These values configure build tools.

</details>

<details><summary><code>crates/world_core/src/lib.rs</code></summary>

**Purpose:** Exposes core modules, the version, and a small checked-addition diagnostic.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/lib.rs)

**Functions outside impl blocks**

<details><summary><code>checked_add()</code></summary>

Add signed native integers with overflow detection.

[Source lines 35–38](C:/Thought/Projects/World/crates/world_core/src/lib.rs:35)

```rust
/// Checked addition for verifying native calls.
pub fn checked_add(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_bindings/Cargo.toml</code></summary>

**Purpose:** Defines the compiled Python extension and its dependency on the native core.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/Cargo.toml)

[Source lines 1–6](C:/Thought/Projects/World/crates/world_bindings/Cargo.toml:1)

```json
[package]
name = "world_bindings"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
```

Section headers group settings; `key = value` assigns them. `.workspace = true` inherits the shared root setting. These values configure build tools.

</details>

<details><summary><code>crates/world_bindings/src/lib.rs</code></summary>

**Purpose:** Registers native Python classes, version information, and diagnostic functions.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/src/lib.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use pyo3::prelude::*;
use super::*;
use pyo3::exceptions::PyOverflowError;
use crate::world::PyFixtureWorld;
use crate::resources::PyResourceLedger;
use crate::habitat::PyHabitat;
```

- `pyo3`: Python-visible classes, methods, properties, and error translation.
- `super`: the surrounding module's items used by its internal tests.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Standalone functions**

<details><summary><code>core_version()</code></summary>

Return the compiled core version.

[Source lines 25–27](C:/Thought/Projects/World/crates/world_bindings/src/lib.rs:25)

```rust
    fn core_version() -> &'static str {
        world_core::VERSION
    }
```

</details>

<details><summary><code>checked_add()</code></summary>

Add signed native integers with overflow detection.

[Source lines 31–34](C:/Thought/Projects/World/crates/world_bindings/src/lib.rs:31)

```rust
    fn checked_add(a: i64, b: i64) -> PyResult<i64> {
        world_core::checked_add(a, b)
            .ok_or_else(|| PyOverflowError::new_err("sum exceeds signed 64-bit range"))
    }
```

</details>

<details><summary><code>init()</code></summary>

Register the native extension version during module initialization.

[Source lines 37–39](C:/Thought/Projects/World/crates/world_bindings/src/lib.rs:37)

```rust
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", env!("CARGO_PKG_VERSION"))
    }
```

</details>

**Useful syntax in this file**

- `#[pymodule]` registers the native Python module; `#[pyfunction]` exposes a function.

</details>

<details><summary><code>pyproject.toml</code></summary>

**Purpose:** Tells Python/Maturin how to build and install the mixed Python/Rust package.

[Open the file](C:/Thought/Projects/World/pyproject.toml)

[Source lines 1–8](C:/Thought/Projects/World/pyproject.toml:1)

```toml
[build-system]
requires = ["maturin==1.16.0"]
build-backend = "maturin"

[project]
name = "world-microcosm"
dynamic = ["version"]
description = "Survival-driven artificial-life simulation"
```

This file is declarative data rather than runtime classes/functions. Workspace declarations share exact reviewed package/dependency settings; native genetics/core remain independent of the viewer.

</details>

<details><summary><code>python/world/__init__.py</code></summary>

**Purpose:** Makes the intended public classes and functions convenient to import.

[Open the file](C:/Thought/Projects/World/python/world/__init__.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from ._native import FixtureWorld, __version__, checked_add, core_version
from .accounting import ResourceLedger
from .habitat import Habitat
```

- `._native`: the compiled Rust habitat, aliased to distinguish it from the Python wrapper.
- `.accounting`: the convenient public Python resource-ledger wrapper.
- `.habitat`: the public habitat wrapper exported from the package.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 7–7](C:/Thought/Projects/World/python/world/__init__.py:7)

```python
__all__ = ["FixtureWorld", "Habitat", "ResourceLedger", "__version__", "checked_add", "core_version"]
```

`__all__` names the public imports: `FixtureWorld`, `ResourceLedger`, version, and scalar diagnostics.

</details>

<details><summary><code>dependencies/python.in</code></summary>

**Purpose:** The human-maintained list of direct Python dependency versions.

[Open the file](C:/Thought/Projects/World/dependencies/python.in)

[Source lines 3–8](C:/Thought/Projects/World/dependencies/python.in:3)

```json
torch==2.14.1+cu126
numpy==2.5.4
maturin==1.16.0
ncps==1.0.1
matplotlib==3.11.2
pandas==3.1.1
```

`package==version` pins a direct Python dependency; the corresponding lock includes its resolved transitive dependencies.

</details>

<details><summary><code>dependencies/python-win-py313.lock</code></summary>

**Purpose:** The resolved Python dependencies and accepted artifact hashes.

[Open the file](C:/Thought/Projects/World/dependencies/python-win-py313.lock)

[Source lines 3–8](C:/Thought/Projects/World/dependencies/python-win-py313.lock:3)

```json
--index-url https://pypi.org/simple

boolean-py==5.0 \
    --hash=sha256:60cbc4bad079753721d32649545505362c754e121570ada4658b852a3a318d95 \
    --hash=sha256:ef28a70bd43115208441b53a045d1549e2f0ec6e3d08a9d142cbc41c1938e8d9
    # via license-expression
```

Exact versions fix dependency resolution; hashes identify accepted package contents. Repeated entries are generated dependency records, so they are not duplicated here.

</details>

<details><summary><code>dependencies/build-tools.in</code></summary>

**Purpose:** Requests the project-local CMake build utility.

[Open the file](C:/Thought/Projects/World/dependencies/build-tools.in)

[Source lines 3–3](C:/Thought/Projects/World/dependencies/build-tools.in:3)

```json
cmake==4.4.4
```

`package==version` pins a direct Python dependency; the corresponding lock includes its resolved transitive dependencies.

</details>

<details><summary><code>dependencies/build-tools-win-py313.lock</code></summary>

**Purpose:** Pins CMake and its accepted artifact hashes.

[Open the file](C:/Thought/Projects/World/dependencies/build-tools-win-py313.lock)

[Source lines 3–8](C:/Thought/Projects/World/dependencies/build-tools-win-py313.lock:3)

```json
--index-url https://pypi.org/simple

cmake==4.4.4 \
    --hash=sha256:02e9599cba932b386dc5c7e29db5dbda0610d13263798ff1521298c18014357d \
    --hash=sha256:3c3f40c4656ec1ad1b873dd502bb31b3bba75295ae66b2623e515f2931eddbcb \
    --hash=sha256:45a5c5e058e1e456f8da99e8bb83c98df079f385afd0bbea222a7eab6fdc1a8f \
```

Exact versions fix dependency resolution; hashes identify accepted package contents. Repeated entries are generated dependency records, so they are not duplicated here.

</details>

<details><summary><code>dependencies/deferred.json</code></summary>

**Purpose:** Records why the optional egui integration was deferred.

[Open the file](C:/Thought/Projects/World/dependencies/deferred.json)

[Source lines 1–6](C:/Thought/Projects/World/dependencies/deferred.json:1)

```json
{
  "bevy_egui": {
    "requested_version": "0.43.1",
    "status": "deferred",
    "reason": "The resolved graph includes ttf-parser 0.25.1, which RustSec marks unmaintained. This is an informational maintenance advisory, not a reported exploitable vulnerability. Deferred under the requested conservative dependency policy.",
    "advisory": "https://rustsec.org/advisories/RUSTSEC-2026-0192.html",
```

These objects/lists supply configuration or expected data to the validated code. Booleans are `true`/`false`; each named field has one value.

</details>

<details><summary><code>scripts/windows_toolchain.ps1</code></summary>

**Purpose:** Finds installed MSVC/SDK components and prepares process-local compiler paths.

[Open the file](C:/Thought/Projects/World/scripts/windows_toolchain.ps1)

<details><summary><code>Set-WorldBuildEnvironment()</code></summary>

Handles Set-WorldBuildEnvironment at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 1–6](C:/Thought/Projects/World/scripts/windows_toolchain.ps1:1)

```powershell
function Set-WorldBuildEnvironment {
    param([Parameter(Mandatory = $true)][string]$ProjectRoot)

    $taskPython = Join-Path $ProjectRoot '.venv\Scripts\python.exe'
    if (-not (Test-Path -LiteralPath $taskPython)) {
        throw 'Project .venv is missing; see DEPENDENCIES.md.'
```

</details>

**Execution and syntax:** `param` declares arguments; `$env:` accesses process environment; `&` invokes a command; `try/finally` restores the caller's environment. Native exit codes are checked, and `plain_command.py` captures UTF-8 output before PowerShell decoding.

</details>

<details><summary><code>scripts/audit_dependencies.py</code></summary>

**Purpose:** Checks pinned dependencies for known advisory matches before use.

[Open the file](C:/Thought/Projects/World/scripts/audit_dependencies.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
import urllib.request
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `datetime`: UTC verification timestamps.
- `hashlib`: input fingerprints.
- `json`: conversion between dictionaries and the native JSON interface.
- `pathlib`: configuration and output paths.
- `re`: dependency-line recognition and name normalization.
- `sys`: interpreter information, import paths, and exit status.
- `tomllib`: Cargo lock parsing.
- `urllib`: advisory HTTP requests.

**Standalone functions**

<details><summary><code>read_packages()</code></summary>

Read registry packages or pinned Python requirements from the selected lock.

[Source lines 18–23](C:/Thought/Projects/World/scripts/audit_dependencies.py:18)

```python
def read_packages(path: Path, kind: str) -> list[dict]:
    if kind == "cargo":
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        packages = [{"name": item["name"], "version": item["version"], "ecosystem": "crates.io"}
                    for item in data.get("package", []) if item.get("source", "").startswith("registry+")]
    else:
```

</details>

<details><summary><code>audit()</code></summary>

Query OSV in bounded batches and reject incomplete results before reporting known advisory matches.

[Source lines 36–41](C:/Thought/Projects/World/scripts/audit_dependencies.py:36)

```python
def audit(packages: list[dict]) -> list[dict]:
    findings = []
    for offset in range(0, len(packages), 100):
        batch = packages[offset:offset + 100]
        queries = [{"package": {"name": p["name"], "ecosystem": p["ecosystem"]},
                    "version": p["version"]} for p in batch]
```

</details>

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 57–62](C:/Thought/Projects/World/scripts/audit_dependencies.py:57)

```python
def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=("python", "cargo"))
    parser.add_argument("lock", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
```

</details>

**Useful syntax in this file**

- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>scripts/verify_dependencies.py</code></summary>

**Purpose:** Checks installed versions, array interchange, tools, and tiny CPU/CUDA neural computations.

[Open the file](C:/Thought/Projects/World/scripts/verify_dependencies.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
from datetime import datetime, timezone
import importlib.metadata
import json
from pathlib import Path
import re
import subprocess
import sys
import traceback
import matplotlib
import numpy as np
import pandas as pd
import torch
from ncps.torch import CfC, LTC
from audit_dependencies import read_packages
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `datetime`: UTC verification timestamps.
- `importlib`: installed-package metadata/native imports.
- `json`: conversion between dictionaries and the native JSON interface.
- `pathlib`: configuration and output paths.
- `re`: dependency-line recognition and name normalization.
- `subprocess`: running another interpreter/tool.
- `sys`: interpreter information, import paths, and exit status.
- `traceback`: error details in reports.
- `matplotlib`: the installed plotting dependency.
- `numpy`: typed host arrays and shape/layout/numeric checks.
- `pandas`: the installed table-analysis dependency.
- `torch`: CPU/CUDA tensors and recurrent diagnostic calculations.
- `ncps.torch`: matching project types and validation used by this file.
- `audit_dependencies`: the local pinned-package parser reused to compare installed versions against locks.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 14–14](C:/Thought/Projects/World/scripts/verify_dependencies.py:14)

```python
ROOT = Path(__file__).resolve().parents[1]
```

**Standalone functions**

<details><summary><code>verify()</code></summary>

Check project interpreter, exact locks, tools, array conversion, and tiny CPU/CUDA CfC/LTC forward/backward runs.

[Source lines 17–22](C:/Thought/Projects/World/scripts/verify_dependencies.py:17)

```python
def verify() -> dict:
    if Path(sys.prefix).resolve() != (ROOT / ".venv").resolve():
        raise RuntimeError("Run with .venv/Scripts/python.exe so global packages are not tested by mistake")
    import matplotlib
    matplotlib.use("Agg")
    import numpy as np
```

</details>

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 86–91](C:/Thought/Projects/World/scripts/verify_dependencies.py:86)

```python
def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/setup/python-verification.json")
    args = parser.parse_args()
    try:
        report = verify()
```

</details>

**Nested helpers**

These functions live inside another function or method; they are not additional public entry points.

Inside `verify`:

<details><summary><code>normalize() — nested helper</code></summary>

Standardize package-name punctuation and case for inventory comparison.

[Source lines 59–60](C:/Thought/Projects/World/scripts/verify_dependencies.py:59)

```python
    def normalize(name):
        return re.sub(r"[-_.]+", "-", name).lower()
```

</details>

**Useful syntax in this file**

- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>scripts/verify_dependencies.ps1</code></summary>

**Purpose:** Runs environment checks and optional Cargo checks while restoring environment settings.

[Open the file](C:/Thought/Projects/World/scripts/verify_dependencies.ps1)

**Script flow**

[Source lines 1–8](C:/Thought/Projects/World/scripts/verify_dependencies.ps1:1)

```json
param([switch]$IncludeRust)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskOldEnvironment = @{}
foreach ($taskKey in @('PATH', 'INCLUDE', 'LIB', 'PYO3_PYTHON', 'VIRTUAL_ENV')) {
    $taskOldEnvironment[$taskKey] = [Environment]::GetEnvironmentVariable($taskKey, 'Process')
}
```

`-IncludeRust` adds Cargo checking to Python environment checks. `$LASTEXITCODE` catches failed external tools; `finally` restores environment settings and the previous directory.

**Useful syntax:** `$name` is a variable; `$env:NAME` accesses a process environment setting; `&` runs the selected command.

**Useful syntax in this file**

- `finally` always runs cleanup, even when the operation fails.

</details>

<details><summary><code>scripts/build_native.ps1</code></summary>

**Purpose:** Audits dependencies and builds an editable extension or an installable wheel.

[Open the file](C:/Thought/Projects/World/scripts/build_native.ps1)

[Source lines 1–6](C:/Thought/Projects/World/scripts/build_native.ps1:1)

```powershell
param([switch]$Wheel)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskOldEnvironment = @{}
foreach ($taskKey in @('PATH', 'INCLUDE', 'LIB', 'PYO3_PYTHON', 'VIRTUAL_ENV', 'PIP_NO_INDEX', 'UV_OFFLINE')) {
```

**Execution and syntax:** `param` declares arguments; `$env:` accesses process environment; `&` invokes a command; `try/finally` restores the caller's environment. Native exit codes are checked, and `plain_command.py` captures UTF-8 output before PowerShell decoding.

</details>

<details><summary><code>tests/test_native_package.py</code></summary>

**Purpose:** Checks the real installed extension, integer conversions, versions, and isolated imports.

[Open the file](C:/Thought/Projects/World/tests/test_native_package.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import importlib.machinery
import importlib.metadata
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import world
from world import _native
```

- `importlib`: installed-package metadata/native imports.
- `json`: conversion between dictionaries and the native JSON interface.
- `pathlib`: configuration and output paths.
- `subprocess`: running another interpreter/tool.
- `sys`: interpreter information, import paths, and exit status.
- `tempfile`: temporary files/directories.
- `unittest`: fresh test cases, assertions, and expected exceptions.
- `world`: the installed public Habitat interface used by examples and tests.

**Class: `NativePackageTests`**

A test group that constructs fresh fixtures and checks related behavior.

[Source lines 16–16](C:/Thought/Projects/World/tests/test_native_package.py:16)

```python
class NativePackageTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>test_installed_distribution_matches_linked_core_and_extension()</code></summary>

Checks installed distribution matches linked core and extension.

[Source lines 17–22](C:/Thought/Projects/World/tests/test_native_package.py:17)

```python
    def test_installed_distribution_matches_linked_core_and_extension(self):
        version = importlib.metadata.version("world-microcosm")
        self.assertEqual(world.__version__, version)
        self.assertEqual(_native.__version__, version)
        self.assertEqual(world.core_version(), version)
        self.assertTrue(any(str(_native.__file__).endswith(suffix)
```

</details>

<details><summary><code>test_integer_values_survive_python_rust_round_trip()</code></summary>

Checks integer values survive python rust round trip.

[Source lines 25–28](C:/Thought/Projects/World/tests/test_native_package.py:25)

```python
    def test_integer_values_survive_python_rust_round_trip(self):
        for a, b in [(2, 3), (-7, 2), (2**53 + 1, 1), (2**63 - 1, 0), (-2**63, 0)]:
            with self.subTest(a=a, b=b):
                self.assertEqual(world.checked_add(a, b), a + b)
```

</details>

<details><summary><code>test_invalid_values_and_native_overflow_become_python_exceptions()</code></summary>

Checks invalid values and native overflow become python exceptions.

[Source lines 30–35](C:/Thought/Projects/World/tests/test_native_package.py:30)

```python
    def test_invalid_values_and_native_overflow_become_python_exceptions(self):
        for a, b in [(2**63 - 1, 1), (-2**63, -1), (2**63, 0)]:
            with self.subTest(a=a, b=b), self.assertRaises(OverflowError):
                world.checked_add(a, b)
        for invalid in [1.5, "2"]:
            with self.subTest(invalid=invalid), self.assertRaises(TypeError):
```

</details>

<details><summary><code>test_installed_import_works_outside_repository_without_pythonpath()</code></summary>

Checks installed import works outside repository without pythonpath.

[Source lines 38–43](C:/Thought/Projects/World/tests/test_native_package.py:38)

```python
    def test_installed_import_works_outside_repository_without_pythonpath(self):
        code = (
            "import json, world; from world import _native; "
            "print(json.dumps({'value': world.checked_add(2, 3), "
            "'version': world.core_version(), 'native': _native.__file__}))"
        )
```

</details>

**Useful syntax in this file**

- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

## Step 3: Diagnostic Rust and PyTorch bridge

<details><summary><code>configs/bridge_fixture.json</code></summary>

**Purpose:** Defines three fixed-signal test organisms, including an ID too large for safe floating-point storage.

[Open the file](C:/Thought/Projects/World/configs/bridge_fixture.json)

[Source lines 1–6](C:/Thought/Projects/World/configs/bridge_fixture.json:1)

```json
{
  "schema_version": 1,
  "organisms": [
    {"id": 17, "topology_version": 0, "signals": [0.25, 0.75, 0.5]},
    {"id": 42, "topology_version": 3, "signals": [0.875, 0.125, 0.25]},
    {"id": 9007199254740993, "topology_version": 1, "signals": [0.5, 0.5, 0.75]}
```

These objects/lists supply configuration or expected data to the validated code. Booleans are `true`/`false`; each named field has one value.

</details>

<details><summary><code>crates/world_core/src/clock.rs</code></summary>

**Purpose:** Tracks the diagnostic tick and rejects integer overflow.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/clock.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use crate::contracts::{ContractError, Result};
```

- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `Clock`**

A typed logical tick with checked advancement; it is not an ecological scheduler.

[Source lines 3–4](C:/Thought/Projects/World/crates/world_core/src/clock.rs:3)

```rust
#[derive(Clone, Copy, Debug)]
pub struct Clock(pub u64);
```

**Methods — open the one you are reading**

<details><summary><code>next()</code></summary>

Calculate the next unsigned fixture tick or reject overflow.

[Source lines 7–12](C:/Thought/Projects/World/crates/world_core/src/clock.rs:7)

```rust
    pub fn next(self) -> Result<Self> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| ContractError("tick overflow".into()))
    }
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.

</details>

<details><summary><code>crates/world_core/src/ids.rs</code></summary>

**Purpose:** Gives organism identities a distinct unsigned-integer type, shared by native components.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/ids.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use serde::{Deserialize, Serialize};
```

- `serde`: typed configuration input and inspectable structured output.

**Structure / enum: `OrganismId`**

An exact unsigned 64-bit identity, kept separate from array positions and floating-point values.

[Source lines 3–5](C:/Thought/Projects/World/crates/world_core/src/ids.rs:3)

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrganismId(pub u64);
```

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.
- `#[serde(transparent)]` represents the identity directly as a JSON integer.

</details>

<details><summary><code>crates/world_core/src/config.rs</code></summary>

**Purpose:** Parses and validates the fixed-signal fixture configuration.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/config.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use crate::contracts::{ContractError, OBSERVATION_WIDTH, Result, SCHEMA_VERSION};
use crate::ids::OrganismId;
```

- `std`: standard-library maps/sets for exact identities, lookup, and duplicate detection.
- `serde`: typed configuration input and inspectable structured output.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `FixtureOrganism`**

An identity, topology version, and three fixed signals for bridge testing.

[Source lines 8–14](C:/Thought/Projects/World/crates/world_core/src/config.rs:8)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureOrganism {
    pub id: OrganismId,
    pub topology_version: u64,
    pub signals: [f32; OBSERVATION_WIDTH],
}
```

**Structure / enum: `FixtureConfig`**

The versioned list of initial diagnostic organisms.

[Source lines 16–21](C:/Thought/Projects/World/crates/world_core/src/config.rs:16)

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureConfig {
    pub schema_version: u32,
    pub organisms: Vec<FixtureOrganism>,
}
```

**Methods — open the one you are reading**

<details><summary><code>from_json()</code></summary>

Decode typed JSON and run schema and record validation.

[Source lines 41–46](C:/Thought/Projects/World/crates/world_core/src/config.rs:41)

```rust
    pub fn from_json(text: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(text)
            .map_err(|error| ContractError(format!("invalid fixture config: {error}")))?;
        if config.schema_version != SCHEMA_VERSION {
            return Err(ContractError("unsupported config schema".into()));
        }
```

</details>

**Standalone functions**

<details><summary><code>validate_organisms()</code></summary>

Reject duplicate IDs and invalid fixed signals.

[Source lines 23–28](C:/Thought/Projects/World/crates/world_core/src/config.rs:23)

```rust
pub fn validate_organisms(organisms: &[FixtureOrganism]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for organism in organisms {
        if !seen.insert(organism.id) {
            return Err(ContractError("duplicate organism ID".into()));
        }
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.
- `deny_unknown_fields` rejects unexpected input fields instead of silently ignoring them.

</details>

<details><summary><code>crates/world_core/src/contracts.rs</code></summary>

**Purpose:** Defines observation/action batches and their schema, dimensions, and value bounds.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/contracts.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use std::fmt;
use crate::ids::OrganismId;
```

- `std`: standard-library maps/sets for exact identities, lookup, and duplicate detection.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `ContractError`**

A printable diagnostic contract error returned when a batch or fixture is invalid.

[Source lines 9–10](C:/Thought/Projects/World/crates/world_core/src/contracts.rs:9)

```rust
#[derive(Debug)]
pub struct ContractError(pub String);
```

**Methods — open the one you are reading**

<details><summary><code>fmt()</code></summary>

Format the stored error message for display.

[Source lines 13–15](C:/Thought/Projects/World/crates/world_core/src/contracts.rs:13)

```rust
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
```

</details>

**Structure / enum: `ObservationBatch`**

Three observation columns per identity, with schema, tick, and topology metadata.

[Source lines 22–29](C:/Thought/Projects/World/crates/world_core/src/contracts.rs:22)

```rust
#[derive(Clone, Debug)]
pub struct ObservationBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub ids: Vec<OrganismId>,
    pub topology_versions: Vec<u64>,
    pub values: Vec<[f32; OBSERVATION_WIDTH]>,
}
```

**Structure / enum: `ActionBatch`**

Two bounded action columns per identity, with the metadata needed to reject stale routing.

[Source lines 31–38](C:/Thought/Projects/World/crates/world_core/src/contracts.rs:31)

```rust
#[derive(Clone, Debug)]
pub struct ActionBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub ids: Vec<OrganismId>,
    pub topology_versions: Vec<u64>,
    pub values: Vec<[f32; ACTION_WIDTH]>,
}
```

**Standalone functions**

<details><summary><code>validate_action()</code></summary>

Require finite turn in [-1,1] and thrust in [0,1].

[Source lines 40–45](C:/Thought/Projects/World/crates/world_core/src/contracts.rs:40)

```rust
pub fn validate_action(action: &[f32; ACTION_WIDTH]) -> Result<()> {
    if !action.iter().all(|value| value.is_finite())
        || !(-1.0..=1.0).contains(&action[0])
        || !(0.0..=1.0).contains(&action[1])
    {
        return Err(ContractError(
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.

</details>

<details><summary><code>crates/world_core/src/world.rs</code></summary>

**Purpose:** Owns diagnostic state and routes accepted action batches by organism identity.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/world.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::clock::Clock;
use crate::config::{FixtureConfig, FixtureOrganism, validate_organisms};
use crate::contracts::{
    ACTION_WIDTH, ActionBatch, ContractError, ObservationBatch, Result, SCHEMA_VERSION,
    validate_action,
};
use crate::ids::OrganismId;
```

- `std`: standard-library maps/sets for exact identities, lookup, and duplicate detection.
- `serde`: typed configuration input and inspectable structured output.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `FixtureState`**

One diagnostic organism plus its last accepted action and accumulated action totals.

[Source lines 13–19](C:/Thought/Projects/World/crates/world_core/src/world.rs:13)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureState {
    organism: FixtureOrganism,
    last_action: [f32; ACTION_WIDTH],
    action_total: [f64; ACTION_WIDTH],
}
```

**Structure / enum: `Snapshot`**

The serializable fixture tick and organism state used for continuation.

[Source lines 21–27](C:/Thought/Projects/World/crates/world_core/src/world.rs:21)

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema_version: u32,
    tick: u64,
    organisms: Vec<FixtureState>,
}
```

**Structure / enum: `FixtureWorld`**

The fixed-signal native test world; its accumulated actions are not position, energy, or fitness.

[Source lines 29–33](C:/Thought/Projects/World/crates/world_core/src/world.rs:29)

```rust
pub struct FixtureWorld {
    clock: Clock,
    organisms: Vec<FixtureState>,
    index: BTreeMap<OrganismId, usize>,
}
```

**Methods — open the one you are reading**

<details><summary><code>from_config_json()</code></summary>

Construct a diagnostic world with validated identities/signals and zero actions.

[Source lines 36–41](C:/Thought/Projects/World/crates/world_core/src/world.rs:36)

```rust
    pub fn from_config_json(text: &str) -> Result<Self> {
        let config = FixtureConfig::from_json(text)?;
        let organisms = config
            .organisms
            .into_iter()
            .map(|organism| FixtureState {
```

</details>

<details><summary><code>restore()</code></summary>

Validate saved fixture state and rebuild its ID lookup.

[Source lines 54–59](C:/Thought/Projects/World/crates/world_core/src/world.rs:54)

```rust
    fn restore(snapshot: Snapshot) -> Result<Self> {
        if snapshot.schema_version != SCHEMA_VERSION {
            return Err(ContractError("unsupported snapshot schema".into()));
        }
        validate_organisms(
            &snapshot
```

</details>

<details><summary><code>tick()</code></summary>

Read the fixture logical clock without advancing it.

[Source lines 80–82](C:/Thought/Projects/World/crates/world_core/src/world.rs:80)

```rust
    pub fn tick(&self) -> u64 {
        self.clock.0
    }
```

</details>

<details><summary><code>observe()</code></summary>

Expose current fixed signals, exact IDs/versions, schema, and tick.

[Source lines 84–89](C:/Thought/Projects/World/crates/world_core/src/world.rs:84)

```rust
    pub fn observe(&self) -> ObservationBatch {
        ObservationBatch {
            schema_version: SCHEMA_VERSION,
            tick: self.clock.0,
            ids: self
                .organisms
```

</details>

<details><summary><code>apply_actions()</code></summary>

Validate every proposed action and accumulated total before committing the complete batch and clock.

[Source lines 106–111](C:/Thought/Projects/World/crates/world_core/src/world.rs:106)

```rust
    pub fn apply_actions(&mut self, batch: ActionBatch) -> Result<()> {
        if batch.schema_version != SCHEMA_VERSION || batch.tick != self.clock.0 {
            return Err(ContractError(
                "unsupported action schema or stale tick".into(),
            ));
        }
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Serialize inspected native state; only the diagnostic fixture supports restoration.

[Source lines 157–162](C:/Thought/Projects/World/crates/world_core/src/world.rs:157)

```rust
    pub fn snapshot_json(&self) -> Result<String> {
        serde_json::to_string(&Snapshot {
            schema_version: SCHEMA_VERSION,
            tick: self.clock.0,
            organisms: self.organisms.clone(),
        })
```

</details>

<details><summary><code>from_snapshot_json()</code></summary>

Decode and validate saved diagnostic state.

[Source lines 166–170](C:/Thought/Projects/World/crates/world_core/src/world.rs:166)

```rust
    pub fn from_snapshot_json(text: &str) -> Result<Self> {
        let snapshot = serde_json::from_str(text)
            .map_err(|error| ContractError(format!("invalid fixture snapshot: {error}")))?;
        Self::restore(snapshot)
    }
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.
- `deny_unknown_fields` rejects unexpected input fields instead of silently ignoring them.

</details>

<details><summary><code>crates/world_bindings/src/arrays.rs</code></summary>

**Purpose:** Converts native batches to owned NumPy arrays and validates/copies incoming actions.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/src/arrays.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use numpy::{PyArray1, PyArrayMethods, PyReadonlyArray1, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use world_core::{
    contracts::{ACTION_WIDTH, ActionBatch, OBSERVATION_WIDTH, ObservationBatch},
    ids::OrganismId,
};
```

- `numpy`: typed host arrays and shape/layout/numeric checks.
- `pyo3`: Python-visible classes, methods, properties, and error translation.
- `world_core`: the Python-independent native habitat and error types.

**Standalone functions**

<details><summary><code>observation_dict()</code></summary>

Allocate independently owned NumPy arrays and return them with scalar metadata.

[Source lines 8–13](C:/Thought/Projects/World/crates/world_bindings/src/arrays.rs:8)

```rust
pub fn observation_dict<'py>(
    py: Python<'py>,
    batch: ObservationBatch,
) -> PyResult<Bound<'py, PyDict>> {
    let count = batch.ids.len();
    let values: Vec<f32> = batch.values.into_iter().flatten().collect();
```

</details>

<details><summary><code>action_batch()</code></summary>

Validate native array dtypes/ranks/contiguity/shapes and copy them into Rust-owned values.

[Source lines 32–40](C:/Thought/Projects/World/crates/world_bindings/src/arrays.rs:32)

```rust
pub fn action_batch(
    schema_version: u32,
    tick: u64,
    ids: PyReadonlyArray1<'_, u64>,
    topology_versions: PyReadonlyArray1<'_, u64>,
    values: PyReadonlyArray2<'_, f32>,
) -> PyResult<ActionBatch> {
    if !ids.is_c_contiguous() || !topology_versions.is_c_contiguous() || !values.is_c_contiguous() {
        return Err(PyValueError::new_err("batch arrays must be C-contiguous"));
```

</details>

</details>

<details><summary><code>crates/world_bindings/src/world.rs</code></summary>

**Purpose:** Exposes the diagnostic world as a Python class with native validation.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/src/world.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBool, PyDict, PyInt},
};
use world_core::world::FixtureWorld;
use crate::arrays;
```

- `numpy`: typed host arrays and shape/layout/numeric checks.
- `pyo3`: Python-visible classes, methods, properties, and error translation.
- `world_core`: the Python-independent native habitat and error types.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `PyFixtureWorld`**

A Python-visible object owning the real Rust fixture and forwarding validated calls.

[Source lines 11–14](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:11)

```rust
#[pyclass(name = "FixtureWorld", module = "world._native")]
pub struct PyFixtureWorld {
    inner: FixtureWorld,
}
```

**Methods — open the one you are reading**

<details><summary><code>new()</code></summary>

Construct this record or wrapper and validate its initial input.

[Source lines 19–23](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:19)

```rust
    fn new(config_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: FixtureWorld::from_config_json(config_json).map_err(value_error)?,
        })
    }
```

</details>

<details><summary><code>tick()</code></summary>

Read the fixture logical clock without advancing it.

[Source lines 26–28](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:26)

```rust
    fn tick(&self) -> u64 {
        self.inner.tick()
    }
```

</details>

<details><summary><code>observe()</code></summary>

Expose current fixed signals, exact IDs/versions, schema, and tick.

[Source lines 30–32](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:30)

```rust
    fn observe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        arrays::observation_dict(py, self.inner.observe())
    }
```

</details>

<details><summary><code>apply_actions()</code></summary>

Validate every proposed action and accumulated total before committing the complete batch and clock.

[Source lines 35–44](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:35)

```rust
    fn apply_actions(
        &mut self,
        schema_version: Bound<'_, PyAny>,
        tick: Bound<'_, PyAny>,
        ids: PyReadonlyArray1<'_, u64>,
        topology_versions: PyReadonlyArray1<'_, u64>,
        values: PyReadonlyArray2<'_, f32>,
    ) -> PyResult<()> {
        let schema_version = u32::try_from(unsigned_integer(&schema_version, "schema_version")?)
            .map_err(|_| PyValueError::new_err("schema_version exceeds unsigned 32-bit range"))?;
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Serialize inspected native state; only the diagnostic fixture supports restoration.

[Source lines 57–59](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:57)

```rust
    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(value_error)
    }
```

</details>

<details><summary><code>from_snapshot_json()</code></summary>

Decode and validate saved diagnostic state.

[Source lines 62–66](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:62)

```rust
    fn from_snapshot_json(text: &str) -> PyResult<Self> {
        Ok(Self {
            inner: FixtureWorld::from_snapshot_json(text).map_err(value_error)?,
        })
    }
```

</details>

**Standalone functions**

<details><summary><code>value_error()</code></summary>

Translate native fixture contract failure into Python ValueError.

[Source lines 69–71](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:69)

```rust
fn value_error(error: world_core::contracts::ContractError) -> PyErr {
    PyValueError::new_err(error.to_string())
}
```

</details>

<details><summary><code>unsigned_integer()</code></summary>

Reject nonintegers and Booleans before extracting unsigned metadata.

[Source lines 73–78](C:/Thought/Projects/World/crates/world_bindings/src/world.rs:73)

```rust
fn unsigned_integer(value: &Bound<'_, PyAny>, name: &str) -> PyResult<u64> {
    if !value.is_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err(format!("{name} must be an integer")));
    }
    value.extract()
}
```

</details>

**Useful syntax in this file**

- `#[pyclass]` and `#[pymethods]` expose Rust records and methods to Python.

</details>

<details><summary><code>python/world/config.py</code></summary>

**Purpose:** Reads a fixture configuration and constructs the native world.

[Open the file](C:/Thought/Projects/World/python/world/config.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from pathlib import Path
from ._native import FixtureWorld
```

- `pathlib`: configuration and output paths.
- `._native`: the compiled Rust habitat, aliased to distinguish it from the Python wrapper.

**Standalone functions**

<details><summary><code>load_fixture()</code></summary>

Read fixture JSON; the installed package helper also constructs the native world.

[Source lines 6–7](C:/Thought/Projects/World/python/world/config.py:6)

```python
def load_fixture(path: str | Path) -> FixtureWorld:
    return FixtureWorld(Path(path).read_text(encoding="utf-8"))
```

</details>

</details>

<details><summary><code>python/world/brains/__init__.py</code></summary>

**Purpose:** Marks and describes the controller-support package.

[Open the file](C:/Thought/Projects/World/python/world/brains/__init__.py)

[Source lines 1–1](C:/Thought/Projects/World/python/world/brains/__init__.py:1)

```python
"""Batched controller contracts and runtime state."""
```

These declarations describe the package or expose neighboring modules; no independent runtime algorithm is defined here.

</details>

<details><summary><code>python/world/brains/contracts.py</code></summary>

**Purpose:** Validates Python batches and protects their owned array copies.

[Open the file](C:/Thought/Projects/World/python/world/brains/contracts.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from dataclasses import dataclass
from typing import ClassVar
import numpy as np
```

- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `typing`: type annotations.
- `numpy`: typed host arrays and shape/layout/numeric checks.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 7–7](C:/Thought/Projects/World/python/world/brains/contracts.py:7)

```python
SCHEMA_VERSION = 1
```

[Source lines 8–8](C:/Thought/Projects/World/python/world/brains/contracts.py:8)

```python
UINT64_MAX = 2**64 - 1
```

**Class: `Batch`**

Common Python validation and ownership rules reused by observations and actions.

[Source lines 17–24](C:/Thought/Projects/World/python/world/brains/contracts.py:17)

```python
@dataclass(frozen=True, slots=True)
class Batch:
    tick: int
    ids: np.ndarray
    topology_versions: np.ndarray
    values: np.ndarray
    schema_version: int = SCHEMA_VERSION
    width: ClassVar[int] = 0
```

**Methods — open the one you are reading**

<details><summary><code>__post_init__()</code></summary>

Check metadata, dtypes, ranks, contiguity, shapes, identities, and finite values; copy arrays before exposing them.

[Source lines 26–31](C:/Thought/Projects/World/python/world/brains/contracts.py:26)

```python
    def __post_init__(self):
        uint64(self.tick, "tick")
        if type(self.schema_version) is not int or self.schema_version != SCHEMA_VERSION:
            raise ValueError("unsupported batch schema")
        for name, dtype, rank in [("ids", np.uint64, 1),
                                  ("topology_versions", np.uint64, 1),
```

</details>

<details><summary><code>_validate_values()</code></summary>

Require a subclass-specific value rule instead of accepting an unspecified batch type.

[Source lines 50–51](C:/Thought/Projects/World/python/world/brains/contracts.py:50)

```python
    def _validate_values(self):
        raise NotImplementedError
```

</details>

<details><summary><code>from_native()</code></summary>

Require exact payload keys and construct a validated Python batch.

[Source lines 54–57](C:/Thought/Projects/World/python/world/brains/contracts.py:54)

```python
    def from_native(cls, payload: dict):
        if set(payload) != {"schema_version", "tick", "ids", "topology_versions", "values"}:
            raise ValueError("unexpected batch fields")
        return cls(**payload)
```

</details>

<details><summary><code>to_native()</code></summary>

Return the native binding payload dictionary.

[Source lines 59–61](C:/Thought/Projects/World/python/world/brains/contracts.py:59)

```python
    def to_native(self) -> dict:
        return {"schema_version": self.schema_version, "tick": self.tick, "ids": self.ids,
                "topology_versions": self.topology_versions, "values": self.values}
```

</details>

<details><summary><code>reordered()</code></summary>

Require a complete permutation and reorder all row-associated arrays together.

[Source lines 63–68](C:/Thought/Projects/World/python/world/brains/contracts.py:63)

```python
    def reordered(self, order):
        order = np.asarray(order)
        if len(self.ids) == 0 and order.shape == (0,):
            order = np.empty(0, dtype=np.intp)
        if order.ndim != 1 or order.dtype.kind not in "iu" or not np.array_equal(
                np.sort(order), np.arange(len(self.ids))):
```

</details>

**Class: `ObservationBatch`**

Three observation columns per identity, with schema, tick, and topology metadata.

[Source lines 75–77](C:/Thought/Projects/World/python/world/brains/contracts.py:75)

```python
class ObservationBatch(Batch):
    __slots__ = ()
    width = 3
```

**Methods — open the one you are reading**

<details><summary><code>_validate_values()</code></summary>

Keep all three fixture signals in [0, 1].

[Source lines 79–81](C:/Thought/Projects/World/python/world/brains/contracts.py:79)

```python
    def _validate_values(self):
        if np.any(self.values < 0) or np.any(self.values > 1):
            raise ValueError("fixture observations must be in [0, 1]")
```

</details>

**Class: `ActionBatch`**

Two bounded action columns per identity, with the metadata needed to reject stale routing.

[Source lines 84–86](C:/Thought/Projects/World/python/world/brains/contracts.py:84)

```python
class ActionBatch(Batch):
    __slots__ = ()
    width = 2
```

**Methods — open the one you are reading**

<details><summary><code>_validate_values()</code></summary>

Keep turn in [-1, 1] and thrust in [0, 1].

[Source lines 88–91](C:/Thought/Projects/World/python/world/brains/contracts.py:88)

```python
    def _validate_values(self):
        if np.any(np.abs(self.values[:, 0]) > 1) or np.any(
                (self.values[:, 1] < 0) | (self.values[:, 1] > 1)):
            raise ValueError("actions require turn in [-1, 1] and thrust in [0, 1]")
```

</details>

**Standalone functions**

<details><summary><code>uint64()</code></summary>

Require an actual Python integer in the unsigned 64-bit range.

[Source lines 11–14](C:/Thought/Projects/World/python/world/brains/contracts.py:11)

```python
def uint64(value: int, name: str) -> int:
    if type(value) is not int or not 0 <= value <= UINT64_MAX:
        raise ValueError(f"{name} must be an unsigned 64-bit integer")
    return value
```

</details>

**Useful syntax in this file**

- `@dataclass` builds record constructors; `frozen=True` prevents normal reassignment, and `slots=True` fixes instance fields.
- `@classmethod` passes the class as `cls`, allowing an alternate constructor.
- `ClassVar` marks a class setting such as batch width, rather than another constructor field.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>python/world/brains/registry.py</code></summary>

**Purpose:** Tracks per-ID recurrent diagnostic memory and proposes bounded actions.

[Open the file](C:/Thought/Projects/World/python/world/brains/registry.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import numpy as np
import torch
from .contracts import ActionBatch, ObservationBatch, uint64
```

- `numpy`: typed host arrays and shape/layout/numeric checks.
- `torch`: CPU/CUDA tensors and recurrent diagnostic calculations.
- `.contracts`: the validated batch types and unsigned-metadata checker.

**Class: `ControllerRegistry`**

Per-ID recurrent memory plus proposal and restoration logic; this is a fixed diagnostic calculation.

[Source lines 7–7](C:/Thought/Projects/World/python/world/brains/registry.py:7)

```python
class ControllerRegistry:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Create this object and store the state its other methods need.

[Source lines 8–13](C:/Thought/Projects/World/python/world/brains/registry.py:8)

```python
    def __init__(self, observations: ObservationBatch, memory: torch.Tensor):
        self.tick = observations.tick
        self._ids = observations.ids
        self._versions = observations.topology_versions
        self._rows = {int(identity): row for row, identity in enumerate(self._ids)}
        self._memory = memory
```

</details>

<details><summary><code>initialize()</code></summary>

Select CPU/CUDA and allocate zero per-ID recurrent diagnostic memory.

[Source lines 16–21](C:/Thought/Projects/World/python/world/brains/registry.py:16)

```python
    def initialize(cls, observations: ObservationBatch, device: str = "cpu"):
        device = torch.device(device)
        if device.type not in {"cpu", "cuda"}:
            raise ValueError("supported controller devices are cpu and cuda")
        if device.type == "cuda" and not torch.cuda.is_available():
            raise RuntimeError("CUDA was requested but is unavailable")
```

</details>

<details><summary><code>validate_alignment()</code></summary>

Match ticks, identity membership, and per-ID topology versions.

[Source lines 25–30](C:/Thought/Projects/World/python/world/brains/registry.py:25)

```python
    def validate_alignment(self, observations: ObservationBatch):
        if observations.tick != self.tick or set(map(int, observations.ids)) != set(self._rows):
            raise ValueError("controller cohort or tick does not match the world")
        for identity, version in zip(observations.ids, observations.topology_versions, strict=True):
            if int(version) != int(self._versions[self._rows[int(identity)]]):
                raise ValueError("controller topology version does not match the world")
```

</details>

<details><summary><code>propose()</code></summary>

Compute fixed recurrent actions and a candidate memory state without editing the current registry.

[Source lines 33–38](C:/Thought/Projects/World/python/world/brains/registry.py:33)

```python
    def propose(self, observations: ObservationBatch):
        self.validate_alignment(observations)
        next_tick = uint64(self.tick + 1, "next tick")
        device = self._memory.device
        rows = torch.tensor([self._rows[int(identity)] for identity in observations.ids],
                            dtype=torch.long, device=device)
```

</details>

<details><summary><code>snapshot()</code></summary>

Return independent inspection/checkpoint data; bridge snapshots also validate memory/action consistency.

[Source lines 52–55](C:/Thought/Projects/World/python/world/brains/registry.py:52)

```python
    def snapshot(self) -> dict:
        return {"schema_version": 1, "kind": "diagnostic_recurrent", "tick": self.tick,
                "ids": self._ids.tolist(), "topology_versions": self._versions.tolist(),
                "memory": self._memory.cpu().numpy().tolist()}
```

</details>

<details><summary><code>from_snapshot()</code></summary>

Validate and recreate diagnostic memory on the chosen device.

[Source lines 58–63](C:/Thought/Projects/World/python/world/brains/registry.py:58)

```python
    def from_snapshot(cls, payload: dict, device: str = "cpu"):
        if set(payload) != {"schema_version", "kind", "tick", "ids", "topology_versions", "memory"}:
            raise ValueError("unexpected controller snapshot fields")
        if type(payload["schema_version"]) is not int or payload["schema_version"] != 1 or payload["kind"] != "diagnostic_recurrent":
            raise ValueError("unsupported controller snapshot")
        for field in ("ids", "topology_versions"):
```

</details>

**Useful syntax in this file**

- `@classmethod` passes the class as `cls`, allowing an alternate constructor.
- `@torch.inference_mode()` disables gradient tracking for the fixed diagnostic calculation.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>python/world/runtime.py</code></summary>

**Purpose:** Coordinates native action acceptance, memory commit, and consistent fixture continuation.

[Open the file](C:/Thought/Projects/World/python/world/runtime.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import json
import os
from pathlib import Path
import tempfile
import numpy as np
from ._native import FixtureWorld
from .brains.contracts import ObservationBatch
from .brains.registry import ControllerRegistry
from .config import load_fixture
```

- `json`: conversion between dictionaries and the native JSON interface.
- `os`: replacing a completed checkpoint file.
- `pathlib`: configuration and output paths.
- `tempfile`: temporary files/directories.
- `numpy`: typed host arrays and shape/layout/numeric checks.
- `._native`: the compiled Rust habitat, aliased to distinguish it from the Python wrapper.
- `.brains.contracts`: validated observation batches for the native/controller boundary.
- `.brains.registry`: the controller memory registry used by the diagnostic runtime.
- `.config`: the fixture loader used to construct native diagnostic state.

**Class: `BridgeRuntime`**

Keeps the native world and Python controller memory consistent through acceptance and continuation.

[Source lines 14–14](C:/Thought/Projects/World/python/world/runtime.py:14)

```python
class BridgeRuntime:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Create this object and store the state its other methods need.

[Source lines 15–18](C:/Thought/Projects/World/python/world/runtime.py:15)

```python
    def __init__(self, world: FixtureWorld, registry: ControllerRegistry):
        self.world = world
        self.registry = registry
        self.snapshot()
```

</details>

<details><summary><code>from_config()</code></summary>

Load native fixture state and create aligned zero controller memory.

[Source lines 21–24](C:/Thought/Projects/World/python/world/runtime.py:21)

```python
    def from_config(cls, path: str | Path, device: str = "cpu"):
        world = load_fixture(path)
        observations = ObservationBatch.from_native(world.observe())
        return cls(world, ControllerRegistry.initialize(observations, device))
```

</details>

<details><summary><code>advance()</code></summary>

Observe, propose, submit to Rust, and commit candidate memory only after acceptance.

[Source lines 26–31](C:/Thought/Projects/World/python/world/runtime.py:26)

```python
    def advance(self):
        observations = ObservationBatch.from_native(self.world.observe())
        actions, candidate = self.registry.propose(observations)
        self.world.apply_actions(**actions.to_native())
        self.registry = candidate
        return actions
```

</details>

<details><summary><code>snapshot()</code></summary>

Return independent inspection/checkpoint data; bridge snapshots also validate memory/action consistency.

[Source lines 33–38](C:/Thought/Projects/World/python/world/runtime.py:33)

```python
    def snapshot(self) -> dict:
        observations = ObservationBatch.from_native(self.world.observe())
        self.registry.validate_alignment(observations)
        world = json.loads(self.world.snapshot_json())
        controller = self.registry.snapshot()
        previous = {item["organism"]["id"]: item["last_action"] for item in world["organisms"]}
```

</details>

<details><summary><code>save()</code></summary>

Write consistent JSON through a neighboring temporary file and replace the checkpoint.

[Source lines 46–51](C:/Thought/Projects/World/python/world/runtime.py:46)

```python
    def save(self, path: str | Path):
        text = json.dumps(self.snapshot(), indent=2, allow_nan=False) + "\n"
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = None
        try:
```

</details>

<details><summary><code>load()</code></summary>

Validate and restore combined fixture/controller state.

[Source lines 62–67](C:/Thought/Projects/World/python/world/runtime.py:62)

```python
    def load(cls, path: str | Path, device: str = "cpu"):
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
        if not isinstance(payload, dict) or set(payload) != {"schema_version", "world", "controller"}:
            raise ValueError("unexpected fixture checkpoint fields")
        if type(payload["schema_version"]) is not int or payload["schema_version"] != 1:
            raise ValueError("unsupported fixture checkpoint schema")
```

</details>

**Useful syntax in this file**

- `@classmethod` passes the class as `cls`, allowing an alternate constructor.
- `finally` always runs cleanup, even when the operation fails.
- `allow_nan=False` prevents nonfinite numbers from being encoded as JSON.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>examples/bridge_demo.py</code></summary>

**Purpose:** Runs the bridge on CPU/CUDA with optional checkpoint save/resume.

[Open the file](C:/Thought/Projects/World/examples/bridge_demo.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
from pathlib import Path
from world.runtime import BridgeRuntime
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `pathlib`: configuration and output paths.
- `world.runtime`: the installed public Habitat interface used by examples and tests.

**Standalone functions**

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 7–12](C:/Thought/Projects/World/examples/bridge_demo.py:7)

```python
def main():
    parser = argparse.ArgumentParser(description="Run the diagnostic batched Rust/PyTorch fixture.")
    parser.add_argument("--config", type=Path, default=Path(__file__).resolve().parents[1] / "configs/bridge_fixture.json")
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cpu")
    parser.add_argument("--ticks", type=int, default=5)
    parser.add_argument("--checkpoint", type=Path)
```

</details>

</details>

<details><summary><code>benchmarks/bridge.py</code></summary>

**Purpose:** Measures completed diagnostic exchanges after warmup; it does not measure ecosystem capacity.

[Open the file](C:/Thought/Projects/World/benchmarks/bridge.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
import json
import time
from world import FixtureWorld
from world.brains.contracts import ObservationBatch
from world.brains.registry import ControllerRegistry
from world.runtime import BridgeRuntime
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `json`: conversion between dictionaries and the native JSON interface.
- `time`: the project component used by this file.
- `world`: the installed public Habitat interface used by examples and tests.
- `world.brains.contracts`: the installed public Habitat interface used by examples and tests.
- `world.brains.registry`: the installed public Habitat interface used by examples and tests.
- `world.runtime`: the installed public Habitat interface used by examples and tests.

**Standalone functions**

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 11–16](C:/Thought/Projects/World/benchmarks/bridge.py:11)

```python
def main():
    parser = argparse.ArgumentParser(description="Measure completed diagnostic batch exchanges.")
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cpu")
    parser.add_argument("--population", type=int, default=128)
    parser.add_argument("--ticks", type=int, default=200)
    args = parser.parse_args()
```

</details>

</details>

<details><summary><code>docs/data_formats.md</code></summary>

**Purpose:** The reference for batch fields, array ownership, validation, and fixture checkpoints.

[Open the file](C:/Thought/Projects/World/docs/data_formats.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

<details><summary><code>tests/test_bridge_contract.py</code></summary>

**Purpose:** Checks routing, strict arrays, rollback, recurrent memory, continuation, and CPU/CUDA agreement.

[Open the file](C:/Thought/Projects/World/tests/test_bridge_contract.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest
import numpy as np
import torch
from world import FixtureWorld
from world.brains.contracts import ActionBatch, ObservationBatch
from world.brains.registry import ControllerRegistry
from world.runtime import BridgeRuntime
```

- `copy`: defensive nested-data copying.
- `json`: conversion between dictionaries and the native JSON interface.
- `pathlib`: configuration and output paths.
- `tempfile`: temporary files/directories.
- `unittest`: fresh test cases, assertions, and expected exceptions.
- `numpy`: typed host arrays and shape/layout/numeric checks.
- `torch`: CPU/CUDA tensors and recurrent diagnostic calculations.
- `world`: the installed public Habitat interface used by examples and tests.
- `world.brains.contracts`: the installed public Habitat interface used by examples and tests.
- `world.brains.registry`: the installed public Habitat interface used by examples and tests.
- `world.runtime`: the installed public Habitat interface used by examples and tests.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 16–16](C:/Thought/Projects/World/tests/test_bridge_contract.py:16)

```python
CONFIG = Path(__file__).resolve().parents[1] / "configs/bridge_fixture.json"
```

**Class: `BridgeContractTests`**

A test group that constructs fresh fixtures and checks related behavior.

[Source lines 28–28](C:/Thought/Projects/World/tests/test_bridge_contract.py:28)

```python
class BridgeContractTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>test_owned_observations_keep_integer_identity_and_do_not_alias_world()</code></summary>

Checks owned observations keep integer identity and do not alias world.

[Source lines 29–34](C:/Thought/Projects/World/tests/test_bridge_contract.py:29)

```python
    def test_owned_observations_keep_integer_identity_and_do_not_alias_world(self):
        world = fixture()
        before = world.snapshot_json()
        batch = world.observe()
        self.assertEqual(batch["ids"].dtype, np.uint64)
        self.assertEqual(int(batch["ids"][-1]), 2**53 + 1)
```

</details>

<details><summary><code>test_reordered_action_rows_reach_correct_organisms()</code></summary>

Checks reordered action rows reach correct organisms.

[Source lines 43–48](C:/Thought/Projects/World/tests/test_bridge_contract.py:43)

```python
    def test_reordered_action_rows_reach_correct_organisms(self):
        world = fixture()
        payload = action_payload(world)
        actions = ActionBatch.from_native(payload).reordered([2, 0, 1])
        world.apply_actions(**actions.to_native())
        snapshot = json.loads(world.snapshot_json())
```

</details>

<details><summary><code>test_rejected_batches_preserve_all_native_state()</code></summary>

Checks rejected batches preserve all native state.

[Source lines 53–58](C:/Thought/Projects/World/tests/test_bridge_contract.py:53)

```python
    def test_rejected_batches_preserve_all_native_state(self):
        world = fixture()
        good = action_payload(world)
        cases = []
        for field, value in [("tick", 1), ("schema_version", 2), ("tick", False), ("schema_version", True)]:
            bad = deepcopy(good); bad[field] = value; cases.append(bad)
```

</details>

<details><summary><code>test_dtype_shape_and_stride_errors_do_not_coerce_data()</code></summary>

Checks dtype shape and stride errors do not coerce data.

[Source lines 78–83](C:/Thought/Projects/World/tests/test_bridge_contract.py:78)

```python
    def test_dtype_shape_and_stride_errors_do_not_coerce_data(self):
        world = fixture()
        good = action_payload(world)
        for field, array in [
            ("ids", good["ids"].astype(np.float64)),
            ("ids", good["ids"].astype(">u8")),
```

</details>

<details><summary><code>test_python_batches_copy_inputs_and_validate_the_same_contract()</code></summary>

Checks python batches copy inputs and validate the same contract.

[Source lines 98–103](C:/Thought/Projects/World/tests/test_bridge_contract.py:98)

```python
    def test_python_batches_copy_inputs_and_validate_the_same_contract(self):
        payload = action_payload(fixture())
        batch = ActionBatch.from_native(payload)
        payload["values"][:] = 0
        self.assertEqual(batch.values[0, 0], 0.5)
        self.assertFalse(batch.values.flags.writeable)
```

</details>

<details><summary><code>test_recurrent_controller_matches_independent_scalar_calculation()</code></summary>

Checks recurrent controller matches independent scalar calculation.

[Source lines 111–116](C:/Thought/Projects/World/tests/test_bridge_contract.py:111)

```python
    def test_recurrent_controller_matches_independent_scalar_calculation(self):
        runtime = BridgeRuntime.from_config(CONFIG)
        observations = runtime.world.observe()
        memory = np.zeros((3, 2), dtype=np.float32)
        total = np.zeros((3, 2), dtype=np.float64)
        signals = observations["values"]
```

</details>

<details><summary><code>test_registry_matches_memory_by_id_after_observation_reordering()</code></summary>

Checks registry matches memory by id after observation reordering.

[Source lines 130–135](C:/Thought/Projects/World/tests/test_bridge_contract.py:130)

```python
    def test_registry_matches_memory_by_id_after_observation_reordering(self):
        observations = ObservationBatch.from_native(fixture().observe())
        first = ControllerRegistry.initialize(observations)
        _, registry = first.propose(observations)
        following = ObservationBatch(1, observations.ids, observations.topology_versions, observations.values)
        direct, direct_registry = registry.propose(following)
```

</details>

<details><summary><code>test_registry_rejects_stale_metadata_without_changing_memory()</code></summary>

Checks registry rejects stale metadata without changing memory.

[Source lines 142–147](C:/Thought/Projects/World/tests/test_bridge_contract.py:142)

```python
    def test_registry_rejects_stale_metadata_without_changing_memory(self):
        observations = ObservationBatch.from_native(fixture().observe())
        registry = ControllerRegistry.initialize(observations)
        before = registry.snapshot()
        for field, value in [("tick", 1), ("ids", np.array([17, 42, 99], dtype=np.uint64)),
                             ("topology_versions", np.ones(3, dtype=np.uint64))]:
```

</details>

<details><summary><code>test_native_rejection_does_not_commit_proposed_controller_memory()</code></summary>

Checks native rejection does not commit proposed controller memory.

[Source lines 152–156](C:/Thought/Projects/World/tests/test_bridge_contract.py:152)

```python
    def test_native_rejection_does_not_commit_proposed_controller_memory(self):
        class RejectOnce:
            def __init__(self):
                self.inner = fixture()
                self.reject = True
```

</details>

<details><summary><code>test_cpu_checkpoint_continuation_matches_uninterrupted_execution()</code></summary>

Checks cpu checkpoint continuation matches uninterrupted execution.

[Source lines 182–187](C:/Thought/Projects/World/tests/test_bridge_contract.py:182)

```python
    def test_cpu_checkpoint_continuation_matches_uninterrupted_execution(self):
        control = BridgeRuntime.from_config(CONFIG)
        split = BridgeRuntime.from_config(CONFIG)
        for _ in range(2):
            control.advance(); split.advance()
        with tempfile.TemporaryDirectory() as directory:
```

</details>

<details><summary><code>test_inconsistent_or_malformed_checkpoints_are_rejected()</code></summary>

Checks inconsistent or malformed checkpoints are rejected.

[Source lines 195–200](C:/Thought/Projects/World/tests/test_bridge_contract.py:195)

```python
    def test_inconsistent_or_malformed_checkpoints_are_rejected(self):
        runtime = BridgeRuntime.from_config(CONFIG)
        runtime.advance()
        original = runtime.snapshot()
        cases = []
        for field, value in [("tick", 0), ("ids", [17, 42, 99]),
```

</details>

<details><summary><code>test_empty_cohort_advances_and_resumes()</code></summary>

Checks empty cohort advances and resumes.

[Source lines 215–220](C:/Thought/Projects/World/tests/test_bridge_contract.py:215)

```python
    def test_empty_cohort_advances_and_resumes(self):
        world = FixtureWorld('{"schema_version":1,"organisms":[]}')
        observations = ObservationBatch.from_native(world.observe())
        self.assertEqual(observations.values.shape, (0, 3))
        runtime = BridgeRuntime(world, ControllerRegistry.initialize(observations))
        self.assertEqual(runtime.advance().values.shape, (0, 2))
```

</details>

<details><summary><code>test_invalid_fixture_configs_and_snapshots_are_rejected()</code></summary>

Checks invalid fixture configs and snapshots are rejected.

[Source lines 228–233](C:/Thought/Projects/World/tests/test_bridge_contract.py:228)

```python
    def test_invalid_fixture_configs_and_snapshots_are_rejected(self):
        config = json.loads(CONFIG.read_text())
        for update in [lambda item: item.update(schema_version=2),
                       lambda item: item["organisms"].append(deepcopy(item["organisms"][0])),
                       lambda item: item["organisms"][0].update(signals=[0, 0, 2]),
                       lambda item: item.update(extra=0)]:
```

</details>

<details><summary><code>test_tick_overflow_does_not_wrap()</code></summary>

Checks tick overflow does not wrap.

[Source lines 240–245](C:/Thought/Projects/World/tests/test_bridge_contract.py:240)

```python
    def test_tick_overflow_does_not_wrap(self):
        snapshot = json.loads(fixture().snapshot_json())
        snapshot["tick"] = 2**64 - 1
        world = FixtureWorld.from_snapshot_json(json.dumps(snapshot))
        before = world.snapshot_json()
        with self.assertRaises(ValueError):
```

</details>

<details><summary><code>test_cuda_and_cpu_produce_matching_fixture_trajectories()</code></summary>

Checks cuda and cpu produce matching fixture trajectories.

[Source lines 250–255](C:/Thought/Projects/World/tests/test_bridge_contract.py:250)

```python
    def test_cuda_and_cpu_produce_matching_fixture_trajectories(self):
        cpu = BridgeRuntime.from_config(CONFIG, "cpu")
        cuda = BridgeRuntime.from_config(CONFIG, "cuda")
        for _ in range(6):
            np.testing.assert_allclose(cpu.advance().values, cuda.advance().values, rtol=1e-6, atol=1e-7)
        np.testing.assert_allclose(cpu.registry.snapshot()["memory"], cuda.registry.snapshot()["memory"],
```

</details>

**Class: `RejectOnce`**

A test-only wrapper that deliberately submits a stale tick, proving native rejection leaves proposed controller memory uncommitted.

[Source lines 153–153](C:/Thought/Projects/World/tests/test_bridge_contract.py:153)

```python
        class RejectOnce:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Create this object and store the state its other methods need.

[Source lines 154–156](C:/Thought/Projects/World/tests/test_bridge_contract.py:154)

```python
            def __init__(self):
                self.inner = fixture()
                self.reject = True
```

</details>

<details><summary><code>observe()</code></summary>

Expose current fixed signals, exact IDs/versions, schema, and tick.

[Source lines 158–159](C:/Thought/Projects/World/tests/test_bridge_contract.py:158)

```python
            def observe(self):
                return self.inner.observe()
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Serialize inspected native state; only the diagnostic fixture supports restoration.

[Source lines 161–162](C:/Thought/Projects/World/tests/test_bridge_contract.py:161)

```python
            def snapshot_json(self):
                return self.inner.snapshot_json()
```

</details>

<details><summary><code>apply_actions()</code></summary>

Validate every proposed action and accumulated total before committing the complete batch and clock.

[Source lines 164–167](C:/Thought/Projects/World/tests/test_bridge_contract.py:164)

```python
            def apply_actions(self, **payload):
                if self.reject:
                    payload["tick"] += 1
                self.inner.apply_actions(**payload)
```

</details>

**Standalone functions**

<details><summary><code>fixture()</code></summary>

Construct a fresh diagnostic native fixture for an isolated test.

[Source lines 19–20](C:/Thought/Projects/World/tests/test_bridge_contract.py:19)

```python
def fixture():
    return FixtureWorld(CONFIG.read_text(encoding="utf-8"))
```

</details>

<details><summary><code>action_payload()</code></summary>

Build known actions paired with the current native metadata.

[Source lines 23–25](C:/Thought/Projects/World/tests/test_bridge_contract.py:23)

```python
def action_payload(world):
    observations = world.observe()
    return {**observations, "values": np.array([[0.5, 0.25], [-0.75, 0.5], [0, 1]], dtype=np.float32)}
```

</details>

**Useful syntax in this file**

- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>crates/world_core/tests/bridge_contract.rs</code></summary>

**Purpose:** Checks the native batch contract without going through Python.

[Open the file](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use world_core::{contracts::ActionBatch, ids::OrganismId, world::FixtureWorld};
```

- `world_core`: the Python-independent native habitat and error types.

**Standalone functions**

<details><summary><code>actions()</code></summary>

Construct the known native action batch used by direct Rust tests.

[Source lines 5–10](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs:5)

```rust
fn actions(world: &FixtureWorld) -> ActionBatch {
    let observations = world.observe();
    ActionBatch {
        schema_version: observations.schema_version,
        tick: observations.tick,
        ids: observations.ids,
```

</details>

<details><summary><code>reordered_actions_are_routed_by_full_width_identity()</code></summary>

Checks reordered actions are routed by full width identity.

[Source lines 17–22](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs:17)

```rust
fn reordered_actions_are_routed_by_full_width_identity() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    let mut batch = actions(&world);
    batch.ids.reverse();
    batch.topology_versions.reverse();
    batch.values.reverse();
```

</details>

<details><summary><code>invalid_batches_never_partially_commit()</code></summary>

Checks invalid batches never partially commit.

[Source lines 36–41](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs:36)

```rust
fn invalid_batches_never_partially_commit() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    let good = actions(&world);
    let mut invalid = Vec::new();
    let mut batch = good.clone();
    batch.ids[2] = batch.ids[0];
```

</details>

<details><summary><code>invalid_configs_are_rejected()</code></summary>

Checks invalid configs are rejected.

[Source lines 85–90](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs:85)

```rust
fn invalid_configs_are_rejected() {
    for text in [
        r#"{"schema_version":2,"organisms":[]}"#,
        r#"{"schema_version":1,"organisms":[],"extra":0}"#,
        r#"{"schema_version":1,"organisms":[{"id":1,"topology_version":0,"signals":[0,0,2]}]}"#,
        r#"{"schema_version":1,"organisms":[{"id":1,"topology_version":0,"signals":[0,0,0]},{"id":1,"topology_version":0,"signals":[0,0,0]}]}"#,
```

</details>

<details><summary><code>snapshots_preserve_state_and_clock_overflow_is_atomic()</code></summary>

Checks snapshots preserve state and clock overflow is atomic.

[Source lines 97–102](C:/Thought/Projects/World/crates/world_core/tests/bridge_contract.rs:97)

```rust
fn snapshots_preserve_state_and_clock_overflow_is_atomic() {
    let mut world = FixtureWorld::from_config_json(CONFIG).unwrap();
    world.apply_actions(actions(&world)).unwrap();
    let saved = world.snapshot_json().unwrap();
    let restored = FixtureWorld::from_snapshot_json(&saved).unwrap();
    assert_eq!(restored.snapshot_json().unwrap(), saved);
```

</details>

**Useful syntax in this file**

- `#[test]` registers a Rust test function; assertions make the test fail on a mismatch.

</details>

## Step 4: Native resource accounting and lifecycle

<details><summary><code>crates/world_genetics/Cargo.toml</code></summary>

**Purpose:** Defines the library for inherited organism blueprints.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/Cargo.toml)

[Source lines 1–6](C:/Thought/Projects/World/crates/world_genetics/Cargo.toml:1)

```json
[package]
name = "world_genetics"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
```

Section headers group settings; `key = value` assigns them. `.workspace = true` inherits the shared root setting. These values configure build tools.

</details>

<details><summary><code>crates/world_genetics/src/lib.rs</code></summary>

**Purpose:** Exposes the genome and inheritance modules.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/lib.rs)

[Source lines 1–5](C:/Thought/Projects/World/crates/world_genetics/src/lib.rs:1)

```rust
pub mod body;
pub mod genome;
pub mod inheritance;
pub mod mutation;
pub mod neural;
```

These declarations expose modules or describe the package; no independent runtime algorithm is defined here.

**Useful syntax in this file**


</details>

<details><summary><code>crates/world_genetics/src/genome.rs</code></summary>

**Purpose:** Stores inherited feeding capabilities and finite encoded weights behind read-only accessors.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs)

**Imports / uses and why**

```rust
use crate::{body::BodyGenome, neural::NeuralGenome};
use serde::{Deserialize, Serialize};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.

**Struct: `Genome`**

Handles Genome at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 6–11](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:6)

```rust
pub struct Genome {
    photosynthetic: bool,
    can_ingest: bool,
    initial_weights: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyGenome>,
```

**Methods in `impl Genome`**

<details><summary><code>new()</code></summary>

Construct this record or wrapper and validate its initial input.

[Source lines 17–22](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:17)

```rust
    pub fn new(
        photosynthetic: bool,
        can_ingest: bool,
        initial_weights: Vec<f64>,
    ) -> Result<Self, &'static str> {
        let genome = Self {
```

</details>

<details><summary><code>validate()</code></summary>

Reject invalid values or inconsistent settings before this record is accepted.

[Source lines 33–38](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:33)

```rust
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.initial_weights.iter().all(|value| value.is_finite()) {
            return Err("inherited weights must be finite");
        }
        if let Some(body) = &self.body {
            body.validate()?;
```

</details>

<details><summary><code>photosynthetic()</code></summary>

Read inherited light-capture capability without editing the blueprint.

[Source lines 69–74](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:69)

```rust
    pub fn photosynthetic(&self) -> bool {
        self.body.as_ref().map_or(self.photosynthetic, |b| {
            if let Some(circle) = &b.circle {
                return circle.photosynthetic;
            }
            b.modules
```

</details>

<details><summary><code>refresh_capabilities()</code></summary>

Updates encoded capability flags from the mutated body modules before validation/serialization.

[Source lines 79–84](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:79)

```rust
    pub fn refresh_capabilities(&mut self) {
        if let Some(body) = &self.body {
            if let Some(circle) = &body.circle {
                self.photosynthetic = circle.photosynthetic;
                self.can_ingest = circle.can_ingest;
                return;
```

</details>

<details><summary><code>can_ingest()</code></summary>

Read inherited ingestion capability without editing the blueprint.

[Source lines 96–101](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:96)

```rust
    pub fn can_ingest(&self) -> bool {
        self.body.as_ref().map_or(self.can_ingest, |b| {
            if let Some(circle) = &b.circle {
                return circle.can_ingest;
            }
            b.modules
```

</details>

<details><summary><code>initial_weights()</code></summary>

Borrow the finite encoded weights for reading; no learning or execution occurs.

[Source lines 106–109](C:/Thought/Projects/World/crates/world_genetics/src/genome.rs:106)

```rust
    pub fn initial_weights(&self) -> &[f64] {
        &self.initial_weights
    }
}
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_genetics/src/inheritance.rs</code></summary>

**Purpose:** Copies the parent blueprint into a separate offspring blueprint without mutation.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/inheritance.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use crate::genome::Genome;
```

- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Standalone functions**

<details><summary><code>inherit()</code></summary>

Independently clone the inherited blueprint without mutation.

[Source lines 3–5](C:/Thought/Projects/World/crates/world_genetics/src/inheritance.rs:3)

```rust
pub fn inherit(parent: &Genome) -> Genome {
    parent.clone()
}
```

</details>

</details>

<details><summary><code>crates/world_core/src/resources.rs</code></summary>

**Purpose:** Represents nutrient/energy amounts and prevents invalid arithmetic or overspending.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/resources.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use std::fmt;
use serde::{Deserialize, Serialize};
```

- `std`: standard-library maps/sets for exact identities, lookup, and duplicate detection.
- `serde`: typed configuration input and inspectable structured output.

**Structure / enum: `AccountingError`**

Separates malformed/disallowed requests from failures of whole-ledger invariants.

[Source lines 5–9](C:/Thought/Projects/World/crates/world_core/src/resources.rs:5)

```rust
#[derive(Debug)]
pub enum AccountingError {
    Invalid(String),
    Invariant(String),
}
```

**Methods — open the one you are reading**

<details><summary><code>fmt()</code></summary>

Format the stored error message for display.

[Source lines 12–16](C:/Thought/Projects/World/crates/world_core/src/resources.rs:12)

```rust
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Invariant(message) => message.fmt(formatter),
        }
    }
```

</details>

**Structure / enum: `Resources`**

A nutrient/energy pair used for tissue, reserves, and detritus; checked arithmetic prevents overspending.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/src/resources.rs:32)

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub nutrient: f64,
    pub energy: f64,
}
```

**Methods — open the one you are reading**

<details><summary><code>new()</code></summary>

Construct this record or wrapper and validate its initial input.

[Source lines 40–44](C:/Thought/Projects/World/crates/world_core/src/resources.rs:40)

```rust
    pub fn new(nutrient: f64, energy: f64) -> Result<Self> {
        let resources = Self { nutrient, energy };
        resources.validate()?;
        Ok(resources)
    }
```

</details>

<details><summary><code>validate()</code></summary>

Reject invalid values or inconsistent settings before this record is accepted.

[Source lines 46–49](C:/Thought/Projects/World/crates/world_core/src/resources.rs:46)

```rust
    pub fn validate(self) -> Result<()> {
        finite(self.nutrient, "nutrient", false)?;
        finite(self.energy, "energy", false)
    }
```

</details>

<details><summary><code>plus()</code></summary>

Return validated resource sums.

[Source lines 51–55](C:/Thought/Projects/World/crates/world_core/src/resources.rs:51)

```rust
    pub fn plus(self, other: Self) -> Result<Self> {
        self.validate()?;
        other.validate()?;
        Self::new(self.nutrient + other.nutrient, self.energy + other.energy)
    }
```

</details>

<details><summary><code>minus()</code></summary>

Reject overdrawing either compartment and return the remainder.

[Source lines 57–62](C:/Thought/Projects/World/crates/world_core/src/resources.rs:57)

```rust
    pub fn minus(self, other: Self) -> Result<Self> {
        self.validate()?;
        other.validate()?;
        if other.nutrient > self.nutrient || other.energy > self.energy {
            return Err(AccountingError::Invalid("insufficient resources".into()));
        }
```

</details>

**Standalone functions**

<details><summary><code>finite()</code></summary>

Require finite nonnegative values, or strictly positive values when requested.

[Source lines 22–27](C:/Thought/Projects/World/crates/world_core/src/resources.rs:22)

```rust
pub fn finite(value: f64, name: &str, positive: bool) -> Result<()> {
    if !value.is_finite() || value < 0.0 || (positive && value == 0.0) {
        return Err(AccountingError::Invalid(format!(
            "{name} must be finite and {}",
            if positive { "positive" } else { "nonnegative" }
        )));
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.
- `deny_unknown_fields` rejects unexpected input fields instead of silently ignoring them.

</details>

<details><summary><code>crates/world_core/src/rules.rs</code></summary>

**Purpose:** Loads and validates the shared biological coefficients in Rust.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/rules.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use serde::{Deserialize, Serialize};
use crate::resources::{AccountingError, Result, finite};
```

- `serde`: typed configuration input and inspectable structured output.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `WorldRules`**

One validated rule record shared by all operations, so coefficients stay consistent.

[Source lines 5–12](C:/Thought/Projects/World/crates/world_core/src/rules.rs:5)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldRules {
    pub schema_version: u32,
    pub model_id: String,
    pub world_width: f64,
    pub world_height: f64,
    pub grid_width: u64,
```

**Methods — open the one you are reading**

<details><summary><code>from_json()</code></summary>

Decode typed JSON and run schema and record validation.

[Source lines 46–51](C:/Thought/Projects/World/crates/world_core/src/rules.rs:46)

```rust
    pub fn from_json(text: &str) -> Result<Self> {
        let rules: Self = serde_json::from_str(text)
            .map_err(|error| AccountingError::Invalid(format!("invalid rules: {error}")))?;
        rules.validate()?;
        Ok(rules)
    }
```

</details>

<details><summary><code>validate()</code></summary>

Reject invalid values or inconsistent settings before this record is accepted.

[Source lines 53–58](C:/Thought/Projects/World/crates/world_core/src/rules.rs:53)

```rust
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.model_id.trim().is_empty() {
            return Err(AccountingError::Invalid(
                "unsupported rules schema or empty model ID".into(),
            ));
        }
```

</details>

<details><summary><code>close()</code></summary>

Compare finite values using the larger of absolute and relative numerical tolerance.

[Source lines 188–193](C:/Thought/Projects/World/crates/world_core/src/rules.rs:188)

```rust
    pub fn close(&self, a: f64, b: f64) -> bool {
        a.is_finite()
            && b.is_finite()
            && (a - b).abs()
                <= self
                    .absolute_tolerance
```

</details>

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.
- `deny_unknown_fields` rejects unexpected input fields instead of silently ignoring them.

</details>

<details><summary><code>crates/world_core/src/organism.rs</code></summary>

**Purpose:** Stores an organism's resources, age, ancestry, reproduction cooldown, and death state.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/organism.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use serde::{Deserialize, Serialize};
use world_genetics::genome::Genome;
use crate::{
    ids::OrganismId,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
};
```

- `serde` defines typed data serialization; strict records reject unsupported fields.
- `world_genetics` owns inherited body/neural genes and deterministic mutation operators.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.

**Structure / enum: `DeathCause`**

The allowed starvation, consumption, and damage classifications.

[Source lines 10–16](C:/Thought/Projects/World/crates/world_core/src/organism.rs:10)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    Starvation,
    Consumption,
    Damage,
}
```

**Structure / enum: `Organism`**

The changing individual state, kept separate from its inherited blueprint.

[Source lines 22–29](C:/Thought/Projects/World/crates/world_core/src/organism.rs:22)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organism {
    pub id: OrganismId,
    pub genome: Genome,
    pub tissue: Resources,
    pub reserves: Resources,
    #[serde(default)]
```

**Methods — open the one you are reading**

<details><summary><code>reserve_capacity()</code></summary>

Derives reserve limits from funded tissue and functional invested storage, with overflow handled by the existing conservative transfer rules.

[Source lines 44–50](C:/Thought/Projects/World/crates/world_core/src/organism.rs:44)

```rust
    pub fn reserve_capacity(&self, rules: &WorldRules) -> Result<Resources> {
        crate::anatomy::reserve_capacity(
            self.tissue.nutrient,
            self.genome.body.as_ref(),
            self.body.as_ref(),
            rules,
        )
```

</details>

<details><summary><code>validate()</code></summary>

Reject invalid values or inconsistent settings before this record is accepted.

[Source lines 52–58](C:/Thought/Projects/World/crates/world_core/src/organism.rs:52)

```rust
    pub fn validate(&self) -> Result<()> {
        self.genome
            .validate()
            .map_err(|message| AccountingError::Invalid(message.into()))?;
        self.tissue.validate()?;
        self.reserves.validate()?;
        match (&self.genome.body, &self.body) {
```

</details>

<details><summary><code>mature()</code></summary>

Require living state, minimum individual age, and minimum tissue for reproduction.

[Source lines 71–77](C:/Thought/Projects/World/crates/world_core/src/organism.rs:71)

```rust
    pub fn mature(&self, rules: &WorldRules) -> bool {
        self.alive
            && self.age >= rules.minimum_reproduction_age
            && self.tissue.nutrient
                >= rules.maturity_tissue_nutrient.max(
                    self.genome
                        .body
```

</details>

**Standalone functions**

<details><summary><code>alive()</code></summary>

Supply true as the default living state when decoding a record.

[Source lines 18–20](C:/Thought/Projects/World/crates/world_core/src/organism.rs:18)

```rust
fn alive() -> bool {
    true
}
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `#[derive]` generates ordinary cloning/debug/serialization implementations.
- `skip_serializing_if` keeps absent extension state out of legacy JSON formats.
- `Option<T>` represents an absent/present value; `Result<T>` carries success/error.

</details>

<details><summary><code>crates/world_core/src/events.rs</code></summary>

**Purpose:** Records which inventories and pools changed during an accepted operation.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/events.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use serde::Serialize;
use crate::{
    ids::OrganismId,
    organism::{DeathCause, Organism},
    resources::Resources,
    transactions::{Operation, Pools},
};
```

- `serde`: typed configuration input and inspectable structured output.
- `crate`: the existing local resource model, validation, scheduling, and audit helpers.

**Structure / enum: `OrganismInventory`**

Only the changing resource/lifecycle fields needed in a receipt; genomes are not repeated in every event.

[Source lines 10–17](C:/Thought/Projects/World/crates/world_core/src/events.rs:10)

```rust
#[derive(Clone, Debug, Serialize)]
pub struct OrganismInventory {
    pub tissue: Resources,
    pub reserves: Resources,
    pub age: f64,
    pub alive: bool,
    pub parent_id: Option<OrganismId>,
    pub next_reproduction_age: f64,
```

**Methods — open the one you are reading**

<details><summary><code>from()</code></summary>

Extract changing organism resources and lifecycle fields for a receipt.

[Source lines 22–27](C:/Thought/Projects/World/crates/world_core/src/events.rs:22)

```rust
    fn from(value: &Organism) -> Self {
        Self {
            tissue: value.tissue,
            reserves: value.reserves,
            age: value.age,
            alive: value.alive,
```

</details>

**Structure / enum: `OrganismChange`**

One identity with a before inventory and an after inventory; a newborn has no before inventory.

[Source lines 35–40](C:/Thought/Projects/World/crates/world_core/src/events.rs:35)

```rust
#[derive(Clone, Debug, Serialize)]
pub struct OrganismChange {
    pub id: OrganismId,
    pub before: Option<OrganismInventory>,
    pub after: OrganismInventory,
}
```

**Structure / enum: `ResourceEvent`**

An accepted request with before/after pools, affected organisms, and conservation residuals.

[Source lines 42–49](C:/Thought/Projects/World/crates/world_core/src/events.rs:42)

```rust
#[derive(Clone, Debug, Serialize)]
pub struct ResourceEvent {
    pub sequence: u64,
    pub operation: Operation,
    pub pools_before: Pools,
    pub pools_after: Pools,
    pub organisms: Vec<OrganismChange>,
    pub nutrient_error: f64,
```

**Useful syntax in this file**

- `#[derive(...)]` generates listed traits such as cloning, comparison, defaults, and serialization.

</details>

<details><summary><code>crates/world_core/src/audit.rs</code></summary>

**Purpose:** Checks resource conservation and valid organism inventories using careful floating-point sums.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/audit.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use serde::Serialize;
use crate::{
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    transactions::Pools,
};
```

- `serde` defines typed data serialization; strict records reject unsupported fields.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.

**Structure / enum: `AuditReport`**

Nutrient and energy totals plus their differences from declared inputs.

[Source lines 11–17](C:/Thought/Projects/World/crates/world_core/src/audit.rs:11)

```rust
#[derive(Clone, Copy, Debug, Serialize)]
pub struct AuditReport {
    pub nutrient: f64,
    pub energy: f64,
    pub nutrient_error: f64,
    pub energy_error: f64,
}
```

**Structure / enum: `Totals`**

Internal audit results plus living amounts and count, used when building summaries.

[Source lines 19–24](C:/Thought/Projects/World/crates/world_core/src/audit.rs:19)

```rust
pub(crate) struct Totals {
    pub audit: AuditReport,
    living_nutrient: f64,
    living_energy: f64,
    living_count: usize,
}
```

**Structure / enum: `Sum`**

A running sum with a rounding correction, reducing floating-point accumulation error.

[Source lines 26–30](C:/Thought/Projects/World/crates/world_core/src/audit.rs:26)

```rust
#[derive(Default)]
pub(crate) struct Sum {
    total: f64,
    correction: f64,
}
```

**Methods — open the one you are reading**

<details><summary><code>add()</code></summary>

Add using compensated floating-point summation.

[Source lines 33–39](C:/Thought/Projects/World/crates/world_core/src/audit.rs:33)

```rust
    pub(crate) fn add(&mut self, value: f64) {
        let updated = self.total + value;
        self.correction += if self.total.abs() >= value.abs() {
            (self.total - updated) + value
        } else {
            (value - updated) + self.total
        };
```

</details>

<details><summary><code>value()</code></summary>

Return the sum plus its rounding correction.

[Source lines 42–44](C:/Thought/Projects/World/crates/world_core/src/audit.rs:42)

```rust
    pub(crate) fn value(&self) -> f64 {
        self.total + self.correction
    }
```

</details>

**Standalone functions**

<details><summary><code>audit_view()</code></summary>

Audit committed or staged records: finite pools, identity, tissue energy, viability, capacities, and budgets.

[Source lines 47–53](C:/Thought/Projects/World/crates/world_core/src/audit.rs:47)

```rust
pub(crate) fn audit_view<'a>(
    rules: &WorldRules,
    pools: Pools,
    initial_nutrient: f64,
    initial_energy: f64,
    organisms: impl Iterator<Item = (&'a OrganismId, &'a Organism)>,
) -> Result<Totals> {
```

</details>

<details><summary><code>summarize()</code></summary>

Build aggregate JSON from audited totals and current pools.

[Source lines 135–141](C:/Thought/Projects/World/crates/world_core/src/audit.rs:135)

```rust
pub(crate) fn summarize(label: &str, pools: Pools, totals: Totals) -> serde_json::Value {
    serde_json::json!({"label": label, "mineral_nutrient": pools.mineral_nutrient,
        "living_nutrient": totals.living_nutrient, "living_energy": totals.living_energy,
        "detritus_nutrient": pools.detritus.nutrient, "detritus_energy": pools.detritus.energy,
        "heat": pools.heat, "mechanical_energy": pools.mechanical, "absorbed_light": pools.absorbed_light,
        "living_count": totals.living_count, "total_nutrient": totals.audit.nutrient, "accounted_energy": totals.audit.energy})
}
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `pub(crate)` shares an implementation item only inside its Rust crate.
- `#[derive]` generates ordinary cloning/debug/serialization implementations.

</details>

<details><summary><code>crates/world_core/src/transactions.rs</code></summary>

**Purpose:** Owns the ledger and commits proposed resource operations only after validation.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/transactions.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{
    audit::{AuditReport, audit_view, summarize},
    events::{OrganismChange, OrganismInventory, ResourceEvent},
    ids::OrganismId,
    organism::Organism,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
};
use crate::{lifecycle, physiology};
use super::*;
```

- `std` supplies maps/sets, paths/files, timing, and checked native utilities without extra crates.
- `serde` defines typed data serialization; strict records reject unsupported fields.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.

**Structure / enum: `Operation`**

Typed alternatives for all eleven resource requests; each alternative carries only its required arguments.

[Source lines 18–25](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:18)

```rust
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Photosynthesize {
        organism_id: OrganismId,
        light_energy: f64,
        #[serde(default = "one")]
        duration: f64,
```

**Structure / enum: `Command`**

The external schema-version and operation envelope decoded from JSON.

[Source lines 80–85](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:80)

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    schema_version: u32,
    operation: Operation,
}
```

**Structure / enum: `Pools`**

Environmental minerals, detritus, available/absorbed light, heat, and mechanical energy.

[Source lines 87–94](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:87)

```rust
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pools {
    pub mineral_nutrient: f64,
    pub detritus: Resources,
    pub light_available: f64,
    pub absorbed_light: f64,
    pub heat: f64,
```

**Methods — open the one you are reading**

<details><summary><code>validate()</code></summary>

Reject invalid values or inconsistent settings before this record is accepted.

[Source lines 99–105](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:99)

```rust
    pub fn validate(self) -> Result<()> {
        self.detritus.validate()?;
        for (name, value) in [
            ("mineral nutrient", self.mineral_nutrient),
            ("available light", self.light_available),
            ("absorbed light", self.absorbed_light),
            ("heat", self.heat),
```

</details>

**Structure / enum: `LedgerInitial`**

Declared starting budgets, pools, and organisms; construction checks their agreement.

[Source lines 114–121](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:114)

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerInitial {
    pub schema_version: u32,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub mineral_nutrient: f64,
    #[serde(default)]
```

**Structure / enum: `ResourceLedger`**

The authoritative Rust inventories and receipts, or their thin public Python wrapper.

[Source lines 128–135](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:128)

```rust
#[derive(Clone)]
pub struct ResourceLedger {
    rules: WorldRules,
    pools: Pools,
    initial_nutrient: f64,
    initial_energy: f64,
    organisms: BTreeMap<OrganismId, Organism>,
    events: Vec<ResourceEvent>,
```

**Methods — open the one you are reading**

<details><summary><code>population_state()</code></summary>

Copies the complete pool/budget/organism record for checkpoint serialization.

[Source lines 205–211](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:205)

```rust
    pub(crate) fn population_state(&self) -> PopulationState {
        PopulationState {
            pools: self.pools,
            initial_nutrient: self.initial_nutrient,
            initial_energy: self.initial_energy,
            organisms: self.organisms.values().cloned().collect(),
        }
```

</details>

<details><summary><code>from_population_state()</code></summary>

Rejects duplicate identities and inconsistent resources before reconstructing a ledger without standalone operation history.

[Source lines 214–220](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:214)

```rust
    pub(crate) fn from_population_state(rules: WorldRules, state: PopulationState) -> Result<Self> {
        rules.validate()?;
        finite(state.initial_nutrient, "initial nutrient", false)?;
        finite(state.initial_energy, "initial energy", false)?;
        let mut organisms = BTreeMap::new();
        for organism in state.organisms {
            if organisms.insert(organism.id, organism).is_some() {
```

</details>

<details><summary><code>new()</code></summary>

Construct this record or wrapper and validate its initial input.

[Source lines 238–244](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:238)

```rust
    pub fn new(rules: WorldRules, initial: LedgerInitial) -> Result<Self> {
        rules.validate()?;
        if initial.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported inventory schema".into(),
            ));
        }
```

</details>

<details><summary><code>from_json()</code></summary>

Decode typed JSON and run schema and record validation.

[Source lines 272–276](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:272)

```rust
    pub fn from_json(rules_json: &str, initial_json: &str) -> Result<Self> {
        let initial = serde_json::from_str(initial_json)
            .map_err(|error| AccountingError::Invalid(format!("invalid inventory: {error}")))?;
        Self::new(WorldRules::from_json(rules_json)?, initial)
    }
```

</details>

<details><summary><code>audit()</code></summary>

Check valid organism/resource state and conserved nutrient/energy budgets.

[Source lines 278–284](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:278)

```rust
    pub fn audit(&self) -> Result<AuditReport> {
        Ok(audit_view(
            &self.rules,
            self.pools,
            self.initial_nutrient,
            self.initial_energy,
            self.organisms.iter(),
```

</details>

<details><summary><code>audit_json()</code></summary>

Serialize the whole-ledger conservation report as JSON.

[Source lines 289–291](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:289)

```rust
    pub fn audit_json(&self) -> Result<String> {
        encode(&self.audit()?)
    }
```

</details>

<details><summary><code>summary_json()</code></summary>

Audit and serialize aggregate resource state with its inspection label.

[Source lines 292–294](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:292)

```rust
    pub fn summary_json(&self, label: &str) -> Result<String> {
        encode(&self.summary(label)?)
    }
```

</details>

<details><summary><code>summary()</code></summary>

Decode a fresh native inventory summary into ordinary Python data.

[Source lines 296–302](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:296)

```rust
    pub(crate) fn summary(&self, label: &str) -> Result<serde_json::Value> {
        let totals = audit_view(
            &self.rules,
            self.pools,
            self.initial_nutrient,
            self.initial_energy,
            self.organisms.iter(),
```

</details>

<details><summary><code>pools()</code></summary>

Read the ledger aggregate pools internally so the habitat can compare them with authoritative cell sums.

[Source lines 307–309](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:307)

```rust
    pub(crate) fn pools(&self) -> Pools {
        self.pools
    }
```

</details>

<details><summary><code>update()</code></summary>

Stage a whole population update, audit its proposed inventory, and commit only after success. Habitat window receipts are separate from standalone operation receipts.

[Source lines 311–317](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:311)

```rust
    pub(crate) fn update<T>(
        &mut self,
        execute: impl FnOnce(&mut Draft<'_>) -> Result<T>,
    ) -> Result<T> {
        self.audit()?;
        let mut draft = Draft {
            rules: &self.rules,
```

</details>

<details><summary><code>organisms()</code></summary>

Combine unchanged records, replacement edits, and newborns into the complete proposed-state audit view.

[Source lines 336–338](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:336)

```rust
    pub fn organisms(&self) -> &BTreeMap<OrganismId, Organism> {
        &self.organisms
    }
```

</details>

<details><summary><code>events()</code></summary>

Borrow committed receipts read-only.

[Source lines 339–341](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:339)

```rust
    pub fn events(&self) -> &[ResourceEvent] {
        &self.events
    }
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Serialize inspected native state; only the diagnostic fixture supports restoration.

[Source lines 343–349](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:343)

```rust
    pub fn snapshot_json(&self) -> Result<String> {
        self.audit()?;
        encode(
            &serde_json::json!({"schema_version": 1, "rules": self.rules,
            "initial_nutrient": self.initial_nutrient, "initial_energy": self.initial_energy,
            "pools": self.pools, "organisms": self.organisms.values().collect::<Vec<_>>(), "events": self.events}),
        )
```

</details>

<details><summary><code>apply_json()</code></summary>

Parse/validate the operation envelope and submit a native transaction.

[Source lines 352–358](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:352)

```rust
    pub fn apply_json(&mut self, text: &str) -> Result<()> {
        let command: Command = serde_json::from_str(text).map_err(|error| {
            AccountingError::Invalid(format!("invalid resource command: {error}"))
        })?;
        if command.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported command schema".into(),
```

</details>

<details><summary><code>apply()</code></summary>

Dispatch a typed native operation, or encode its command in the thin Python wrapper.

[Source lines 364–370](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:364)

```rust
    pub fn apply(&mut self, operation: Operation) -> Result<()> {
        self.transact(&operation, |draft| {
            use crate::{lifecycle, physiology};
            match operation {
                Operation::Photosynthesize {
                    organism_id,
                    light_energy,
```

</details>

<details><summary><code>transact()</code></summary>

Audit old state, execute draft, audit proposed state, prepare receipt, then commit records/pools/history.

[Source lines 428–434](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:428)

```rust
    fn transact(
        &mut self,
        operation: &Operation,
        execute: impl FnOnce(&mut Draft<'_>) -> Result<()>,
    ) -> Result<()> {
        self.audit()?;
        let mut draft = Draft {
```

</details>

**Structure / enum: `PopulationState`**

Carries initial resource budgets, current pools, and complete organism records when reconstructing the ledger.

[Source lines 138–145](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:138)

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PopulationState {
    pub pools: Pools,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub organisms: Vec<Organism>,
}
```

**Structure / enum: `Draft`**

Copied pools and staged organism edits over a read-only committed population, allowing rejection without partial writes.

[Source lines 147–152](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:147)

```rust
pub(crate) struct Draft<'a> {
    pub rules: &'a WorldRules,
    pub pools: Pools,
    base: &'a BTreeMap<OrganismId, Organism>,
    edits: BTreeMap<OrganismId, Organism>,
}
```

**Methods — open the one you are reading**

<details><summary><code>get()</code></summary>

Borrow the latest proposed organism record, falling back to committed state, without cloning it for inspection.

[Source lines 155–160](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:155)

```rust
    pub(crate) fn get(&self, id: OrganismId) -> Result<&Organism> {
        self.edits
            .get(&id)
            .or_else(|| self.base.get(&id))
            .ok_or_else(|| AccountingError::Invalid(format!("unknown organism {}", id.0)))
    }
```

</details>

<details><summary><code>living()</code></summary>

Read edits before committed state and clone a living organism for proposed changes.

[Source lines 162–168](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:162)

```rust
    pub fn living(&self, id: OrganismId) -> Result<Organism> {
        let organism = self.get(id)?;
        if !organism.alive {
            return Err(AccountingError::Invalid("organism is dead".into()));
        }
        Ok(organism.clone())
    }
```

</details>

<details><summary><code>exists()</code></summary>

Check both committed and staged identity records.

[Source lines 170–172](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:170)

```rust
    pub fn exists(&self, id: OrganismId) -> bool {
        self.edits.contains_key(&id) || self.base.contains_key(&id)
    }
```

</details>

<details><summary><code>put()</code></summary>

Stage a changed/new organism record.

[Source lines 173–175](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:173)

```rust
    pub fn put(&mut self, organism: Organism) {
        self.edits.insert(organism.id, organism);
    }
```

</details>

<details><summary><code>organisms()</code></summary>

Combine unchanged records, replacement edits, and newborns into the complete proposed-state audit view.

[Source lines 177–183](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:177)

```rust
    pub(crate) fn organisms(&self) -> impl Iterator<Item = (&OrganismId, &Organism)> {
        self.base
            .iter()
            .map(|(id, organism)| (id, self.edits.get(id).unwrap_or(organism)))
            .chain(
                self.edits
                    .iter()
```

</details>

<details><summary><code>set_reserves()</code></summary>

Apply capacities and account overflow before staging the updated organism.

[Source lines 188–194](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:188)

```rust
    pub fn set_reserves(&mut self, mut organism: Organism, reserves: Resources) -> Result<()> {
        reserves.validate()?;
        let capacity = organism.reserve_capacity(self.rules)?;
        let n_cap = capacity.nutrient;
        let e_cap = capacity.energy;
        finite(n_cap, "nutrient capacity", false)?;
        finite(e_cap, "energy capacity", false)?;
```

</details>

**Standalone functions**

<details><summary><code>one()</code></summary>

Supply a one-unit default duration when the JSON request omits it.

[Source lines 14–16](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:14)

```rust
fn one() -> f64 {
    1.0
}
```

</details>

<details><summary><code>encode()</code></summary>

Serialize structured data as JSON and translate errors.

[Source lines 481–483](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:481)

```rust
fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|error| AccountingError::Invalid(error.to_string()))
}
```

</details>

<details><summary><code>failed_commit_audit_discards_staged_pools_and_organisms()</code></summary>

Failed commit audit discards staged pools and organisms.

[Source lines 490–496](C:/Thought/Projects/World/crates/world_core/src/transactions.rs:490)

```rust
    fn failed_commit_audit_discards_staged_pools_and_organisms() {
        let rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        let initial = LedgerInitial {
            schema_version: 1,
            initial_nutrient: 1.0,
            initial_energy: 2.0,
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `pub(crate)` shares an implementation item only inside its Rust crate.
- `#[derive]` generates ordinary cloning/debug/serialization implementations.
- `checked_add` rejects identity/counter overflow instead of wrapping.
- `#[test]` registers native contract checks; failed assertions fail validation.

</details>

<details><summary><code>crates/world_core/src/physiology.rs</code></summary>

**Purpose:** Calculates resource acquisition, feeding, growth, maintenance, and energy costs.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/physiology.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use crate::{
    ids::OrganismId,
    lifecycle::{after_tissue_loss, die},
    organism::DeathCause,
    resources::{AccountingError, Resources, Result, finite},
    transactions::Draft,
};
```

- `crate` imports neighboring native modules so biology reuses one accounting implementation.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.

**Standalone functions**

<details><summary><code>rate()</code></summary>

Validate finite positive inputs/capacity and reject excess processing.

[Source lines 9–15](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:9)

```rust
fn rate(amount: f64, duration: f64, limit: f64) -> Result<()> {
    finite(amount, "amount", true)?;
    finite(duration, "duration", true)?;
    let capacity = limit * duration;
    finite(capacity, "process capacity", true)?;
    if amount > capacity {
        return Err(AccountingError::Invalid("process capacity exceeded".into()));
```

</details>

<details><summary><code>photosynthesize()</code></summary>

Spend available light and divide captured input between reserves and heat.

[Source lines 20–26](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:20)

```rust
pub(crate) fn photosynthesize(
    draft: &mut Draft<'_>,
    id: OrganismId,
    light: f64,
    duration: f64,
) -> Result<()> {
    let organism = draft.living(id)?;
```

</details>

<details><summary><code>absorb()</code></summary>

Transfer existing minerals into capable reserves within processing/storage limits.

[Source lines 47–53](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:47)

```rust
pub(crate) fn absorb(
    draft: &mut Draft<'_>,
    id: OrganismId,
    nutrient: f64,
    duration: f64,
) -> Result<()> {
    let organism = draft.living(id)?;
```

</details>

<details><summary><code>feed()</code></summary>

Validate contact/capability; prepay digestion; remove target tissue; split assimilation/waste; check viability.

[Source lines 69–75](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:69)

```rust
pub(crate) fn feed(
    draft: &mut Draft<'_>,
    consumer_id: OrganismId,
    target_id: OrganismId,
    amount: f64,
    contact: bool,
    duration: f64,
```

</details>

<details><summary><code>feed_from_module()</code></summary>

Transfers only existing contacted module tissue into the usual assimilation/waste/heat accounts and applies local injury/death.

[Source lines 83–89](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:83)

```rust
pub(crate) fn feed_from_module(
    draft: &mut Draft<'_>,
    consumer_id: OrganismId,
    target_id: OrganismId,
    amount: f64,
    duration: f64,
    module: Option<u64>,
```

</details>

<details><summary><code>grow()</code></summary>

Spend nutrient and energy reserves on new tissue and construction heat.

[Source lines 124–130](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:124)

```rust
pub(crate) fn grow(
    draft: &mut Draft<'_>,
    id: OrganismId,
    amount: f64,
    duration: f64,
) -> Result<()> {
    let mut organism = draft.living(id)?;
```

</details>

<details><summary><code>grow_available()</code></summary>

Calculates growth affordable from rates and reserves, conservatively handling a possible floating-point boundary, then uses the existing growth transfer.

[Source lines 147–153](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:147)

```rust
pub(crate) fn grow_available(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<f64> {
    let rules = draft.rules;
    let organism = draft.get(id)?;
    let cost = rules.tissue_energy_per_nutrient + rules.construction_heat_per_nutrient;
    finite(cost, "growth energy per nutrient", true)?;
    let mut amount = (rules.max_growth_nutrient_per_time * duration)
        .min(organism.reserves.nutrient)
```

</details>

<details><summary><code>maintain()</code></summary>

Pay tissue-based maintenance, advance individual age, and commit starvation if underfunded.

[Source lines 166–172](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:166)

```rust
pub(crate) fn maintain(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<()> {
    let mut organism = draft.living(id)?;
    finite(duration, "duration", true)?;
    let required =
        organism.tissue.nutrient * draft.rules.maintenance_per_tissue_nutrient_per_time * duration;
    finite(required, "maintenance requirement", false)?;
    let paid = required.min(organism.reserves.energy);
```

</details>

<details><summary><code>decompose()</code></summary>

Process existing detritus within separate nutrient/energy rate caps; minerals and heat receive outputs.

[Source lines 185–191](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:185)

```rust
pub(crate) fn decompose(draft: &mut Draft<'_>, fraction: f64, duration: f64) -> Result<()> {
    finite(fraction, "fraction", true)?;
    finite(duration, "duration", true)?;
    if fraction > 1.0 || draft.pools.detritus == Resources::default() {
        return Err(AccountingError::Invalid(
            "decomposition requires existing detritus and fraction at most one".into(),
        ));
```

</details>

<details><summary><code>sense()</code></summary>

Spend sensing energy as heat.

[Source lines 214–220](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:214)

```rust
pub(crate) fn sense(draft: &mut Draft<'_>, id: OrganismId, duration: f64) -> Result<()> {
    finite(duration, "duration", true)?;
    let mut organism = draft.living(id)?;
    let cost = draft.rules.sensor_energy_per_time * duration;
    finite(cost, "sensing cost", true)?;
    organism.reserves = organism.reserves.minus(Resources::new(0.0, cost)?)?;
    draft.pools.heat += cost;
```

</details>

<details><summary><code>actuate()</code></summary>

Spend reserves on mechanical energy plus heat.

[Source lines 225–231](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:225)

```rust
pub(crate) fn actuate(draft: &mut Draft<'_>, id: OrganismId, energy: f64) -> Result<()> {
    finite(energy, "energy", true)?;
    let mut organism = draft.living(id)?;
    organism.reserves = organism.reserves.minus(Resources::new(0.0, energy)?)?;
    let work = energy * draft.rules.movement_efficiency;
    draft.pools.mechanical += work;
    draft.pools.heat += energy - work;
```

</details>

<details><summary><code>dissipate_motion()</code></summary>

Transfer existing mechanical energy to heat.

[Source lines 236–242](C:/Thought/Projects/World/crates/world_core/src/physiology.rs:236)

```rust
pub(crate) fn dissipate_motion(draft: &mut Draft<'_>, energy: f64) -> Result<()> {
    finite(energy, "mechanical dissipation", true)?;
    if energy > draft.pools.mechanical {
        return Err(AccountingError::Invalid(
            "insufficient mechanical energy".into(),
        ));
    }
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `pub(crate)` shares an implementation item only inside its Rust crate.
- `Option<T>` represents an absent/present value; `Result<T>` carries success/error.

</details>

<details><summary><code>crates/world_core/src/lifecycle.rs</code></summary>

**Purpose:** Handles viability, death transfers, and fully parent-funded clone births.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use world_genetics::inheritance::inherit;
use crate::{
    ids::OrganismId,
    organism::{DeathCause, Organism},
    resources::{AccountingError, Resources, Result, finite},
    transactions::Draft,
};
```

- `world_genetics` owns inherited body/neural genes and deterministic mutation operators.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.

**Standalone functions**

<details><summary><code>die()</code></summary>

Move all remaining resources to detritus and retain an empty dead organism with cause.

[Source lines 10–16](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:10)

```rust
pub(crate) fn die(draft: &mut Draft<'_>, mut organism: Organism, cause: DeathCause) -> Result<()> {
    draft.pools.detritus = draft
        .pools
        .detritus
        .plus(organism.tissue)?
        .plus(organism.reserves)?;
    organism.tissue = Resources::default();
```

</details>

<details><summary><code>after_tissue_loss()</code></summary>

Kill below viability or adjust storage capacity for surviving tissue.

[Source lines 29–35](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:29)

```rust
pub(crate) fn after_tissue_loss(
    draft: &mut Draft<'_>,
    organism: Organism,
    cause: DeathCause,
) -> Result<()> {
    let core_failed = organism
        .genome
```

</details>

<details><summary><code>damage()</code></summary>

Transfer destroyed tissue to detritus and check remaining viability/capacity.

[Source lines 58–64](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:58)

```rust
pub(crate) fn damage(draft: &mut Draft<'_>, id: OrganismId, amount: f64) -> Result<()> {
    finite(amount, "damaged tissue", true)?;
    let mut organism = draft.living(id)?;
    let lost = Resources::new(amount, amount * draft.rules.tissue_energy_per_nutrient)?;
    if let Some(body) = &mut organism.body {
        body.remove(amount, None)?;
    }
```

</details>

<details><summary><code>reproduce()</code></summary>

Check maturity/cooldown/new ID/placement, fund the child entirely from parent reserves, and clone the genome.

[Source lines 70–76](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:70)

```rust
pub(crate) fn reproduce(
    draft: &mut Draft<'_>,
    parent_id: OrganismId,
    child_id: OrganismId,
    placement: bool,
) -> Result<()> {
    let genome = inherit(&draft.living(parent_id)?.genome);
```

</details>

<details><summary><code>reproduce_blueprint()</code></summary>

Supports reproduce blueprint within this component's responsibility; the source link opens the complete implementation.

[Source lines 87–93](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:87)

```rust
pub(crate) fn reproduce_blueprint(
    draft: &mut Draft<'_>,
    parent_id: OrganismId,
    child_id: OrganismId,
    placement: bool,
    genome: world_genetics::genome::Genome,
    tissue_nutrient: f64,
```

</details>

<details><summary><code>birth_investment()</code></summary>

Calculates the nutrient and energy required for offspring tissue, reserves, and construction heat before attempting birth.

[Source lines 148–150](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:148)

```rust
pub(crate) fn birth_investment(rules: &crate::rules::WorldRules) -> Result<Resources> {
    birth_investment_for(rules, rules.offspring_tissue_nutrient)
}
```

</details>

<details><summary><code>birth_investment_for()</code></summary>

Computes exact tissue/reserve/construction funding for a declared juvenile tissue amount using the lifecycle's arithmetic order.

[Source lines 152–158](C:/Thought/Projects/World/crates/world_core/src/lifecycle.rs:152)

```rust
pub(crate) fn birth_investment_for(
    rules: &crate::rules::WorldRules,
    tissue: f64,
) -> Result<Resources> {
    Resources::new(
        tissue + rules.offspring_reserve_nutrient,
        tissue * rules.tissue_energy_per_nutrient
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `pub(crate)` shares an implementation item only inside its Rust crate.

</details>

<details><summary><code>crates/world_bindings/src/resources.rs</code></summary>

**Purpose:** Makes the native ledger callable from Python and translates its errors.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use world_core::{resources::AccountingError, transactions::ResourceLedger};
```

- `pyo3`: Python-visible classes, methods, properties, and error translation.
- `world_core`: the Python-independent native habitat and error types.

**Structure / enum: `PyResourceLedger`**

The Python-visible wrapper owning the authoritative Rust ledger.

[Source lines 7–10](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:7)

```rust
#[pyclass(name = "ResourceLedger", module = "world._native")]
pub struct PyResourceLedger {
    inner: ResourceLedger,
}
```

**Methods — open the one you are reading**

<details><summary><code>new()</code></summary>

Construct this record or wrapper and validate its initial input.

[Source lines 15–19](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:15)

```rust
    fn new(rules_json: &str, initial_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: ResourceLedger::from_json(rules_json, initial_json).map_err(error)?,
        })
    }
```

</details>

<details><summary><code>apply_json()</code></summary>

Parse/validate the operation envelope and submit a native transaction.

[Source lines 21–23](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:21)

```rust
    fn apply_json(&mut self, command_json: &str) -> PyResult<()> {
        self.inner.apply_json(command_json).map_err(error)
    }
```

</details>

<details><summary><code>audit_json()</code></summary>

Serialize the whole-ledger conservation report as JSON.

[Source lines 25–27](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:25)

```rust
    fn audit_json(&self) -> PyResult<String> {
        self.inner.audit_json().map_err(error)
    }
```

</details>

<details><summary><code>summary_json()</code></summary>

Audit and serialize aggregate resource state with its inspection label.

[Source lines 28–30](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:28)

```rust
    fn summary_json(&self, label: &str) -> PyResult<String> {
        self.inner.summary_json(label).map_err(error)
    }
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Serialize inspected native state; only the diagnostic fixture supports restoration.

[Source lines 31–33](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:31)

```rust
    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(error)
    }
```

</details>

**Standalone functions**

<details><summary><code>error()</code></summary>

Map invalid accounting requests to ValueError and invariant failures to RuntimeError.

[Source lines 36–41](C:/Thought/Projects/World/crates/world_bindings/src/resources.rs:36)

```rust
fn error(value: AccountingError) -> PyErr {
    match value {
        AccountingError::Invalid(message) => PyValueError::new_err(message),
        AccountingError::Invariant(message) => PyRuntimeError::new_err(message),
    }
}
```

</details>

**Useful syntax in this file**

- `#[pyclass]` and `#[pymethods]` expose Rust records and methods to Python.

</details>

<details><summary><code>python/world/accounting.py</code></summary>

**Purpose:** Provides the small public Python interface without duplicating biological formulas.

[Open the file](C:/Thought/Projects/World/python/world/accounting.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import json
from ._native import ResourceLedger as NativeResourceLedger
```

- `json`: conversion between dictionaries and the native JSON interface.
- `._native`: the compiled Rust habitat, aliased to distinguish it from the Python wrapper.

**Class: `ResourceLedger`**

The authoritative Rust inventories and receipts, or their thin public Python wrapper.

[Source lines 6–6](C:/Thought/Projects/World/python/world/accounting.py:6)

```python
class ResourceLedger:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Encode input dictionaries and construct the native ledger; reject nonfinite JSON numbers.

[Source lines 7–9](C:/Thought/Projects/World/python/world/accounting.py:7)

```python
    def __init__(self, rules: dict, initial: dict):
        self._native = NativeResourceLedger(json.dumps(rules, allow_nan=False),
                                             json.dumps(initial, allow_nan=False))
```

</details>

<details><summary><code>apply()</code></summary>

Build the versioned operation envelope and submit it to Rust.

[Source lines 11–15](C:/Thought/Projects/World/python/world/accounting.py:11)

```python
    def apply(self, operation: str, **arguments):
        if "kind" in arguments:
            raise ValueError("kind is reserved for the operation name")
        command = {"schema_version": 1, "operation": {"kind": operation, **arguments}}
        self._native.apply_json(json.dumps(command, allow_nan=False))
```

</details>

<details><summary><code>audit()</code></summary>

Decode the native conservation report into a Python dictionary.

[Source lines 17–18](C:/Thought/Projects/World/python/world/accounting.py:17)

```python
    def audit(self) -> dict:
        return json.loads(self._native.audit_json())
```

</details>

<details><summary><code>summary()</code></summary>

Return the audited aggregate resource inventory.

[Source lines 20–21](C:/Thought/Projects/World/python/world/accounting.py:20)

```python
    def summary(self, label: str = "state") -> dict:
        return json.loads(self._native.summary_json(label))
```

</details>

<details><summary><code>snapshot()</code></summary>

Decode an independent inspection copy; this is not a restore API.

[Source lines 23–24](C:/Thought/Projects/World/python/world/accounting.py:23)

```python
    def snapshot(self) -> dict:
        return json.loads(self._native.snapshot_json())
```

</details>

**Useful syntax in this file**

- `**arguments` collects or expands named Python arguments.
- `allow_nan=False` prevents nonfinite numbers from being encoded as JSON.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>examples/native_lifecycle.py</code></summary>

**Purpose:** Runs the known lifecycle through Rust and checks all expected summaries.

[Open the file](C:/Thought/Projects/World/examples/native_lifecycle.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import argparse
import json
import math
from pathlib import Path
from world import ResourceLedger
```

- `argparse`: command-line controls for run duration, sampling, and output.
- `json`: conversion between dictionaries and the native JSON interface.
- `math`: independent finite-value checks and inventory sums in tests.
- `pathlib`: configuration and output paths.
- `world`: the installed public Habitat interface used by examples and tests.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 9–9](C:/Thought/Projects/World/examples/native_lifecycle.py:9)

```python
ROOT = Path(__file__).resolve().parents[1]
```

[Source lines 10–10](C:/Thought/Projects/World/examples/native_lifecycle.py:10)

```python
IDS = {"producer": 1, "consumer": 2, "child": 3}
```

**Standalone functions**

<details><summary><code>initial_from_fixture()</code></summary>

Map reference string labels and flat resources into the native numeric-ID inventory.

[Source lines 13–18](C:/Thought/Projects/World/examples/native_lifecycle.py:13)

```python
def initial_from_fixture(fixture: dict) -> dict:
    inventory = fixture["initial_inventory"]
    organisms = [{"id": IDS[item["id"]],
                  "genome": {"photosynthetic": item["photosynthetic"], "can_ingest": item["can_ingest"],
                             "initial_weights": item["weights"]},
                  "tissue": {"nutrient": item["tissue_nutrient"], "energy": item["tissue_energy"]},
```

</details>

<details><summary><code>run_example()</code></summary>

Run seven prescribed operations, collect eight summaries, and check independent expected results.

[Source lines 26–31](C:/Thought/Projects/World/examples/native_lifecycle.py:26)

```python
def run_example():
    rules = json.loads((ROOT / "configs/world_rules_v0.json").read_text(encoding="utf-8"))
    fixture = json.loads((ROOT / "examples/lifecycle_expected.json").read_text(encoding="utf-8"))
    if rules["model_id"] != fixture["model_id"]:
        raise ValueError("fixture and rules have different model IDs")
    ledger = ResourceLedger(rules, initial_from_fixture(fixture))
```

</details>

<details><summary><code>main()</code></summary>

Read command-line options, coordinate this file's work, and print or save its result.

[Source lines 57–62](C:/Thought/Projects/World/examples/native_lifecycle.py:57)

```python
def main():
    parser = argparse.ArgumentParser(description="Run the native resource lifecycle against expected results.")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    ledger, snapshots = run_example()
    for row in snapshots:
```

</details>

**Useful syntax in this file**

- `**arguments` collects or expands named Python arguments.
- `allow_nan=False` prevents nonfinite numbers from being encoded as JSON.

</details>

<details><summary><code>tests/test_native_resources.py</code></summary>

**Purpose:** Compares native and reference behavior, including 200 seeded mixed requests and rejected-state preservation.

[Open the file](C:/Thought/Projects/World/tests/test_native_resources.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
from copy import deepcopy
from dataclasses import asdict
import json
import math
from pathlib import Path
import random
import unittest
from world import ResourceLedger
from world._native import ResourceLedger as RawLedger
from reference.ledger import Genome, Ledger, Organism, Resources
from reference.rules import WorldRules
from reference.worked_example import run_example as run_reference
from examples.native_lifecycle import initial_from_fixture, run_example
```

- `copy`: defensive nested-data copying.
- `dataclasses`: record construction, field inspection, and updated immutable copies.
- `json`: conversion between dictionaries and the native JSON interface.
- `math`: independent finite-value checks and inventory sums in tests.
- `pathlib`: configuration and output paths.
- `random`: repeatable pseudorandom test inputs.
- `unittest`: fresh test cases, assertions, and expected exceptions.
- `world`: the installed public Habitat interface used by examples and tests.
- `world._native`: the raw native boundary, tested independently of wrapper JSON validation.
- `reference.ledger`: matching project types and validation used by this file.
- `reference.rules`: matching project types and validation used by this file.
- `reference.worked_example`: matching project types and validation used by this file.
- `examples.native_lifecycle`: matching project types and validation used by this file.

**Module-level values**

These names hold shared metadata, paths, identities, or test inputs used by the functions below.

[Source lines 17–17](C:/Thought/Projects/World/tests/test_native_resources.py:17)

```python
ROOT = Path(__file__).resolve().parents[1]
```

[Source lines 18–18](C:/Thought/Projects/World/tests/test_native_resources.py:18)

```python
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text())
```

[Source lines 19–19](C:/Thought/Projects/World/tests/test_native_resources.py:19)

```python
FIXTURE = json.loads((ROOT / "examples/lifecycle_expected.json").read_text())
```

[Source lines 20–20](C:/Thought/Projects/World/tests/test_native_resources.py:20)

```python
INITIAL = initial_from_fixture(FIXTURE)
```

[Source lines 21–21](C:/Thought/Projects/World/tests/test_native_resources.py:21)

```python
ID_FIELDS = {"organism_id", "consumer_id", "target_id", "parent_id", "child_id"}
```

**Class: `NativeResourceTests`**

A test group that constructs fresh fixtures and checks related behavior.

[Source lines 36–36](C:/Thought/Projects/World/tests/test_native_resources.py:36)

```python
class NativeResourceTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>setUp()</code></summary>

Create fresh state before each test.

[Source lines 37–38](C:/Thought/Projects/World/tests/test_native_resources.py:37)

```python
    def setUp(self):
        self.pair()
```

</details>

<details><summary><code>pair()</code></summary>

Create native/reference ledgers from copied matching inventories, converting identity representations.

[Source lines 40–45](C:/Thought/Projects/World/tests/test_native_resources.py:40)

```python
    def pair(self, initial=None, rules=None):
        self.initial = deepcopy(INITIAL if initial is None else initial)
        self.rules = deepcopy(RULES if rules is None else rules)
        self.native = ResourceLedger(self.rules, self.initial)
        organisms = []
        for item in self.initial["organisms"]:
```

</details>

<details><summary><code>equal_values()</code></summary>

Compare nested structures; IDs/integers exactly, floats within tolerance.

[Source lines 57–62](C:/Thought/Projects/World/tests/test_native_resources.py:57)

```python
    def equal_values(self, actual, expected):
        if isinstance(expected, dict):
            self.assertEqual(actual.keys(), expected.keys())
            for key in expected:
                self.equal_values(actual[key], expected[key])
        elif isinstance(expected, (list, tuple)):
```

</details>

<details><summary><code>equal_state()</code></summary>

Compare complete organism state, genomes, summaries, audits, and event metadata.

[Source lines 71–76](C:/Thought/Projects/World/tests/test_native_resources.py:71)

```python
    def equal_state(self):
        self.equal_values(self.native.summary(), self.reference.summary("state"))
        expected = {}
        for identity, organism in self.reference.organisms.items():
            item = asdict(organism)
            item["id"] = int(item.pop("organism_id"))
```

</details>

<details><summary><code>both()</code></summary>

Run equivalent native/reference requests; compare acceptance, results, and unchanged rejected state.

[Source lines 88–93](C:/Thought/Projects/World/tests/test_native_resources.py:88)

```python
    def both(self, kind, **arguments):
        before_native = self.native.snapshot()
        before_reference = (self.reference.summary("state"), dict(self.reference.organisms), self.reference.events)
        errors = []
        converted = {key: str(value) if key in ID_FIELDS else value for key, value in arguments.items()}
        for operation in (lambda: self.native.apply(kind, **arguments),
```

</details>

<details><summary><code>test_lifecycle_matches_independent_fixture_and_python_reference()</code></summary>

Checks lifecycle matches independent fixture and python reference.

[Source lines 107–112](C:/Thought/Projects/World/tests/test_native_resources.py:107)

```python
    def test_lifecycle_matches_independent_fixture_and_python_reference(self):
        ledger, actual = run_example()
        _, expected = run_reference()
        self.equal_values(actual, expected)
        self.equal_values(actual, FIXTURE["expected_snapshots"])
        self.assertEqual(len(ledger.snapshot()["events"]), 7)
```

</details>

<details><summary><code>test_photosynthesis_light_budget_and_energy_saturation()</code></summary>

Checks photosynthesis light budget and energy saturation.

[Source lines 114–119](C:/Thought/Projects/World/tests/test_native_resources.py:114)

```python
    def test_photosynthesis_light_budget_and_energy_saturation(self):
        self.assertFalse(self.both("photosynthesize", organism_id=2, light_energy=1))
        self.assertFalse(self.both("photosynthesize", organism_id=1, light_energy=11))
        self.assertTrue(self.both("photosynthesize", organism_id=1, light_energy=10))
        self.assertFalse(self.both("photosynthesize", organism_id=1, light_energy=1))
        self.pair(single(energy=39, nutrient=5, photosynthetic=True, light=10))
```

</details>

<details><summary><code>test_mineral_uptake_capacity_capability_and_rate()</code></summary>

Checks mineral uptake capacity capability and rate.

[Source lines 123–128](C:/Thought/Projects/World/tests/test_native_resources.py:123)

```python
    def test_mineral_uptake_capacity_capability_and_rate(self):
        self.assertFalse(self.both("absorb", organism_id=2, nutrient=1))
        self.assertFalse(self.both("absorb", organism_id=1, nutrient=2))
        self.pair(single(nutrient=8, photosynthetic=True))
        self.assertTrue(self.both("absorb", organism_id=7, nutrient=1))
        self.assertEqual(self.native.summary()["mineral_nutrient"], 10)
```

</details>

<details><summary><code>test_feeding_assimilation_prepayment_and_repeat_claims()</code></summary>

Checks feeding assimilation prepayment and repeat claims.

[Source lines 132–137](C:/Thought/Projects/World/tests/test_native_resources.py:132)

```python
    def test_feeding_assimilation_prepayment_and_repeat_claims(self):
        self.assertTrue(self.both("feed", consumer_id=2, target_id=1, tissue_nutrient=2, contact=True))
        self.assertEqual(self.native.summary()["detritus_nutrient"], 0.5)
        for arguments in [dict(consumer_id=2, target_id=1, tissue_nutrient=1, contact=False),
                          dict(consumer_id=2, target_id=2, tissue_nutrient=1, contact=True),
                          dict(consumer_id=1, target_id=2, tissue_nutrient=1, contact=True),
```

</details>

<details><summary><code>test_growth_requires_reserves_and_retains_embodied_energy()</code></summary>

Checks growth requires reserves and retains embodied energy.

[Source lines 149–154](C:/Thought/Projects/World/tests/test_native_resources.py:149)

```python
    def test_growth_requires_reserves_and_retains_embodied_energy(self):
        self.assertTrue(self.both("grow", organism_id=2, tissue_nutrient=1))
        organism = self.native.snapshot()["organisms"][1]
        self.assertEqual(organism["tissue"], {"nutrient": 9, "energy": 18})
        self.assertEqual(organism["reserves"], {"nutrient": 1, "energy": 31})
        self.assertFalse(self.both("grow", organism_id=2, tissue_nutrient=2))
```

</details>

<details><summary><code>test_exact_maintenance_then_starvation_commits_death()</code></summary>

Checks exact maintenance then starvation commits death.

[Source lines 159–164](C:/Thought/Projects/World/tests/test_native_resources.py:159)

```python
    def test_exact_maintenance_then_starvation_commits_death(self):
        self.pair(single(energy=0.8))
        self.assertTrue(self.both("maintain", organism_id=7))
        self.assertTrue(self.native.snapshot()["organisms"][0]["alive"])
        self.assertTrue(self.both("maintain", organism_id=7))
        organism = self.native.snapshot()["organisms"][0]
```

</details>

<details><summary><code>test_birth_debits_parent_preserves_genome_and_records_ancestry()</code></summary>

Checks birth debits parent preserves genome and records ancestry.

[Source lines 170–175](C:/Thought/Projects/World/tests/test_native_resources.py:170)

```python
    def test_birth_debits_parent_preserves_genome_and_records_ancestry(self):
        self.pair(single(weights=[0.25, -0.5]))
        self.assertTrue(self.both("reproduce", parent_id=7, child_id=2**53 + 1, placement_available=True))
        parent, child = self.native.snapshot()["organisms"]
        self.assertEqual(parent["reserves"], {"nutrient": 2.5, "energy": 9})
        self.assertEqual(child["genome"], parent["genome"])
```

</details>

<details><summary><code>test_birth_rejects_unfunded_immature_duplicate_and_unplaced_requests()</code></summary>

Checks birth rejects unfunded immature duplicate and unplaced requests.

[Source lines 182–187](C:/Thought/Projects/World/tests/test_native_resources.py:182)

```python
    def test_birth_rejects_unfunded_immature_duplicate_and_unplaced_requests(self):
        for initial in [single(age=1), single(tissue=5), single(energy=10), single(nutrient=2)]:
            self.pair(initial)
            self.assertFalse(self.both("reproduce", parent_id=7, child_id=8, placement_available=True))
        self.pair(single())
        self.assertFalse(self.both("reproduce", parent_id=7, child_id=7, placement_available=True))
```

</details>

<details><summary><code>test_damage_viability_capacity_reduction_and_single_death_transfer()</code></summary>

Checks damage viability capacity reduction and single death transfer.

[Source lines 190–195](C:/Thought/Projects/World/tests/test_native_resources.py:190)

```python
    def test_damage_viability_capacity_reduction_and_single_death_transfer(self):
        self.assertTrue(self.both("damage", organism_id=1, tissue_nutrient=9))
        self.assertTrue(self.native.snapshot()["organisms"][0]["alive"])
        self.assertEqual(self.native.snapshot()["organisms"][0]["reserves"], {"nutrient": 1, "energy": 5})
        self.assertTrue(self.both("damage", organism_id=1, tissue_nutrient=0.5))
        self.assertFalse(self.both("damage", organism_id=1, tissue_nutrient=1))
```

</details>

<details><summary><code>test_decomposition_preserves_nutrients_and_handles_energy_only_waste()</code></summary>

Checks decomposition preserves nutrients and handles energy only waste.

[Source lines 200–205](C:/Thought/Projects/World/tests/test_native_resources.py:200)

```python
    def test_decomposition_preserves_nutrients_and_handles_energy_only_waste(self):
        self.both("damage", organism_id=1, tissue_nutrient=10)
        self.both("decompose", fraction=0.5)
        self.assertEqual(self.native.summary()["detritus_nutrient"], 7.5)
        self.assertEqual(self.native.summary()["mineral_nutrient"], 82.5)
        self.both("decompose", fraction=1)
```

</details>

<details><summary><code>test_sensing_and_mechanical_energy_survive_actor_death()</code></summary>

Checks sensing and mechanical energy survive actor death.

[Source lines 213–218](C:/Thought/Projects/World/tests/test_native_resources.py:213)

```python
    def test_sensing_and_mechanical_energy_survive_actor_death(self):
        self.both("sense", organism_id=2)
        self.both("actuate", organism_id=2, energy=4)
        self.assertEqual(self.native.summary()["mechanical_energy"], 2)
        self.both("damage", organism_id=2, tissue_nutrient=8)
        self.assertEqual(self.native.summary()["mechanical_energy"], 2)
```

</details>

<details><summary><code>test_invalid_values_and_small_process_windows_leave_no_partial_state()</code></summary>

Checks invalid values and small process windows leave no partial state.

[Source lines 223–228](C:/Thought/Projects/World/tests/test_native_resources.py:223)

```python
    def test_invalid_values_and_small_process_windows_leave_no_partial_state(self):
        for bad in [-1, 0, math.nan, math.inf, -math.inf, True]:
            for kind, arguments in [("actuate", {"energy": bad}), ("grow", {"tissue_nutrient": bad}),
                                    ("maintain", {"duration": bad})]:
                self.assertFalse(self.both(kind, organism_id=2, **arguments))
        for kind, arguments in [("photosynthesize", {"organism_id": 1, "light_energy": 2}),
```

</details>

<details><summary><code>test_initial_inventory_and_organism_invariants_are_enforced()</code></summary>

Checks initial inventory and organism invariants are enforced.

[Source lines 235–240](C:/Thought/Projects/World/tests/test_native_resources.py:235)

```python
    def test_initial_inventory_and_organism_invariants_are_enforced(self):
        for change in [lambda item: item.update(initial_nutrient=101),
                       lambda item: item["organisms"][0]["tissue"].update(energy=21),
                       lambda item: item["organisms"][0].update(alive=False),
                       lambda item: item["organisms"][0].update(death_cause="damage"),
                       lambda item: item["organisms"][0]["reserves"].update(energy=100)]:
```

</details>

<details><summary><code>test_invalid_rule_definitions_are_rejected()</code></summary>

Checks invalid rule definitions are rejected.

[Source lines 250–255](C:/Thought/Projects/World/tests/test_native_resources.py:250)

```python
    def test_invalid_rule_definitions_are_rejected(self):
        for update in [{"schema_version": 2}, {"physics_dt": 0}, {"grid_width": 2.5},
                       {"energy_assimilation_fraction": 1.1}, {"movement_efficiency": math.nan},
                       {"offspring_tissue_nutrient": 0.5}, {"offspring_reserve_energy": 100},
                       {"offspring_reserve_nutrient": 3}, {"model_id": ""}, {"unknown": 1}]:
            with self.subTest(update=update), self.assertRaises((ValueError, TypeError)):
```

</details>

<details><summary><code>test_raw_protocol_rejects_unknown_fields_bad_ids_and_nonfinite_json()</code></summary>

Checks raw protocol rejects unknown fields bad ids and nonfinite json.

[Source lines 258–263](C:/Thought/Projects/World/tests/test_native_resources.py:258)

```python
    def test_raw_protocol_rejects_unknown_fields_bad_ids_and_nonfinite_json(self):
        native = RawLedger(json.dumps(RULES), json.dumps(INITIAL))
        before = native.snapshot_json()
        for text in ['{"schema_version":1,"operation":{"kind":"sense","organism_id":true}}',
                     '{"schema_version":1,"operation":{"kind":"sense","organism_id":1.0}}',
                     '{"schema_version":1,"operation":{"kind":"sense","organism_id":1,"extra":0}}',
```

</details>

<details><summary><code>test_receipts_and_snapshot_copies_are_independent()</code></summary>

Checks receipts and snapshot copies are independent.

[Source lines 271–276](C:/Thought/Projects/World/tests/test_native_resources.py:271)

```python
    def test_receipts_and_snapshot_copies_are_independent(self):
        self.both("grow", organism_id=2, tissue_nutrient=1)
        snapshot = self.native.snapshot()
        event = snapshot["events"][0]
        self.assertEqual(event["organisms"][0]["before"]["tissue"]["nutrient"], 8)
        self.assertEqual(event["organisms"][0]["after"]["tissue"]["nutrient"], 9)
```

</details>

<details><summary><code>test_seeded_mixed_requests_match_reference_and_preserve_rejected_state()</code></summary>

Checks seeded mixed requests match reference and preserve rejected state.

[Source lines 282–287](C:/Thought/Projects/World/tests/test_native_resources.py:282)

```python
    def test_seeded_mixed_requests_match_reference_and_preserve_rejected_state(self):
        rng = random.Random(73)
        for _ in range(200):
            kind, arguments = rng.choice([
                ("photosynthesize", {"organism_id": 1, "light_energy": rng.uniform(0.01, 3)}),
                ("absorb", {"organism_id": 1, "nutrient": rng.uniform(0.01, 0.5)}),
```

</details>

**Standalone functions**

<details><summary><code>single()</code></summary>

Create a balanced one-organism native inventory.

[Source lines 24–29](C:/Thought/Projects/World/tests/test_native_resources.py:24)

```python
def single(*, tissue=8.0, energy=20.0, nutrient=5.0, age=3.0, photosynthetic=False,
           can_ingest=True, weights=None, mineral=10.0, light=0.0, detritus=None):
    detritus = detritus or {"nutrient": 0.0, "energy": 0.0}
    return {"schema_version": 1, "initial_nutrient": mineral + tissue + nutrient + detritus["nutrient"],
            "initial_energy": 2 * tissue + energy + detritus["energy"], "mineral_nutrient": mineral,
            "light_available": light, "detritus": detritus,
```

</details>

**Useful syntax in this file**

- `**arguments` collects or expands named Python arguments.
- `as` gives an imported component a shorter or distinct local name, such as `np` or `NativeResourceLedger`.

</details>

<details><summary><code>crates/world_core/tests/resource_contract.rs</code></summary>

**Purpose:** Direct Rust checks for accounting, inheritance, mortality, and invalid inputs.

[Open the file](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use serde_json::{Value, json};
use world_core::{
    ids::OrganismId,
    resources::{AccountingError, Resources},
    rules::WorldRules,
    transactions::{Operation, ResourceLedger},
};
use world_genetics::genome::Genome;
```

- `serde_json`: native JSON scenarios, inspection output, and test assertions.
- `world_core`: the Python-independent native habitat and error types.
- `world_genetics`: the inherited producer blueprint, with no mutation or executing neural controller.

**Standalone functions**

<details><summary><code>initial()</code></summary>

Adapt independent lifecycle fixture fields into a native starting inventory.

[Source lines 13–18](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:13)

```rust
fn initial() -> Value {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let organisms = fixture["organisms"].as_array().unwrap().iter().enumerate().map(|(row, item)| json!({
        "id": row + 1,
        "genome": {"photosynthetic": item["photosynthetic"], "can_ingest": item["can_ingest"], "initial_weights": item["weights"]},
        "tissue": {"nutrient": item["tissue_nutrient"], "energy": item["tissue_energy"]},
```

</details>

<details><summary><code>ledger()</code></summary>

Construct a fresh native ledger from the shared rules and adapted inventory.

[Source lines 27–29](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:27)

```rust
fn ledger() -> ResourceLedger {
    ResourceLedger::from_json(RULES, &initial().to_string()).unwrap()
}
```

</details>

<details><summary><code>full_lifecycle_matches_independent_expected_snapshots_and_inheritance()</code></summary>

Checks full lifecycle matches independent expected snapshots and inheritance.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:32)

```rust
fn full_lifecycle_matches_independent_expected_snapshots_and_inheritance() {
    let rules = WorldRules::from_json(RULES).unwrap();
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let expected = fixture["expected_snapshots"].as_array().unwrap();
    let mut ledger = ledger();
    let mut snapshots = vec![ledger.summary_json("initial").unwrap()];
```

</details>

<details><summary><code>rejected_operations_leave_inventory_and_receipts_unchanged()</code></summary>

Checks rejected operations leave inventory and receipts unchanged.

[Source lines 123–128](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:123)

```rust
fn rejected_operations_leave_inventory_and_receipts_unchanged() {
    let mut ledger = ledger();
    for operation in [
        Operation::Feed {
            consumer_id: OrganismId(2),
            target_id: OrganismId(1),
```

</details>

<details><summary><code>starvation_commits_and_death_does_not_erase_mechanical_energy()</code></summary>

Checks starvation commits and death does not erase mechanical energy.

[Source lines 163–168](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:163)

```rust
fn starvation_commits_and_death_does_not_erase_mechanical_energy() {
    let mut ledger = ledger();
    ledger
        .apply(Operation::Actuate {
            organism_id: OrganismId(2),
            energy: 4.0,
```

</details>

<details><summary><code>malformed_initial_states_and_rule_investments_fail()</code></summary>

Checks malformed initial states and rule investments fail.

[Source lines 199–204](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:199)

```rust
fn malformed_initial_states_and_rule_investments_fail() {
    let mut value = initial();
    value["initial_nutrient"] = json!(101);
    assert!(matches!(
        ResourceLedger::from_json(RULES, &value.to_string()),
        Err(AccountingError::Invariant(_))
```

</details>

<details><summary><code>resource_arithmetic_and_inherited_parameters_reject_nonfinite_values()</code></summary>

Checks resource arithmetic and inherited parameters reject nonfinite values.

[Source lines 215–220](C:/Thought/Projects/World/crates/world_core/tests/resource_contract.rs:215)

```rust
fn resource_arithmetic_and_inherited_parameters_reject_nonfinite_values() {
    assert!(Resources::new(-1.0, 0.0).is_err());
    assert!(Resources::new(0.0, f64::INFINITY).is_err());
    assert!(
        Resources::new(0.0, 1e308)
            .unwrap()
```

</details>

**Useful syntax in this file**

- `#[test]` registers a Rust test function; assertions make the test fail on a mismatch.

</details>

<details><summary><code>docs/resource_accounting.md</code></summary>

**Purpose:** The API reference for constructing a ledger, submitting operations, and inspecting results.

[Open the file](C:/Thought/Projects/World/docs/resource_accounting.md)

This guide or specification has no executable imports, types, or functions. Use the file link to read its current contents.

</details>

## Step 5: Headless producer habitat

The habitat now decides where and when supported producer processes occur. Cells own resources; ledger pools are checked aggregate views. A native tick commits completely or remains unchanged. Founders are controlled stationary points, not evolved species.

<details><summary><code>crates/world_core/src/environment.rs</code></summary>

**Purpose:** Owns local cell resources and maps bounded positions to cells. Ledger totals are views of these cells, not additional resources.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/environment.rs)

**Imports / uses and why**

```rust
use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use crate::{
    audit::Sum,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    transactions::Pools,
};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.

**Struct: `Position`**

Handles Position at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 14–19](C:/Thought/Projects/World/crates/world_core/src/environment.rs:14)

```rust
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
```

**Struct: `CellOverride`**

Handles CellOverride at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 21–26](C:/Thought/Projects/World/crates/world_core/src/environment.rs:21)

```rust
pub struct CellOverride {
    pub column: u64,
    pub row: u64,
    pub mineral_nutrient: Option<f64>,
    pub light_energy_per_time: Option<f64>,
}
```

**Struct: `Cell`**

Handles Cell at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 30–35](C:/Thought/Projects/World/crates/world_core/src/environment.rs:30)

```rust
pub struct Cell {
    pub(crate) mineral_nutrient: f64,
    pub(crate) detritus: Resources,
    pub(crate) light_available: f64,
    pub(crate) light_energy_per_time: f64,
}
```

**Struct: `CellTotals`**

Handles CellTotals at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 60–65](C:/Thought/Projects/World/crates/world_core/src/environment.rs:60)

```rust
pub(crate) struct CellTotals {
    pub mineral: f64,
    pub detritus: Resources,
    pub light: f64,
    pub power: f64,
}
```

**Struct: `EnvironmentGrid`**

Handles EnvironmentGrid at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 69–74](C:/Thought/Projects/World/crates/world_core/src/environment.rs:69)

```rust
pub struct EnvironmentGrid {
    width: f64,
    height: f64,
    columns: usize,
    rows: usize,
    pub(crate) cells: Vec<Cell>,
```

**Methods in `impl Cell`**

<details><summary><code>mineral_nutrient()</code></summary>

Provides a read-only view of mineral nutrient for native observers and validation.

[Source lines 38–43](C:/Thought/Projects/World/crates/world_core/src/environment.rs:38)

```rust
    pub fn mineral_nutrient(&self) -> f64 {
        self.mineral_nutrient
    }
    pub fn detritus(&self) -> Resources {
        self.detritus
    }
```

</details>

<details><summary><code>detritus()</code></summary>

Provides a read-only view of detritus for native observers and validation.

[Source lines 41–46](C:/Thought/Projects/World/crates/world_core/src/environment.rs:41)

```rust
    pub fn detritus(&self) -> Resources {
        self.detritus
    }
    pub fn light_available(&self) -> f64 {
        self.light_available
    }
```

</details>

<details><summary><code>light_available()</code></summary>

Provides a read-only view of light available for native observers and validation.

[Source lines 44–49](C:/Thought/Projects/World/crates/world_core/src/environment.rs:44)

```rust
    pub fn light_available(&self) -> f64 {
        self.light_available
    }
    pub(crate) fn load_pools(&self, pools: &mut Pools) {
        pools.mineral_nutrient = self.mineral_nutrient;
        pools.detritus = self.detritus;
```

</details>

<details><summary><code>load_pools()</code></summary>

Temporarily give physiology this cell's environmental pools while retaining global heat, captured light, and mechanical energy.

[Source lines 47–52](C:/Thought/Projects/World/crates/world_core/src/environment.rs:47)

```rust
    pub(crate) fn load_pools(&self, pools: &mut Pools) {
        pools.mineral_nutrient = self.mineral_nutrient;
        pools.detritus = self.detritus;
        pools.light_available = self.light_available;
    }

```

</details>

<details><summary><code>store_pools()</code></summary>

Write the processed local pools back into this cell; final grid sums synchronize the ledger.

[Source lines 53–58](C:/Thought/Projects/World/crates/world_core/src/environment.rs:53)

```rust
    pub(crate) fn store_pools(&mut self, pools: Pools) {
        self.mineral_nutrient = pools.mineral_nutrient;
        self.detritus = pools.detritus;
        self.light_available = pools.light_available;
    }
}
```

</details>

**Methods in `impl EnvironmentGrid`**

<details><summary><code>validate()</code></summary>

Rejects malformed state or configuration before it can enter the running world.

[Source lines 78–83](C:/Thought/Projects/World/crates/world_core/src/environment.rs:78)

```rust
    pub(crate) fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.width != rules.world_width
            || self.height != rules.world_height
            || self.columns as u64 != rules.grid_width
            || self.rows as u64 != rules.grid_height
            || self.columns.checked_mul(self.rows) != Some(self.cells.len())
```

</details>

<details><summary><code>dimensions()</code></summary>

Provides a read-only view of dimensions for native observers and validation.

[Source lines 93–98](C:/Thought/Projects/World/crates/world_core/src/environment.rs:93)

```rust
    pub fn dimensions(&self) -> (f64, f64, usize, usize) {
        (self.width, self.height, self.columns, self.rows)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
```

</details>

<details><summary><code>cells()</code></summary>

Provides a read-only view of cells for native observers and validation.

[Source lines 97–102](C:/Thought/Projects/World/crates/world_core/src/environment.rs:97)

```rust
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub(crate) fn diffuse_minerals(&mut self, coefficient: f64, duration: f64) -> Result<()> {
        finite(coefficient, "mineral diffusivity", false)?;
```

</details>

<details><summary><code>diffuse_minerals()</code></summary>

Transfers minerals across adjacent cell faces with closed boundaries and stable substeps. Pairwise transfers conserve inventory and preserve nonnegative concentrations.

[Source lines 101–106](C:/Thought/Projects/World/crates/world_core/src/environment.rs:101)

```rust
    pub(crate) fn diffuse_minerals(&mut self, coefficient: f64, duration: f64) -> Result<()> {
        finite(coefficient, "mineral diffusivity", false)?;
        finite(duration, "diffusion duration", true)?;
        if coefficient == 0.0 {
            return Ok(());
        }
```

</details>

<details><summary><code>new()</code></summary>

Validate and construct the object. Habitat construction checks that declared budgets fund every founder and environmental cell.

[Source lines 157–162](C:/Thought/Projects/World/crates/world_core/src/environment.rs:157)

```rust
    pub fn new(
        rules: &WorldRules,
        minerals: f64,
        light_power: f64,
        light_gradient: f64,
        overrides: &[CellOverride],
```

</details>

<details><summary><code>cell_index()</code></summary>

Validate a position and map it to exactly one cell, including points on the outer walls.

[Source lines 230–235](C:/Thought/Projects/World/crates/world_core/src/environment.rs:230)

```rust
    pub fn cell_index(&self, position: Position) -> Result<usize> {
        finite(position.x, "position x", false)?;
        finite(position.y, "position y", false)?;
        if position.x > self.width || position.y > self.height {
            return Err(AccountingError::Invalid(
                "position is outside the habitat".into(),
```

</details>

<details><summary><code>totals()</code></summary>

Validate every cell and use compensated summation for its mineral, detrital, light, and light-power totals.

[Source lines 244–249](C:/Thought/Projects/World/crates/world_core/src/environment.rs:244)

```rust
    pub(crate) fn totals(&self) -> Result<CellTotals> {
        let mut sums: [Sum; 5] = std::array::from_fn(|_| Sum::default());
        for cell in &self.cells {
            cell.detritus.validate()?;
            for (sum, value) in sums.iter_mut().zip([
                cell.mineral_nutrient,
```

</details>

<details><summary><code>write_totals()</code></summary>

Rebuild the ledger environmental aggregates from the canonical cells before auditing the complete proposed state.

[Source lines 271–276](C:/Thought/Projects/World/crates/world_core/src/environment.rs:271)

```rust
    pub(crate) fn write_totals(&self, pools: &mut Pools) -> Result<()> {
        let totals = self.totals()?;
        pools.mineral_nutrient = totals.mineral;
        pools.detritus = totals.detritus;
        pools.light_available = totals.light;
        Ok(())
```

</details>

**Functions outside impl blocks**

<details><summary><code>diffusion_is_conservative_positive_and_has_closed_boundaries()</code></summary>

Handles diffusion is conservative positive and has closed boundaries at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 285–290](C:/Thought/Projects/World/crates/world_core/src/environment.rs:285)

```rust
    fn diffusion_is_conservative_positive_and_has_closed_boundaries() {
        let mut rules: WorldRules =
            serde_json::from_str(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        rules.world_width = 3.0;
        rules.world_height = 2.0;
        rules.grid_width = 3;
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/habitat_config.rs</code></summary>

**Purpose:** Defines funded producer/consumer founders, diagnostic ecological switches, and deterministic placement inputs.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs)

**Imports / uses and why**

```rust
use serde::{Deserialize, Serialize};
use crate::{
    environment::{CellOverride, Position},
    resources::{AccountingError, Result, finite},
    rules::WorldRules,
};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.

**Struct: `FounderPopulation`**

Handles FounderPopulation at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:11)

```rust
pub struct FounderPopulation {
    pub count: usize,
    pub tissue_nutrient: f64,
    pub reserve_nutrient: f64,
    pub reserve_energy: f64,
    #[serde(default)]
```

**Struct: `HabitatConfig`**

Handles HabitatConfig at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 24–29](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:24)

```rust
pub struct HabitatConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub initial_nutrient: f64,
    pub initial_energy: f64,
    pub mineral_nutrient_per_cell: f64,
```

**Struct: `EcologyConfig`**

Handles EcologyConfig at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 156–161](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:156)

```rust
pub struct EcologyConfig {
    pub reproduction: bool,
    pub decomposition: bool,
    pub consumer_speed: f64,
    pub movement_energy_per_distance: f64,
    pub perception_radius: f64,
```

**Methods in `impl HabitatConfig`**

<details><summary><code>from_json()</code></summary>

Decode the versioned JSON; habitat construction then applies rule-aware validation.

[Source lines 46–51](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:46)

```rust
    pub fn from_json(text: &str) -> Result<Self> {
        serde_json::from_str(text)
            .map_err(|error| AccountingError::Invalid(format!("invalid habitat config: {error}")))
    }

    pub fn validate(&self, rules: &WorldRules) -> Result<()> {
```

</details>

<details><summary><code>validate()</code></summary>

Reject unsupported schemas, invalid numeric resources, oversized reserves, or a mismatched explicit position list.

[Source lines 51–56](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:51)

```rust
    pub fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.schema_version != 1 {
            return Err(AccountingError::Invalid(
                "unsupported habitat schema".into(),
            ));
        }
```

</details>

**Methods in `impl FounderPopulation`**

<details><summary><code>validate()</code></summary>

Reject unsupported schemas, invalid numeric resources, oversized reserves, or a mismatched explicit position list.

[Source lines 116–121](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:116)

```rust
    fn validate(&self, rules: &WorldRules) -> Result<()> {
        if let Some(genome) = &self.genome {
            genome
                .validate()
                .map_err(|e| AccountingError::Invalid(e.into()))?;
        }
```

</details>

**Methods in `impl Default for EcologyConfig`**

<details><summary><code>default()</code></summary>

Supplies explicit diagnostic defaults while keeping optional processes disabled for older producer-only configurations.

[Source lines 172–177](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:172)

```rust
    fn default() -> Self {
        Self {
            reproduction: false,
            decomposition: false,
            consumer_speed: 1.3,
            movement_energy_per_distance: 0.1,
```

</details>

**Methods in `impl EcologyConfig`**

<details><summary><code>validate()</code></summary>

Reject unsupported schemas, invalid numeric resources, oversized reserves, or a mismatched explicit position list.

[Source lines 191–196](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:191)

```rust
    pub fn validate(&self, rules: &WorldRules) -> Result<()> {
        finite(
            self.mineral_diffusion_per_time,
            "mineral diffusivity",
            false,
        )?;
```

</details>

**Functions outside impl blocks**

<details><summary><code>seeded_position()</code></summary>

Map a seed and founder ordinal to deterministic x/y coordinates, using the recorded SplitMix64 counter layout.

[Source lines 237–242](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:237)

```rust
pub(crate) fn seeded_position(seed: u64, ordinal: u64, width: f64, height: f64) -> Position {
    fn unit(seed: u64, counter: u64) -> f64 {
        let mut value = seed.wrapping_add(counter.wrapping_add(1).wrapping_mul(0x9e3779b97f4a7c15));
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((value ^ (value >> 31)) >> 11) as f64 / 9_007_199_254_740_992.0
```

</details>

<details><summary><code>unit()</code></summary>

Use intentional wrapping integer arithmetic, XOR, and shifts to mix a counter; the top 53 bits produce a fraction in [0, 1).

[Source lines 238–243](C:/Thought/Projects/World/crates/world_core/src/habitat_config.rs:238)

```rust
    fn unit(seed: u64, counter: u64) -> f64 {
        let mut value = seed.wrapping_add(counter.wrapping_add(1).wrapping_mul(0x9e3779b97f4a7c15));
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((value ^ (value >> 31)) >> 11) as f64 / 9_007_199_254_740_992.0
    }
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/scheduler.rs</code></summary>

**Purpose:** Plans fixed native ticks and complete ecological windows without changing committed time.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use serde::{Deserialize, Serialize};
use crate::{
clock::Clock,
resources::{AccountingError, Result, finite},
rules::WorldRules,
};
use super::*;
```

- `serde`: typed data serialization/deserialization; strict records reject unknown fields.
- `crate`: neighboring modules of this Rust crate; existing accounting and validation are reused.

**Structure / enum: `Scheduler`**

The committed native tick and last completed ecological tick. A proposed next value is committed only after successful processing.

[Source lines 9–14](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:9)

```rust
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scheduler {
    tick: u64,
    last_ecology_tick: u64,
}
```

**Methods — open the one you are reading**

<details><summary><code>validate()</code></summary>

Rejects malformed state or configuration before it can enter the running world.

[Source lines 17–22](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:17)

```rust
    pub(crate) fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.last_ecology_tick
            != self.tick / rules.ecology_interval_ticks * rules.ecology_interval_ticks
        {
            return Err(AccountingError::Invariant(
                "clock window is inconsistent".into(),
```

</details>

<details><summary><code>tick()</code></summary>

Read the current native tick without performing another update.

[Source lines 32–34](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:32)

```rust
    pub fn tick(&self) -> u64 {
        self.tick
    }
```

</details>

<details><summary><code>last_ecology_tick()</code></summary>

Read the last fully processed ecological window boundary, keeping age updates separate from intermediate ticks.

[Source lines 36–38](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:36)

```rust
    pub fn last_ecology_tick(&self) -> u64 {
        self.last_ecology_tick
    }
```

</details>

<details><summary><code>next()</code></summary>

Plan a checked next tick and return a duration only at an ecological boundary. This does not mutate the current scheduler.

[Source lines 40–45](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:40)

```rust
    pub fn next(&self, rules: &WorldRules) -> Result<(Self, Option<f64>)> {
        finite(rules.physics_dt, "physics duration", true)?;
        if rules.ecology_interval_ticks == 0 {
            return Err(AccountingError::Invalid(
                "ecology interval must be positive".into(),
            ));
```

</details>

**Standalone functions**

<details><summary><code>overflow_and_nonfinite_time_do_not_change_the_clock()</code></summary>

Checks overflow and nonfinite time do not change the clock.

[Source lines 79–84](C:/Thought/Projects/World/crates/world_core/src/scheduler.rs:79)

```rust
    fn overflow_and_nonfinite_time_do_not_change_the_clock() {
        let mut rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        let scheduler = Scheduler {
            tick: u64::MAX,
            last_ecology_tick: 0,
```

</details>

**Useful syntax in this file**

- `pub(crate)` shares implementation data within this crate while keeping it out of the public API.
- `#[derive(...)]` generates standard implementations such as cloning, debugging, or serialization.
- `deny_unknown_fields` rejects misspelled or unsupported JSON fields rather than ignoring them.
- `?` returns a failure immediately; the enclosing staged operation can discard its candidate state.
- `Option<T>` represents a value that can be absent; `Result<T>` represents success or failure.
- `#[test]` registers a native check; failed assertions fail the test run.

</details>

<details><summary><code>crates/world_core/src/habitat.rs</code></summary>

**Purpose:** Owns the headless world and coordinates complete ecological windows, local inventories, resource audits, and atomic commits.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/habitat.rs)

**Imports / uses and why**

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use world_genetics::genome::Genome;
use crate::{
    audit::{AuditReport, Sum},
    ecology::{EcologyReport, EcologyState, EcologyWindow, initial_heading, reserve_for_birth},
    environment::{EnvironmentGrid, Position},
    habitat_config::{HabitatConfig, seeded_position},
    ids::OrganismId,
    organism::Organism,
    physiology,
    resources::{AccountingError, Resources, Result, finite},
    rules::WorldRules,
    scheduler::Scheduler,
    transactions::{Draft, LedgerInitial, Pools, ResourceLedger},
};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `WindowReport`**

Handles WindowReport at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 22–27](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:22)

```rust
pub(crate) struct WindowReport {
    pub(crate) tick: u64,
    pub(crate) duration: f64,
    pub(crate) producers_processed: usize,
    pub(crate) deaths: usize,
    pub(crate) light_captured: f64,
```

**Struct: `Habitat`**

Handles Habitat at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 58–63](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:58)

```rust
pub struct Habitat {
    pub(crate) rules: WorldRules,
    pub(crate) config: HabitatConfig,
    pub(crate) grid: EnvironmentGrid,
    pub(crate) positions: BTreeMap<OrganismId, Position>,
    pub(crate) ledger: ResourceLedger,
```

**Methods in `impl WindowReport`**

<details><summary><code>validate()</code></summary>

Rejects malformed state or configuration before it can enter the running world.

[Source lines 34–39](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:34)

```rust
    pub(crate) fn validate(&self, scheduler: Scheduler, rules: &WorldRules) -> Result<()> {
        if self.tick != scheduler.last_ecology_tick()
            || !rules.close(
                self.duration,
                rules.physics_dt * rules.ecology_interval_ticks as f64,
            )
```

</details>

**Methods in `impl Habitat`**

<details><summary><code>from_json()</code></summary>

Decode the versioned JSON; habitat construction then applies rule-aware validation.

[Source lines 73–78](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:73)

```rust
    pub fn from_json(rules_json: &str, config_json: &str) -> Result<Self> {
        let rules = WorldRules::from_json(rules_json)?;
        let config = HabitatConfig::from_json(config_json)?;
        Self::new(rules, config)
    }

```

</details>

<details><summary><code>new()</code></summary>

Validate and construct the object. Habitat construction checks that declared budgets fund every founder and environmental cell.

[Source lines 79–84](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:79)

```rust
    pub fn new(rules: WorldRules, config: HabitatConfig) -> Result<Self> {
        rules.validate()?;
        config.validate(&rules)?;
        let grid = EnvironmentGrid::new(
            &rules,
            config.mineral_nutrient_per_cell,
```

</details>

<details><summary><code>tick()</code></summary>

Read the current native tick without performing another update.

[Source lines 232–237](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:232)

```rust
    pub fn tick(&self) -> u64 {
        self.scheduler.tick()
    }

    pub fn rules(&self) -> &WorldRules {
        &self.rules
```

</details>

<details><summary><code>rules()</code></summary>

Borrows the world's validated coefficient contract.

[Source lines 236–241](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:236)

```rust
    pub fn rules(&self) -> &WorldRules {
        &self.rules
    }

    pub fn grid(&self) -> &EnvironmentGrid {
        &self.grid
```

</details>

<details><summary><code>grid()</code></summary>

Borrows the actual environment grid for read-only observation.

[Source lines 240–245](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:240)

```rust
    pub fn grid(&self) -> &EnvironmentGrid {
        &self.grid
    }

    pub fn organisms(&self) -> &BTreeMap<OrganismId, Organism> {
        self.ledger.organisms()
```

</details>

<details><summary><code>organisms()</code></summary>

Borrows organism records while keeping simulation mutation inside the core.

[Source lines 244–249](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:244)

```rust
    pub fn organisms(&self) -> &BTreeMap<OrganismId, Organism> {
        self.ledger.organisms()
    }

    pub fn positions(&self) -> &BTreeMap<OrganismId, Position> {
        &self.positions
```

</details>

<details><summary><code>positions()</code></summary>

Borrows exact organism positions for read-only rendering or inspection.

[Source lines 248–253](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:248)

```rust
    pub fn positions(&self) -> &BTreeMap<OrganismId, Position> {
        &self.positions
    }

    pub fn metrics(&self) -> &EcologyReport {
        &self.ecology.totals
```

</details>

<details><summary><code>metrics()</code></summary>

Borrows cumulative ecological counters for observers without allowing them to mutate the world.

[Source lines 252–257](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:252)

```rust
    pub fn metrics(&self) -> &EcologyReport {
        &self.ecology.totals
    }
    pub fn heading(&self, id: OrganismId) -> Option<f64> {
        self.ecology.headings.get(&id).copied()
    }
```

</details>

<details><summary><code>heading()</code></summary>

Borrows the native organism heading so rendering/picking need not serialize a complete world snapshot.

[Source lines 255–260](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:255)

```rust
    pub fn heading(&self, id: OrganismId) -> Option<f64> {
        self.ecology.headings.get(&id).copied()
    }

    pub fn advance(&mut self) -> Result<()> {
        if self.config.evolution.is_some() {
```

</details>

<details><summary><code>advance()</code></summary>

Stages an entire ecological window: light, recycling, producers, consumers, and births; audits it before committing all metadata and resources.

[Source lines 259–264](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:259)

```rust
    pub fn advance(&mut self) -> Result<()> {
        if self.config.evolution.is_some() {
            return Err(AccountingError::Invalid(
                "use the biological controller runtime to advance this habitat".into(),
            ));
        }
```

</details>

<details><summary><code>advance_impl()</code></summary>

Runs native clock/window work for either the original scripted control or the staged biological controller path.

[Source lines 268–273](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:268)

```rust
    pub(crate) fn advance_impl(
        &mut self,
        controls: Option<&crate::agents::Controls>,
    ) -> Result<()> {
        let (scheduler, window) = self.scheduler.next(&self.rules)?;
        if let Some(duration) = window {
```

</details>

<details><summary><code>audit()</code></summary>

Checks resources, local/global pool agreement, light, identity/ancestry, headings, capacity, and organism clock bounds.

[Source lines 399–404](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:399)

```rust
    pub fn audit(&self) -> Result<AuditReport> {
        let report = self.ledger.audit()?;
        self.rules.validate()?;
        self.config.validate(&self.rules)?;
        self.scheduler.validate(&self.rules)?;
        self.grid.validate(&self.rules)?;
```

</details>

<details><summary><code>audit_json()</code></summary>

Serialize the nutrient, chemical-energy, and light audit results for Python.

[Source lines 525–530](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:525)

```rust
    pub fn audit_json(&self) -> Result<String> {
        let report = self.audit()?;
        encode(
            &serde_json::json!({"nutrient":report.nutrient,"energy":report.energy,
            "nutrient_error":report.nutrient_error,"energy_error":report.energy_error,
            "light_error":light_balance(&self.rules,self.incident_light,self.escaped_light,self.ledger.pools())?}),
```

</details>

<details><summary><code>summary_json()</code></summary>

Return a compact inventory plus clock, seed, light counters, and the latest ecological receipt.

[Source lines 534–539](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:534)

```rust
    pub fn summary_json(&self) -> Result<String> {
        self.audit()?;
        let mut value = self.ledger.summary("habitat")?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| AccountingError::Invariant("summary is not an object".into()))?;
```

</details>

<details><summary><code>diagnostics()</code></summary>

Supports diagnostics within this component's responsibility; the source link opens the complete implementation.

[Source lines 553–558](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:553)

```rust
    pub fn diagnostics(&self) -> Option<&crate::diagnostics::Diagnostics> {
        self.diagnostics.as_ref()
    }

    pub fn snapshot_json(&self) -> Result<String> {
        self.audit()?;
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Return independent rules/configuration, cells, positions, organisms, aggregate pools, and time for inspection. Habitat restoration is not implemented.

[Source lines 557–562](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:557)

```rust
    pub fn snapshot_json(&self) -> Result<String> {
        self.audit()?;
        let organisms = self.ledger.organisms().values().map(|organism| {
            let position = self.positions[&organism.id];
            Ok(serde_json::json!({"organism":organism,"position":position,"cell_index":self.grid.cell_index(position)?}))
        }).collect::<Result<Vec<_>>>()?;
```

</details>

**Functions outside impl blocks**

<details><summary><code>process_producer()</code></summary>

Limit capture/uptake to the resident's initial cell shares, physiology rates, and reserve headroom. Pay maintenance before growth; retain starvation detritus locally.

[Source lines 574–579](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:574)

```rust
fn process_producer(
    draft: &mut Draft<'_>,
    id: OrganismId,
    light_share: f64,
    nutrient_share: f64,
    duration: f64,
```

</details>

<details><summary><code>light_balance()</code></summary>

Require incident light to equal absorbed light plus expired light plus current available light. Only absorbed light enters chemical-energy accounting.

[Source lines 621–626](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:621)

```rust
fn light_balance(rules: &WorldRules, incident: f64, escaped: f64, pools: Pools) -> Result<f64> {
    finite(incident, "incident light", false)?;
    finite(escaped, "escaped light", false)?;
    let mut accounted = Sum::default();
    for value in [pools.absorbed_light, pools.light_available, escaped] {
        accounted.add(value);
```

</details>

<details><summary><code>encode()</code></summary>

Serialize structured state as JSON and translate errors into the existing accounting error type.

[Source lines 636–638](C:/Thought/Projects/World/crates/world_core/src/habitat.rs:636)

```rust
fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|error| AccountingError::Invalid(error.to_string()))
}
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_bindings/src/habitat.rs</code></summary>

**Purpose:** Exposes native habitat construction, ticking, inspection, and complete save/load through PyO3.

[Open the file](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs)

**Imports / uses and why**

```rust
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use world_core::{habitat::Habitat, resources::AccountingError};
```

- `pyo3`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `PyHabitat`**

Handles PyHabitat at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 8–13](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:8)

```rust
pub struct PyHabitat {
    inner: Habitat,
}

#[pymethods]
impl PyHabitat {
```

**Methods in `impl PyHabitat`**

<details><summary><code>new()</code></summary>

Validate and construct the object. Habitat construction checks that declared budgets fund every founder and environmental cell.

[Source lines 15–20](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:15)

```rust
    fn new(rules_json: &str, config_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::from_json(rules_json, config_json).map_err(error)?,
        })
    }

```

</details>

<details><summary><code>tick()</code></summary>

Read the current native tick without performing another update.

[Source lines 22–27](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:22)

```rust
    fn tick(&self) -> u64 {
        self.inner.tick()
    }

    #[getter]
    fn decision_duration(&self) -> f64 {
```

</details>

<details><summary><code>decision_duration()</code></summary>

Reads the fixed biological decision duration directly, avoiding a complete observation/audit merely to pace the display loop.

[Source lines 27–32](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:27)

```rust
    fn decision_duration(&self) -> f64 {
        let rules = self.inner.rules();
        rules.physics_dt * rules.ecology_interval_ticks as f64
    }

    #[getter]
```

</details>

<details><summary><code>elapsed_time()</code></summary>

Reads authoritative elapsed model time for measured wall-clock speed; camera frames do not define biological time.

[Source lines 33–38](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:33)

```rust
    fn elapsed_time(&self) -> f64 {
        self.inner.tick() as f64 * self.inner.rules().physics_dt
    }

    #[getter]
    fn next_event(&self) -> u64 {
```

</details>

<details><summary><code>next_event()</code></summary>

Reads the monotonic event cursor without serializing the complete diagnostic history before every decision.

[Source lines 38–43](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:38)

```rust
    fn next_event(&self) -> u64 {
        self.inner.diagnostics().map_or(0, |d| d.next_event)
    }

    fn advance(&mut self) -> PyResult<()> {
        self.inner.advance().map_err(error)
```

</details>

<details><summary><code>advance()</code></summary>

Stages an entire ecological window: light, recycling, producers, consumers, and births; audits it before committing all metadata and resources.

[Source lines 42–47](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:42)

```rust
    fn advance(&mut self) -> PyResult<()> {
        self.inner.advance().map_err(error)
    }

    fn fork(&self) -> Self {
        Self {
```

</details>

<details><summary><code>fork()</code></summary>

Clones a native candidate without publishing it, enabling cross-language rollback if neural reconciliation later fails.

[Source lines 46–51](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:46)

```rust
    fn fork(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

```

</details>

<details><summary><code>agent_frame_json()</code></summary>

Supports agent frame json within this component's responsibility; the source link opens the complete implementation.

[Source lines 52–57](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:52)

```rust
    fn agent_frame_json(&self) -> PyResult<String> {
        self.inner.agent_frame_json().map_err(error)
    }

    fn events_since_json(&self, sequence: u64) -> PyResult<String> {
        self.inner.events_since_json(sequence).map_err(error)
```

</details>

<details><summary><code>events_since_json()</code></summary>

Exports the incremental event receipt, rejecting a future cursor or already-evicted events.

[Source lines 56–61](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:56)

```rust
    fn events_since_json(&self, sequence: u64) -> PyResult<String> {
        self.inner.events_since_json(sequence).map_err(error)
    }

    fn agent_frame_delta_json(&self, known: &str) -> PyResult<String> {
        self.inner.agent_frame_delta_json(known).map_err(error)
```

</details>

<details><summary><code>agent_frame_delta_json()</code></summary>

Omits unchanged neural genes by known organism/version identity while keeping current sensory observations.

[Source lines 60–65](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:60)

```rust
    fn agent_frame_delta_json(&self, known: &str) -> PyResult<String> {
        self.inner.agent_frame_delta_json(known).map_err(error)
    }

    #[pyo3(signature = (selected=None))]
    fn display_json(&self, selected: Option<u64>) -> PyResult<String> {
```

</details>

<details><summary><code>display_json()</code></summary>

Builds a small packet of live organisms, optional inspected dead organism, resource fields, and summary. Complete checkpoints use another API.

[Source lines 65–70](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:65)

```rust
    fn display_json(&self, selected: Option<u64>) -> PyResult<String> {
        self.inner.display_json(selected).map_err(error)
    }

    fn advance_controlled(&mut self, actions: &str) -> PyResult<()> {
        self.inner.advance_controlled(actions).map_err(error)
```

</details>

<details><summary><code>advance_controlled()</code></summary>

Validates the complete action cohort, advances a cloned native window, audits it, and replaces the world only after success.

[Source lines 69–74](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:69)

```rust
    fn advance_controlled(&mut self, actions: &str) -> PyResult<()> {
        self.inner.advance_controlled(actions).map_err(error)
    }

    fn audit_json(&self) -> PyResult<String> {
        self.inner.audit_json().map_err(error)
```

</details>

<details><summary><code>audit_json()</code></summary>

Serialize the nutrient, chemical-energy, and light audit results for Python.

[Source lines 73–78](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:73)

```rust
    fn audit_json(&self) -> PyResult<String> {
        self.inner.audit_json().map_err(error)
    }

    fn summary_json(&self) -> PyResult<String> {
        self.inner.summary_json().map_err(error)
```

</details>

<details><summary><code>summary_json()</code></summary>

Return a compact inventory plus clock, seed, light counters, and the latest ecological receipt.

[Source lines 77–82](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:77)

```rust
    fn summary_json(&self) -> PyResult<String> {
        self.inner.summary_json().map_err(error)
    }

    fn telemetry_json(&self) -> PyResult<String> {
        self.inner.telemetry_json().map_err(error)
```

</details>

<details><summary><code>telemetry_json()</code></summary>

Serializes the compact native diagnostic receipt for the Python runner and lightweight observer.

[Source lines 81–86](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:81)

```rust
    fn telemetry_json(&self) -> PyResult<String> {
        self.inner.telemetry_json().map_err(error)
    }

    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(error)
```

</details>

<details><summary><code>snapshot_json()</code></summary>

Return independent rules/configuration, cells, positions, organisms, aggregate pools, and time for inspection. Habitat restoration is not implemented.

[Source lines 85–90](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:85)

```rust
    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(error)
    }

    fn checkpoint_json(&self) -> PyResult<String> {
        self.inner.checkpoint_json().map_err(error)
```

</details>

<details><summary><code>checkpoint_json()</code></summary>

Returns the complete versioned native checkpoint payload.

[Source lines 89–94](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:89)

```rust
    fn checkpoint_json(&self) -> PyResult<String> {
        self.inner.checkpoint_json().map_err(error)
    }

    #[staticmethod]
    fn from_checkpoint_json(text: &str) -> PyResult<Self> {
```

</details>

<details><summary><code>from_checkpoint_json()</code></summary>

Creates a native habitat from the strict complete checkpoint format.

[Source lines 94–99](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:94)

```rust
    fn from_checkpoint_json(text: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::from_checkpoint_json(text).map_err(error)?,
        })
    }

```

</details>

<details><summary><code>save()</code></summary>

Saves the full world through the native atomic-file API, rather than exporting an inspection snapshot.

[Source lines 100–105](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:100)

```rust
    fn save(&self, path: &str) -> PyResult<()> {
        self.inner.save(path).map_err(error)
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
```

</details>

<details><summary><code>load()</code></summary>

Restores the full world without reseeding founders or resetting partially completed ecological windows.

[Source lines 105–110](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:105)

```rust
    fn load(path: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::load(path).map_err(error)?,
        })
    }
}
```

</details>

**Functions outside impl blocks**

<details><summary><code>error()</code></summary>

Translate invalid requests to Python ValueError and invariant failures to RuntimeError.

[Source lines 112–117](C:/Thought/Projects/World/crates/world_bindings/src/habitat.rs:112)

```rust
fn error(value: AccountingError) -> PyErr {
    match value {
        AccountingError::Invalid(message) => PyValueError::new_err(message),
        AccountingError::Invariant(message) => PyRuntimeError::new_err(message),
    }
}
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>python/world/habitat.py</code></summary>

**Purpose:** Provides the Python dictionary/path API while delegating simulation and complete checkpoint handling to Rust.

[Open the file](C:/Thought/Projects/World/python/world/habitat.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import json
from pathlib import Path
from ._native import Habitat as NativeHabitat
```

- `json`: Python dictionary/JSON conversion; it does not implement the simulation.
- `pathlib.Path`: readable filesystem paths for configurations, artifacts, and checkpoints.
- `NativeHabitat`: the installed Rust extension; Python wrappers delegate state changes to it.

**Class: `Habitat`**

The native habitat owns rules, scenario, grid, positions, ledger, scheduler, light counters, and latest receipt. The Python class forwards to that native owner.

[Source lines 7–9](C:/Thought/Projects/World/python/world/habitat.py:7)

```python
class Habitat:
    def __init__(self, rules: dict, config: dict):
        self._native = NativeHabitat(json.dumps(rules, allow_nan=False),
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Encode finite JSON inputs and construct the native habitat; the wrapper stores no duplicate biological state.

[Source lines 8–10](C:/Thought/Projects/World/python/world/habitat.py:8)

```python
    def __init__(self, rules: dict, config: dict):
        self._native = NativeHabitat(json.dumps(rules, allow_nan=False),
                                     json.dumps(config, allow_nan=False))
```

</details>

<details><summary><code>tick()</code></summary>

Read the current native tick without performing another update.

[Source lines 12–14](C:/Thought/Projects/World/python/world/habitat.py:12)

```python
    @property
    def tick(self) -> int:
        return self._native.tick
```

</details>

<details><summary><code>advance()</code></summary>

Advance native time. At ecological boundaries, stage light renewal and local producer processes; commit only after audits pass. The Python wrapper repeats successful native ticks.

[Source lines 16–20](C:/Thought/Projects/World/python/world/habitat.py:16)

```python
    def advance(self, ticks: int = 1):
        if type(ticks) is not int or not 0 <= ticks <= 2**64 - 1:
            raise ValueError("ticks must be an unsigned 64-bit integer")
        for _ in range(ticks):
            self._native.advance()
```

</details>

<details><summary><code>audit()</code></summary>

Check the ledger, position coverage, cell/aggregate agreement, and incident-light identity without counting cells twice.

[Source lines 22–23](C:/Thought/Projects/World/python/world/habitat.py:22)

```python
    def audit(self) -> dict:
        return json.loads(self._native.audit_json())
```

</details>

<details><summary><code>summary()</code></summary>

Decode a fresh native inventory summary into ordinary Python data.

[Source lines 25–26](C:/Thought/Projects/World/python/world/habitat.py:25)

```python
    def summary(self) -> dict:
        return json.loads(self._native.summary_json())
```

</details>

<details><summary><code>snapshot()</code></summary>

Decode an independent inspection copy; editing it cannot change native cell or organism state.

[Source lines 28–29](C:/Thought/Projects/World/python/world/habitat.py:28)

```python
    def snapshot(self) -> dict:
        return json.loads(self._native.snapshot_json())
```

</details>

<details><summary><code>save()</code></summary>

Normalizes a Python path and delegates the complete save to Rust.

[Source lines 31–32](C:/Thought/Projects/World/python/world/habitat.py:31)

```python
    def save(self, path: str | Path):
        self._native.save(str(Path(path)))
```

</details>

<details><summary><code>load()</code></summary>

Builds a Python wrapper around a loaded native world without calling the founder constructor.

[Source lines 34–38](C:/Thought/Projects/World/python/world/habitat.py:34)

```python
    @classmethod
    def load(cls, path: str | Path):
        result = cls.__new__(cls)
        result._native = NativeHabitat.load(str(Path(path)))
        return result
```

</details>

**Useful syntax in this file**

- `@classmethod` receives the class as `cls`, allowing restoration without running the founder initializer.
- `__new__` allocates the wrapper before attaching the restored native world; `__init__` would create a fresh one.

</details>

<details><summary><code>configs/producer_habitat.json</code></summary>

**Purpose:** Funds 64 controlled producers and the environmental grid; sets seed, cell light power, and its horizontal gradient.

[Open the file](C:/Thought/Projects/World/configs/producer_habitat.json)

[Source lines 1–7](C:/Thought/Projects/World/configs/producer_habitat.json:1)

```json
{
  "schema_version": 1,
  "seed": 73,
  "initial_nutrient": 4256.0,
  "initial_energy": 576.0,
  "mineral_nutrient_per_cell": 1.0,
  "light_energy_per_cell_per_time": 8.0,
```

Budgets cover the grid and founders together. `light_gradient` sets the horizontal light field. Optional `positions` and `cell_overrides` allow explicit local scenarios. These are configuration values, not organism instructions.

</details>

<details><summary><code>examples/habitat_demo.py</code></summary>

**Purpose:** Runs either habitat preset headlessly, prints sampled counters/audits, and saves or resumes complete worlds.

[Open the file](C:/Thought/Projects/World/examples/habitat_demo.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import argparse
import json
from pathlib import Path
from world import Habitat
```

- `json`: Python dictionary/JSON conversion; it does not implement the simulation.
- `pathlib.Path`: readable filesystem paths for configurations, artifacts, and checkpoints.
- `argparse`: command-line options and input error reporting.

**Standalone functions**

<details><summary><code>main()</code></summary>

Loads or initializes a habitat, runs additional ticks in sample chunks, prints audits, and optionally writes inspection output and a complete checkpoint.

[Source lines 11–16](C:/Thought/Projects/World/examples/habitat_demo.py:11)

```python
def main():
    parser = argparse.ArgumentParser(description="Run a headless diagnostic habitat.")
    parser.add_argument("--rules", type=Path, default=ROOT / "configs/world_rules_v0.json")
    parser.add_argument("--config", type=Path, default=ROOT / "configs/producer_habitat.json")
    parser.add_argument("--ticks", type=int, default=3600)
    parser.add_argument("--sample-every", type=int, default=600)
```

</details>

</details>

<details><summary><code>tests/test_habitat.py</code></summary>

**Purpose:** Checks local resource access, shared inputs, scheduling, repeatability, hand-calculated inventories, and atomic failure through Python.

[Open the file](C:/Thought/Projects/World/tests/test_habitat.py)

**Imports / uses — why they are here**

Shown together here, including local imports.

```python
import json
import math
from pathlib import Path
import unittest
from world import Habitat
from world._native import Habitat as NativeHabitat
```

- `json`: conversion between dictionaries and the native JSON interface.
- `math`: independent finite-value checks and inventory sums in tests.
- `pathlib`: configuration and output paths.
- `unittest`: fresh test cases, assertions, and expected exceptions.
- `world`: the installed public Habitat interface used by examples and tests.
- `world._native`: the raw native boundary, tested independently of wrapper JSON validation.

**Module-level values**

[Source lines 10–10](C:/Thought/Projects/World/tests/test_habitat.py:10)

```python
ROOT = Path(__file__).resolve().parents[1]
```

**Module-level values**

[Source lines 11–11](C:/Thought/Projects/World/tests/test_habitat.py:11)

```python
RULES = json.loads((ROOT / "configs/world_rules_v0.json").read_text(encoding="utf-8"))
```

**Module-level values**

[Source lines 12–12](C:/Thought/Projects/World/tests/test_habitat.py:12)

```python
PRESET = json.loads((ROOT / "configs/producer_habitat.json").read_text(encoding="utf-8"))
```

**Class: `HabitatTests`**

Independent Python test cases with fresh habitat configurations and assertions about local/global behavior.

[Source lines 29–29](C:/Thought/Projects/World/tests/test_habitat.py:29)

```python
class HabitatTests(unittest.TestCase):
```

**Methods — open the one you are reading**

<details><summary><code>test_preset_is_funded_and_seed_reproduces_initial_positions()</code></summary>

Checks preset is funded and seed reproduces initial positions.

[Source lines 30–35](C:/Thought/Projects/World/tests/test_habitat.py:30)

```python
    def test_preset_is_funded_and_seed_reproduces_initial_positions(self):
        first = Habitat(RULES, PRESET)
        second = Habitat(RULES, PRESET)
        self.assertEqual(first.snapshot(), second.snapshot())
        self.assertEqual(first.summary()["living_count"], 64)
        self.assertEqual(first.summary()["total_nutrient"], 4256)
```

</details>

<details><summary><code>test_seed_layout_has_a_stable_versioned_example()</code></summary>

Checks seed layout has a stable versioned example.

[Source lines 45–50](C:/Thought/Projects/World/tests/test_habitat.py:45)

```python
    def test_seed_layout_has_a_stable_versioned_example(self):
        config = small_config()
        config["producers"].pop("positions")
        position = Habitat(small_rules(), config).snapshot()["organisms"][0]["position"]
        self.assertEqual(position, {"x": 1.7666216164272852, "y": 0.43152799704850997})
        config["seed"] = 2**53 + 1
```

</details>

<details><summary><code>test_ecology_runs_once_per_window_and_ages_once()</code></summary>

Checks ecology runs once per window and ages once.

[Source lines 53–58](C:/Thought/Projects/World/tests/test_habitat.py:53)

```python
    def test_ecology_runs_once_per_window_and_ages_once(self):
        habitat = Habitat(RULES, PRESET)
        habitat.advance(5)
        self.assertEqual(habitat.tick, 5)
        self.assertIsNone(habitat.summary()["last_update"])
        self.assertTrue(all(item["organism"]["age"] == 0 for item in habitat.snapshot()["organisms"]))
```

</details>

<details><summary><code>test_one_window_matches_independent_hand_calculation()</code></summary>

Checks one window matches independent hand calculation.

[Source lines 68–73](C:/Thought/Projects/World/tests/test_habitat.py:68)

```python
    def test_one_window_matches_independent_hand_calculation(self):
        habitat = Habitat(small_rules(), small_config())
        habitat.advance(2)
        row = habitat.summary()
        for key, expected in {"total_nutrient": 4.5, "mineral_nutrient": 1,
                              "living_nutrient": 3.5, "living_energy": 12.8,
```

</details>

<details><summary><code>test_remote_minerals_cannot_fund_local_growth()</code></summary>

Checks remote minerals cannot fund local growth.

[Source lines 83–88](C:/Thought/Projects/World/tests/test_habitat.py:83)

```python
    def test_remote_minerals_cannot_fund_local_growth(self):
        config = small_config()
        config["producers"]["reserve_nutrient"] = 0
        config["cell_overrides"] = [{"column": 0, "row": 0, "mineral_nutrient": 0},
                                     {"column": 1, "row": 0, "mineral_nutrient": 10}]
        config["initial_nutrient"] = 12
```

</details>

<details><summary><code>test_dark_cell_cannot_capture_remote_light_and_death_stays_local()</code></summary>

Checks dark cell cannot capture remote light and death stays local.

[Source lines 96–101](C:/Thought/Projects/World/tests/test_habitat.py:96)

```python
    def test_dark_cell_cannot_capture_remote_light_and_death_stays_local(self):
        config = small_config()
        config["producers"].update(reserve_nutrient=0, reserve_energy=0)
        config.update(initial_nutrient=3, initial_energy=4,
                      cell_overrides=[{"column": 0, "row": 0,
                                       "mineral_nutrient": 0, "light_energy_per_time": 0}])
```

</details>

<details><summary><code>test_cell_shares_do_not_duplicate_light_or_uptake()</code></summary>

Checks cell shares do not duplicate light or uptake.

[Source lines 114–119](C:/Thought/Projects/World/tests/test_habitat.py:114)

```python
    def test_cell_shares_do_not_duplicate_light_or_uptake(self):
        config = small_config()
        config.update(initial_nutrient=6, initial_energy=8, light_energy_per_cell_per_time=2)
        config["producers"].update(count=2, reserve_nutrient=0, reserve_energy=0,
            positions=[{"x": 0.25, "y": 0.5}, {"x": 0.75, "y": 0.5}])
        habitat = Habitat(small_rules(), config)
```

</details>

<details><summary><code>test_unabsorbed_light_expires_and_has_an_explicit_budget()</code></summary>

Checks unabsorbed light expires and has an explicit budget.

[Source lines 130–135](C:/Thought/Projects/World/tests/test_habitat.py:130)

```python
    def test_unabsorbed_light_expires_and_has_an_explicit_budget(self):
        habitat = Habitat(small_rules(), small_config())
        habitat.advance(4)
        row = habitat.summary()
        self.assertEqual(row["incident_light"], 32)
        self.assertAlmostEqual(row["escaped_light"], 9.75)
```

</details>

<details><summary><code>test_rate_caps_use_the_actual_window()</code></summary>

Checks rate caps use the actual window.

[Source lines 139–144](C:/Thought/Projects/World/tests/test_habitat.py:139)

```python
    def test_rate_caps_use_the_actual_window(self):
        rules = {**small_rules(), "physics_dt": 0.05}
        config = small_config()
        config.update(light_energy_per_cell_per_time=1000, mineral_nutrient_per_cell=10,
                      initial_nutrient=22, initial_energy=4)
        config["producers"].update(reserve_energy=0, reserve_nutrient=0)
```

</details>

<details><summary><code>test_chunking_and_repeated_runs_produce_identical_states()</code></summary>

Checks chunking and repeated runs produce identical states.

[Source lines 153–158](C:/Thought/Projects/World/tests/test_habitat.py:153)

```python
    def test_chunking_and_repeated_runs_produce_identical_states(self):
        first = Habitat(small_rules(), small_config())
        second = Habitat(small_rules(), small_config())
        first.advance(120)
        for _ in range(120):
            second.advance()
```

</details>

<details><summary><code>test_empty_world_advances_without_respawning()</code></summary>

Checks empty world advances without respawning.

[Source lines 162–167](C:/Thought/Projects/World/tests/test_habitat.py:162)

```python
    def test_empty_world_advances_without_respawning(self):
        config = small_config()
        config.update(initial_nutrient=0, initial_energy=0, mineral_nutrient_per_cell=0,
                      light_energy_per_cell_per_time=2)
        config["producers"].update(count=0, positions=[])
        habitat = Habitat(small_rules(), config)
```

</details>

<details><summary><code>test_position_boundaries_and_invalid_positions()</code></summary>

Checks position boundaries and invalid positions.

[Source lines 175–180](C:/Thought/Projects/World/tests/test_habitat.py:175)

```python
    def test_position_boundaries_and_invalid_positions(self):
        for position, cell in [({"x": 0, "y": 0}, 0), ({"x": 1, "y": 0}, 1),
                               ({"x": 2, "y": 1}, 1)]:
            config = small_config()
            config["producers"]["positions"] = [position]
            self.assertEqual(Habitat(small_rules(), config).snapshot()["organisms"][0]["cell_index"], cell)
```

</details>

<details><summary><code>test_malformed_configs_and_unfunded_founders_are_rejected()</code></summary>

Checks malformed configs and unfunded founders are rejected.

[Source lines 188–193](C:/Thought/Projects/World/tests/test_habitat.py:188)

```python
    def test_malformed_configs_and_unfunded_founders_are_rejected(self):
        for update in [{"schema_version": 2}, {"seed": True}, {"seed": 2**64},
                       {"light_gradient": 1.1}, {"mineral_nutrient_per_cell": -1},
                       {"light_energy_per_cell_per_time": math.inf}, {"unknown": 1},
                       {"cell_overrides": [{"column": 2, "row": 0, "mineral_nutrient": 0}]},
                       {"cell_overrides": [{"column": 0, "row": 0}]},
```

</details>

<details><summary><code>test_invalid_advance_and_snapshot_edits_do_not_change_native_state()</code></summary>

Checks invalid advance and snapshot edits do not change native state.

[Source lines 206–211](C:/Thought/Projects/World/tests/test_habitat.py:206)

```python
    def test_invalid_advance_and_snapshot_edits_do_not_change_native_state(self):
        habitat = Habitat(small_rules(), small_config())
        before = habitat.snapshot()
        for count in [-1, True, 1.5, "1", 2**64]:
            with self.subTest(count=count), self.assertRaises(ValueError):
                habitat.advance(count)
```

</details>

<details><summary><code>test_light_overflow_rejects_the_whole_tick()</code></summary>

Checks light overflow rejects the whole tick.

[Source lines 219–224](C:/Thought/Projects/World/tests/test_habitat.py:219)

```python
    def test_light_overflow_rejects_the_whole_tick(self):
        rules = {**small_rules(), "grid_width": 1, "physics_dt": 1, "ecology_interval_ticks": 1}
        config = small_config()
        config.update(initial_nutrient=0, initial_energy=0, mineral_nutrient_per_cell=0,
                      light_energy_per_cell_per_time=1e308)
        config["producers"].update(count=0, positions=[])
```

</details>

<details><summary><code>test_failed_processing_rolls_back_cells_age_and_clock()</code></summary>

Checks failed processing rolls back cells age and clock.

[Source lines 233–238](C:/Thought/Projects/World/tests/test_habitat.py:233)

```python
    def test_failed_processing_rolls_back_cells_age_and_clock(self):
        rules = {**small_rules(), "physics_dt": 1e308, "ecology_interval_ticks": 1,
                 "maintenance_per_tissue_nutrient_per_time": 1e-308,
                 "max_growth_nutrient_per_time": 1e308}
        config = {**small_config(), "light_energy_per_cell_per_time": 0}
        habitat = Habitat(rules, config)
```

</details>

<details><summary><code>test_raw_native_boundary_rejects_nonfinite_and_unknown_json()</code></summary>

Checks raw native boundary rejects nonfinite and unknown json.

[Source lines 245–250](C:/Thought/Projects/World/tests/test_habitat.py:245)

```python
    def test_raw_native_boundary_rejects_nonfinite_and_unknown_json(self):
        for text in [json.dumps({**small_config(), "unknown": 1}),
                     json.dumps({**small_config(), "seed": 1.5}),
                     json.dumps({**small_config(), "initial_nutrient": math.nan})]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                NativeHabitat(json.dumps(small_rules()), text)
```

</details>

<details><summary><code>test_default_run_keeps_global_and_cell_inventories_balanced()</code></summary>

Checks default run keeps global and cell inventories balanced.

[Source lines 252–257](C:/Thought/Projects/World/tests/test_habitat.py:252)

```python
    def test_default_run_keeps_global_and_cell_inventories_balanced(self):
        habitat = Habitat(RULES, PRESET)
        habitat.advance(600)
        state = habitat.snapshot()
        cells = state["environment"]["cells"]
        organisms = [item["organism"] for item in state["organisms"]]
```

</details>

**Standalone functions**

<details><summary><code>small_rules()</code></summary>

Create a two-cell test world with a one-unit ecological duration so expected results can be calculated by hand.

[Source lines 15–18](C:/Thought/Projects/World/tests/test_habitat.py:15)

```python
def small_rules():
    return {**RULES, "world_width": 2.0, "world_height": 1.0,
            "grid_width": 2, "grid_height": 1, "physics_dt": 0.5,
            "ecology_interval_ticks": 2}
```

</details>

<details><summary><code>small_config()</code></summary>

Define the independently funded one-producer test case: 4.5 N and 9 E.

[Source lines 21–26](C:/Thought/Projects/World/tests/test_habitat.py:21)

```python
def small_config():
    return {"schema_version": 1, "seed": 0, "initial_nutrient": 4.5, "initial_energy": 9.0,
            "mineral_nutrient_per_cell": 1.0, "light_energy_per_cell_per_time": 8.0,
            "light_gradient": 0.0, "producers": {"count": 1, "tissue_nutrient": 2.0,
                "reserve_nutrient": 0.5, "reserve_energy": 5.0,
                "positions": [{"x": 0.25, "y": 0.5}]}}
```

</details>

</details>

<details><summary><code>crates/world_core/tests/habitat_contract.rs</code></summary>

**Purpose:** Checks habitat accounting, locality, geometry, deterministic execution, and rollback directly in Rust.

[Open the file](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs)

**Imports / uses — why they are here**

Shown together here, including local imports.

```rust
use serde_json::{Value, json};
use world_core::{
    environment::{EnvironmentGrid, Position},
    habitat::Habitat,
    resources::AccountingError,
    rules::WorldRules,
};
```

- `serde_json`: native JSON scenarios, inspection output, and test assertions.
- `world_core`: the Python-independent native habitat and error types.

**Standalone functions**

<details><summary><code>rules()</code></summary>

Build the equivalent small native-test rules, keeping expected timing and geometry explicit.

[Source lines 12–17](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:12)

```rust
fn rules() -> Value {
    let mut rules: Value = serde_json::from_str(RULES).unwrap();
    for (name, value) in [
        ("world_width", json!(2)),
        ("world_height", json!(1)),
        ("grid_width", json!(2)),
```

</details>

<details><summary><code>config()</code></summary>

Build the independently funded initial inventory used by native habitat tests.

[Source lines 27–32](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:27)

```rust
fn config() -> Value {
    json!({"schema_version":1,"seed":0,"initial_nutrient":4.5,"initial_energy":9,
        "mineral_nutrient_per_cell":1,"light_energy_per_cell_per_time":8,"light_gradient":0,
        "producers":{"count":1,"tissue_nutrient":2,"reserve_nutrient":0.5,"reserve_energy":5,
            "positions":[{"x":0.25,"y":0.5}]}})
}
```

</details>

<details><summary><code>summary()</code></summary>

Decode a fresh native inventory summary into ordinary Python data.

[Source lines 34–36](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:34)

```rust
fn summary(habitat: &Habitat) -> Value {
    serde_json::from_str(&habitat.summary_json().unwrap()).unwrap()
}
```

</details>

<details><summary><code>preset_founders_are_funded_and_repeatable()</code></summary>

Checks preset founders are funded and repeatable.

[Source lines 39–44](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:39)

```rust
fn preset_founders_are_funded_and_repeatable() {
    let first = Habitat::from_json(RULES, PRESET).unwrap();
    let second = Habitat::from_json(RULES, PRESET).unwrap();
    assert_eq!(
        first.snapshot_json().unwrap(),
        second.snapshot_json().unwrap()
```

</details>

<details><summary><code>local_window_matches_hand_calculated_inventories()</code></summary>

Checks local window matches hand calculated inventories.

[Source lines 52–57](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:52)

```rust
fn local_window_matches_hand_calculated_inventories() {
    let mut habitat = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    habitat.advance().unwrap();
    assert_eq!(summary(&habitat)["absorbed_light"], 0.0);
    habitat.advance().unwrap();
    let result = summary(&habitat);
```

</details>

<details><summary><code>colocated_producers_share_only_their_cell_inputs()</code></summary>

Checks colocated producers share only their cell inputs.

[Source lines 80–85](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:80)

```rust
fn colocated_producers_share_only_their_cell_inputs() {
    let mut config = config();
    config["initial_nutrient"] = json!(6);
    config["initial_energy"] = json!(8);
    config["light_energy_per_cell_per_time"] = json!(2);
    config["producers"]["count"] = json!(2);
```

</details>

<details><summary><code>starvation_returns_tissue_to_the_occupied_cell_once()</code></summary>

Checks starvation returns tissue to the occupied cell once.

[Source lines 102–107](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:102)

```rust
fn starvation_returns_tissue_to_the_occupied_cell_once() {
    let mut config = config();
    config["initial_nutrient"] = json!(3);
    config["initial_energy"] = json!(4);
    config["producers"]["reserve_nutrient"] = json!(0);
    config["producers"]["reserve_energy"] = json!(0);
```

</details>

<details><summary><code>world_boundaries_map_to_valid_cells_and_reject_outside_points()</code></summary>

Checks world boundaries map to valid cells and reject outside points.

[Source lines 128–133](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:128)

```rust
fn world_boundaries_map_to_valid_cells_and_reject_outside_points() {
    let rules = WorldRules::from_json(&rules().to_string()).unwrap();
    let grid = EnvironmentGrid::new(&rules, 1.0, 8.0, 0.0, &[]).unwrap();
    assert_eq!(grid.cell_index(Position { x: 0.0, y: 0.0 }).unwrap(), 0);
    assert_eq!(grid.cell_index(Position { x: 2.0, y: 1.0 }).unwrap(), 1);
    for point in [
```

</details>

<details><summary><code>invalid_starting_budgets_and_overrides_are_rejected()</code></summary>

Checks invalid starting budgets and overrides are rejected.

[Source lines 146–151](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:146)

```rust
fn invalid_starting_budgets_and_overrides_are_rejected() {
    let mut invalid = config();
    invalid["initial_nutrient"] = json!(100);
    assert!(matches!(
        Habitat::from_json(&rules().to_string(), &invalid.to_string()),
        Err(AccountingError::Invariant(_))
```

</details>

<details><summary><code>failed_processing_discards_staged_cells_organisms_and_clock()</code></summary>

Checks failed processing discards staged cells organisms and clock.

[Source lines 159–164](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:159)

```rust
fn failed_processing_discards_staged_cells_organisms_and_clock() {
    let mut rules = rules();
    rules["physics_dt"] = json!(1e308);
    rules["ecology_interval_ticks"] = json!(1);
    rules["maintenance_per_tissue_nutrient_per_time"] = json!(1e-308);
    rules["max_growth_nutrient_per_time"] = json!(1e308);
```

</details>

<details><summary><code>repeated_scheduled_runs_and_light_budgets_agree()</code></summary>

Checks repeated scheduled runs and light budgets agree.

[Source lines 175–180](C:/Thought/Projects/World/crates/world_core/tests/habitat_contract.rs:175)

```rust
fn repeated_scheduled_runs_and_light_budgets_agree() {
    let mut first = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    let mut second = Habitat::from_json(&rules().to_string(), &config().to_string()).unwrap();
    for _ in 0..120 {
        first.advance().unwrap();
        second.advance().unwrap();
```

</details>

**Useful syntax in this file**

- `#[test]` registers native checks; failed assertions fail the test.

</details>

## Step 6: Diagnostic ecological cycle

The producer habitat now includes local consumers, paid movement/feeding, funded clone births, death, and aggregate recycling. The existing habitat/configuration/physiology/binding files above show their current shared implementation. Fixed diagnostic behavior makes accounting testable before ANN control and mutation.

<details><summary><code>configs/ecosystem_rules.json</code></summary>

**Purpose:** Uses the existing biological coefficients with a denser 24 x 24 habitat/grid for inspectable local interactions.

[Open the file](C:/Thought/Projects/World/configs/ecosystem_rules.json)

[Source lines 1–8](C:/Thought/Projects/World/configs/ecosystem_rules.json:1)

```json
{
  "schema_version": 1,
  "model_id": "world-rules-v0.1",
  "world_width": 24.0,
  "world_height": 24.0,
  "grid_width": 24,
  "grid_height": 24,
  "physics_dt": 0.016666666666666666,
```

This is declarative data, so it has no runtime imports, classes, or functions. Named fields become validated configuration values. `true`/`false` are process switches; numeric amounts and radii use the documented model units.

</details>

<details><summary><code>configs/ecosystem_habitat.json</code></summary>

**Purpose:** Funds 64 producers and 16 consumers and enables the diagnostic reproduction/recycling cycle with explicit movement and placement limits.

[Open the file](C:/Thought/Projects/World/configs/ecosystem_habitat.json)

[Source lines 1–8](C:/Thought/Projects/World/configs/ecosystem_habitat.json:1)

```json
{
  "schema_version": 1,
  "seed": 73,
  "initial_nutrient": 4296.0,
  "initial_energy": 3136.0,
  "mineral_nutrient_per_cell": 6.0,
  "light_energy_per_cell_per_time": 8.0,
  "light_gradient": 0.5,
```

This is declarative data, so it has no runtime imports, classes, or functions. Named fields become validated configuration values. `true`/`false` are process switches; numeric amounts and radii use the documented model units.

</details>

<details><summary><code>crates/world_core/src/ecology.rs</code></summary>

**Purpose:** Runs the fixed diagnostic consumer policy, funded reproduction, and aggregate recycling inside a staged habitat window.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/ecology.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::{
collections::BTreeMap,
f64::consts::{PI, TAU},
};
use serde::{Deserialize, Serialize};
use crate::{
environment::{EnvironmentGrid, Position},
habitat_config::{EcologyConfig, seeded_position},
ids::OrganismId,
lifecycle, physiology,
resources::{AccountingError, Resources, Result, finite},
spatial::{SpatialIndex, distance, segment_distance},
transactions::Draft,
};
```

- `std`: standard-library maps, paths, files, arithmetic, and process utilities; no added crate is needed for these.
- `serde`: typed data serialization/deserialization; strict records reject unknown fields.
- `crate`: neighboring modules of this Rust crate; existing accounting and validation are reused.

**Structure / enum: `EcologyReport`**

Stores cumulative or per-window counts and resource quantities; it is an observation record, not another resource pool.

[Source lines 18–25](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:18)

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EcologyReport {
    pub consumers_processed: u64,
    pub births: u64,
    pub deaths: u64,
    pub feeding_events: u64,
    pub ingested_nutrient: f64,
```

**Methods — open the one you are reading**

<details><summary><code>validate()</code></summary>

Rejects non-finite or negative measured quantities before a receipt or checkpoint is accepted.

[Source lines 33–38](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:33)

```rust
    pub(crate) fn validate(&self) -> Result<()> {
        for value in [
            self.ingested_nutrient,
            self.tissue_grown,
            self.distance_moved,
            self.decomposed_nutrient,
```

</details>

<details><summary><code>accumulate()</code></summary>

Adds one accepted window to cumulative metrics with checked integer increments and finite float validation.

[Source lines 46–51](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:46)

```rust
    pub(crate) fn accumulate(&mut self, window: &Self) -> Result<()> {
        for (total, increment) in [
            (&mut self.consumers_processed, window.consumers_processed),
            (&mut self.births, window.births),
            (&mut self.deaths, window.deaths),
            (&mut self.feeding_events, window.feeding_events),
```

</details>

**Structure / enum: `EcologyState`**

Retains consumer headings, the next unused offspring identity, and cumulative metrics across windows and checkpoints.

[Source lines 66–71](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:66)

```rust
#[derive(Clone, Debug)]
pub(crate) struct EcologyState {
    pub headings: BTreeMap<OrganismId, f64>,
    pub next_id: u64,
    pub totals: EcologyReport,
}
```

**Methods — open the one you are reading**

<details><summary><code>new()</code></summary>

Starts the ecological state with no consumer headings, an unused ID, and zero metrics.

[Source lines 82–87](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:82)

```rust
    pub fn new(next_id: u64) -> Self {
        Self {
            headings: BTreeMap::new(),
            next_id,
            totals: EcologyReport::default(),
        }
```

</details>

<details><summary><code>recycle()</code></summary>

Partitions one world-level decomposition budget over cell detritus, returning nutrients locally and dissipating substrate energy.

[Source lines 90–95](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:90)

```rust
    pub fn recycle(
        &self,
        draft: &mut Draft<'_>,
        grid: &mut EnvironmentGrid,
        config: &EcologyConfig,
        duration: f64,
```

</details>

<details><summary><code>consumers()</code></summary>

Pays for sensing, chooses local food, attempts paid collision-checked travel, ingests accessible tissue, and schedules maintenance/growth.

[Source lines 137–142](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:137)

```rust
    pub fn consumers(
        &mut self,
        draft: &mut Draft<'_>,
        grid: &mut EnvironmentGrid,
        positions: &mut BTreeMap<OrganismId, Position>,
        window: EcologyWindow<'_>,
```

</details>

<details><summary><code>births()</code></summary>

Checks maturity, cooldown, parent investment, capacity, and local placement before committing a clone child and unused ID.

[Source lines 285–290](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:285)

```rust
    pub fn births(
        &mut self,
        draft: &mut Draft<'_>,
        grid: &EnvironmentGrid,
        positions: &mut BTreeMap<OrganismId, Position>,
        window: EcologyWindow<'_>,
```

</details>

**Structure / enum: `EcologyWindow`**

Passes one window's configuration, duration, seed, and tick together so consumer and birth logic use consistent inputs.

[Source lines 73–79](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:73)

```rust
#[derive(Clone, Copy)]
pub(crate) struct EcologyWindow<'a> {
    pub config: &'a EcologyConfig,
    pub duration: f64,
    pub seed: u64,
    pub tick: u64,
}
```

**Standalone functions**

<details><summary><code>reserve_for_birth()</code></summary>

Switches a tissue-mature organism from diagnostic growth to saving reserves when reproduction is enabled.

[Source lines 366–371](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:366)

```rust
pub(crate) fn reserve_for_birth(
    draft: &Draft<'_>,
    id: OrganismId,
    config: &EcologyConfig,
) -> Result<bool> {
    Ok(config.reproduction
```

</details>

<details><summary><code>initial_heading()</code></summary>

Derives an initial consumer heading from the versioned seed/ID mixing function without a hidden mutable RNG stream.

[Source lines 375–377](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:375)

```rust
pub(crate) fn initial_heading(seed: u64, id: OrganismId) -> f64 {
    seeded_position(seed, id.0, 1.0, 1.0).x * TAU - PI
}
```

</details>

<details><summary><code>clear_path()</code></summary>

Checks the whole proposed movement segment against nearby living points so an organism cannot tunnel through them.

[Source lines 379–384](C:/Thought/Projects/World/crates/world_core/src/ecology.rs:379)

```rust
fn clear_path(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    index: &SpatialIndex,
    id: OrganismId,
```

</details>

**Useful syntax in this file**

- `pub(crate)` shares implementation data within this crate while keeping it out of the public API.
- `#[derive(...)]` generates standard implementations such as cloning, debugging, or serialization.
- `deny_unknown_fields` rejects misspelled or unsupported JSON fields rather than ignoring them.
- `?` returns a failure immediately; the enclosing staged operation can discard its candidate state.
- `next_down()` selects the next smaller representable float to avoid spending more than an exact remaining budget.

</details>

<details><summary><code>crates/world_core/src/spatial.rs</code></summary>

**Purpose:** Indexes living points by cell and supplies exact local distances and swept-path geometry.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/spatial.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::collections::BTreeMap;
use crate::{
environment::{EnvironmentGrid, Position},
ids::OrganismId,
organism::Organism,
resources::{AccountingError, Result, finite},
};
use super::*;
use crate::rules::WorldRules;
```

- `std`: standard-library maps, paths, files, arithmetic, and process utilities; no added crate is needed for these.
- `crate`: neighboring modules of this Rust crate; existing accounting and validation are reused.

**Structure / enum: `SpatialIndex`**

Buckets organism IDs by occupied grid cell to narrow local queries; exact distance checks follow the cell lookup.

[Source lines 10–12](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:10)

```rust
pub(crate) struct SpatialIndex {
    buckets: Vec<Vec<OrganismId>>,
}
```

**Methods — open the one you are reading**

<details><summary><code>new()</code></summary>

Builds buckets from living records and rejects a missing position rather than assigning one silently.

[Source lines 15–20](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:15)

```rust
    pub fn new<'a>(
        grid: &EnvironmentGrid,
        positions: &BTreeMap<OrganismId, Position>,
        organisms: impl Iterator<Item = (&'a OrganismId, &'a Organism)>,
    ) -> Result<Self> {
        let mut result = Self {
```

</details>

<details><summary><code>insert()</code></summary>

Adds an exact organism identity to its position's grid bucket.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:32)

```rust
    pub fn insert(
        &mut self,
        grid: &EnvironmentGrid,
        id: OrganismId,
        position: Position,
    ) -> Result<()> {
```

</details>

<details><summary><code>relocate()</code></summary>

Removes old cell membership before inserting the accepted destination.

[Source lines 42–47](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:42)

```rust
    pub fn relocate(
        &mut self,
        grid: &EnvironmentGrid,
        id: OrganismId,
        from: Position,
        to: Position,
```

</details>

<details><summary><code>within()</code></summary>

Scans intersecting cells, applies the actual radius, and returns sorted IDs for deterministic decisions.

[Source lines 54–59](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:54)

```rust
    pub fn within(
        &self,
        grid: &EnvironmentGrid,
        positions: &BTreeMap<OrganismId, Position>,
        center: Position,
        radius: f64,
```

</details>

**Standalone functions**

<details><summary><code>distance()</code></summary>

Computes point separation with hypot, avoiding unnecessary squared-distance overflow.

[Source lines 89–91](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:89)

```rust
pub(crate) fn distance(a: Position, b: Position) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}
```

</details>

<details><summary><code>segment_distance()</code></summary>

Finds separation from a finite movement segment, including stationary paths and endpoint cases.

[Source lines 93–98](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:93)

```rust
pub(crate) fn segment_distance(point: Position, from: Position, to: Position) -> f64 {
    let length = distance(from, to);
    if length == 0.0 {
        return distance(point, from);
    }
    let dx = to.x - from.x;
```

</details>

<details><summary><code>swept_distance_covers_interior_endpoints_and_stationary_paths()</code></summary>

Supports swept distance covers interior endpoints and stationary paths within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 118–123](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:118)

```rust
    fn swept_distance_covers_interior_endpoints_and_stationary_paths() {
        let from = Position { x: 0.0, y: 0.0 };
        let to = Position { x: 4.0, y: 0.0 };
        assert_eq!(segment_distance(Position { x: 2.0, y: 0.0 }, from, to), 0.0);
        assert_eq!(segment_distance(Position { x: 2.0, y: 1.0 }, from, to), 1.0);
        assert_eq!(segment_distance(Position { x: 5.0, y: 0.0 }, from, to), 1.0);
```

</details>

<details><summary><code>local_queries_cross_cells_and_relocation_removes_old_membership()</code></summary>

Supports local queries cross cells and relocation removes old membership within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 128–133](C:/Thought/Projects/World/crates/world_core/src/spatial.rs:128)

```rust
    fn local_queries_cross_cells_and_relocation_removes_old_membership() {
        let mut rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        rules.world_width = 4.0;
        rules.world_height = 2.0;
        rules.grid_width = 4;
```

</details>

**Useful syntax in this file**

- `pub(crate)` shares implementation data within this crate while keeping it out of the public API.
- `?` returns a failure immediately; the enclosing staged operation can discard its candidate state.
- `#[test]` registers a native check; failed assertions fail the test run.

</details>

<details><summary><code>crates/world_core/tests/ecosystem_contract.rs</code></summary>

**Purpose:** Checks ecological continuation directly in Rust without Python or a viewer.

[Open the file](C:/Thought/Projects/World/crates/world_core/tests/ecosystem_contract.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use world_core::habitat::Habitat;
```

- `world_core`: the authoritative native habitat/resource API, shared with the Python package.

**Standalone functions**

<details><summary><code>native_checkpoint_continues_without_a_viewer_or_python_runtime()</code></summary>

Builds or checks the controlled native checkpoint continues without a viewer or python runtime scenario used by the surrounding tests.

[Source lines 7–12](C:/Thought/Projects/World/crates/world_core/tests/ecosystem_contract.rs:7)

```rust
fn native_checkpoint_continues_without_a_viewer_or_python_runtime() {
    let mut uninterrupted = Habitat::from_json(RULES, CONFIG).unwrap();
    for _ in 0..137 {
        uninterrupted.advance().unwrap();
    }
    let mut restored =
```

</details>

**Useful syntax in this file**

- `#[test]` registers a native check; failed assertions fail the test run.

</details>

<details><summary><code>tests/test_ecosystem.py</code></summary>

**Purpose:** Checks the installed native ecological cycle against hand-calculated transfers, malformed inputs, atomic failures, and exact saved continuation.

[Open the file](C:/Thought/Projects/World/tests/test_ecosystem.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest
from world import Habitat
from world._native import Habitat as NativeHabitat
```

- `json`: Python dictionary/JSON conversion; it does not implement the simulation.
- `pathlib.Path`: readable filesystem paths for configurations, artifacts, and checkpoints.
- `unittest`: standard-library test cases, assertions, and failure reporting.
- `tempfile`: isolated test-owned files that are cleaned after checks.
- `deepcopy`: independent corruption-test candidates, preserving the baseline payload.
- `NativeHabitat`: the installed Rust extension; Python wrappers delegate state changes to it.

**Class: `EcosystemTests`**

Groups the data used by the related methods and keeps its role explicit.

[Source lines 52–54](C:/Thought/Projects/World/tests/test_ecosystem.py:52)

```python
class EcosystemTests(unittest.TestCase):
    def test_birth_is_parent_funded_and_newborn_waits_until_next_window(self):
        habitat = Habitat(rules(), config())
```

**Methods — open the one you are reading**

<details><summary><code>test_birth_is_parent_funded_and_newborn_waits_until_next_window()</code></summary>

Checks birth is parent funded and newborn waits until next window.

[Source lines 53–58](C:/Thought/Projects/World/tests/test_ecosystem.py:53)

```python
    def test_birth_is_parent_funded_and_newborn_waits_until_next_window(self):
        habitat = Habitat(rules(), config())
        habitat.advance()
        organisms = [item["organism"] for item in habitat.snapshot()["organisms"]]
        parent, child = organisms
        self.assertEqual(habitat.summary()["ecology"]["births"], 1)
```

</details>

<details><summary><code>test_blocked_population_cap_spends_no_birth_investment()</code></summary>

Checks blocked population cap spends no birth investment.

[Source lines 70–75](C:/Thought/Projects/World/tests/test_ecosystem.py:70)

```python
    def test_blocked_population_cap_spends_no_birth_investment(self):
        habitat = Habitat(rules(), config(max_population=1))
        habitat.advance()
        parent = habitat.snapshot()["organisms"][0]["organism"]
        self.assertEqual(habitat.summary()["ecology"]["births"], 0)
        self.assertEqual(parent["reserves"]["nutrient"], 2.5)
```

</details>

<details><summary><code>test_failed_placement_is_unfunded_and_does_not_create_an_id()</code></summary>

Checks failed placement is unfunded and does not create an id.

[Source lines 78–83](C:/Thought/Projects/World/tests/test_ecosystem.py:78)

```python
    def test_failed_placement_is_unfunded_and_does_not_create_an_id(self):
        habitat = Habitat(rules(), config(birth_radius=100))
        habitat.advance()
        self.assertEqual(habitat.summary()["living_count"], 1)
        self.assertEqual(habitat.snapshot()["ecology"]["next_id"], 2)
        self.assertAlmostEqual(habitat.snapshot()["organisms"][0]["organism"]["reserves"]["energy"], 11.4)
```

</details>

<details><summary><code>test_cooldown_prevents_repeated_births()</code></summary>

Checks cooldown prevents repeated births.

[Source lines 85–90](C:/Thought/Projects/World/tests/test_ecosystem.py:85)

```python
    def test_cooldown_prevents_repeated_births(self):
        habitat = Habitat({**rules(), "reproduction_cooldown": 10},
                          config(population(10, 8, 40, [(1, 1)])))
        habitat.advance(2)
        self.assertEqual(habitat.summary()["ecology"]["births"], 1)
        self.assertEqual(habitat.snapshot()["organisms"][0]["organism"]["next_reproduction_age"], 11)
```

</details>

<details><summary><code>test_clone_weights_and_large_offspring_ids_survive_checkpoint()</code></summary>

Checks clone weights and large offspring ids survive checkpoint.

[Source lines 92–97](C:/Thought/Projects/World/tests/test_ecosystem.py:92)

```python
    def test_clone_weights_and_large_offspring_ids_survive_checkpoint(self):
        value = payload(Habitat(rules(), config()))
        value["ledger"]["organisms"][0]["genome"]["initial_weights"] = [0.25, -0.5]
        value["next_id"] = 2**53 + 1
        habitat = restore(value)
        habitat.advance()
```

</details>

<details><summary><code>feeding_case()</code></summary>

Builds or checks the controlled feeding case scenario used by the surrounding tests.

[Source lines 104–109](C:/Thought/Projects/World/tests/test_ecosystem.py:104)

```python
    def feeding_case(self, tissue=2, reserve_nutrient=1, reserve_energy=2, speed=0):
        cfg = config(population(tissue, reserve_nutrient, reserve_energy, [(0.99, 0.5)]),
                     population(6, 0, 10, [(1.01, 0.5)]), consumer_speed=speed,
                     decomposition=False)
        world_rules = {**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}
        return Habitat(world_rules, cfg)
```

</details>

<details><summary><code>test_cross_cell_feeding_keeps_carcass_and_waste_in_correct_cells()</code></summary>

Checks cross cell feeding keeps carcass and waste in correct cells.

[Source lines 111–116](C:/Thought/Projects/World/tests/test_ecosystem.py:111)

```python
    def test_cross_cell_feeding_keeps_carcass_and_waste_in_correct_cells(self):
        habitat = self.feeding_case()
        habitat.advance()
        state = habitat.snapshot()
        target, consumer = [item["organism"] for item in state["organisms"]]
        self.assertFalse(target["alive"])
```

</details>

<details><summary><code>test_tissue_loss_overflow_returns_minerals_to_target_cell()</code></summary>

Checks tissue loss overflow returns minerals to target cell.

[Source lines 127–132](C:/Thought/Projects/World/tests/test_ecosystem.py:127)

```python
    def test_tissue_loss_overflow_returns_minerals_to_target_cell(self):
        habitat = self.feeding_case(tissue=8, reserve_nutrient=8, reserve_energy=30)
        value = payload(habitat)
        value["rules"]["max_ingestion_nutrient_per_time"] = 1
        habitat = restore(value)
        habitat.advance()
```

</details>

<details><summary><code>test_consumers_cannot_sense_or_feed_on_remote_producers()</code></summary>

Checks consumers cannot sense or feed on remote producers.

[Source lines 139–144](C:/Thought/Projects/World/tests/test_ecosystem.py:139)

```python
    def test_consumers_cannot_sense_or_feed_on_remote_producers(self):
        cfg = config(population(2, 1, 2, [(0.2, 0.5)]), population(6, 0, 10, [(3.8, 0.5)]),
                     perception_radius=0.5, consumer_speed=0)
        habitat = Habitat({**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}, cfg)
        habitat.advance()
        self.assertEqual(habitat.summary()["ecology"]["feeding_events"], 0)
```

</details>

<details><summary><code>test_sensing_and_actual_movement_are_paid_and_work_dissipates()</code></summary>

Checks sensing and actual movement are paid and work dissipates.

[Source lines 148–153](C:/Thought/Projects/World/tests/test_ecosystem.py:148)

```python
    def test_sensing_and_actual_movement_are_paid_and_work_dissipates(self):
        cfg = config(population(2, 1, 2, [(1.5, 0.5)]), population(6, 0, 10, [(0.2, 0.5)]),
                     consumer_speed=0.1)
        habitat = Habitat({**rules(), "maturity_tissue_nutrient": 2, "minimum_reproduction_age": 10}, cfg)
        habitat.advance()
        consumer = habitat.snapshot()["organisms"][1]
```

</details>

<details><summary><code>test_decomposition_rate_is_shared_across_the_entire_grid()</code></summary>

Checks decomposition rate is shared across the entire grid.

[Source lines 160–165](C:/Thought/Projects/World/tests/test_ecosystem.py:160)

```python
    def test_decomposition_rate_is_shared_across_the_entire_grid(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5), (2.5, 0.5)]),
                     decomposition_fraction_per_time=10, reproduction=False)
        habitat = Habitat({**rules(), "max_decomposition_nutrient_per_time": 1}, cfg)
        habitat.advance(2)
        row = habitat.summary()
```

</details>

<details><summary><code>test_decomposition_also_respects_the_world_energy_cap()</code></summary>

Checks decomposition also respects the world energy cap.

[Source lines 173–178](C:/Thought/Projects/World/tests/test_ecosystem.py:173)

```python
    def test_decomposition_also_respects_the_world_energy_cap(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5), (2.5, 0.5)]),
                     decomposition_fraction_per_time=10, reproduction=False)
        habitat = Habitat({**rules(), "max_decomposition_energy_per_time": 1}, cfg)
        habitat.advance(2)
        self.assertAlmostEqual(habitat.summary()["ecology"]["decomposed_energy"], 1)
```

</details>

<details><summary><code>test_depleted_digestion_cannot_consume_food_for_free()</code></summary>

Checks depleted digestion cannot consume food for free.

[Source lines 181–186](C:/Thought/Projects/World/tests/test_ecosystem.py:181)

```python
    def test_depleted_digestion_cannot_consume_food_for_free(self):
        cfg = config(population(2, 1, 2, [(0.99, 0.5)]), population(6, 0, 0.02, [(1.01, 0.5)]),
                     reproduction=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        self.assertEqual(habitat.summary()["ecology"]["feeding_events"], 0)
```

</details>

<details><summary><code>test_fast_movement_cannot_tunnel_through_another_consumer()</code></summary>

Checks fast movement cannot tunnel through another consumer.

[Source lines 189–194](C:/Thought/Projects/World/tests/test_ecosystem.py:189)

```python
    def test_fast_movement_cannot_tunnel_through_another_consumer(self):
        cfg = config(population(6, 0, 20, [(3.5, 0.5)]),
                     population(6, 0, 20, [(0.5, 0.5), (1.5, 0.5)]),
                     reproduction=False, consumer_speed=10, perception_radius=4)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
```

</details>

<details><summary><code>test_nearly_equal_clearance_and_feeding_radii_still_allow_contact()</code></summary>

Checks nearly equal clearance and feeding radii still allow contact.

[Source lines 200–205](C:/Thought/Projects/World/tests/test_ecosystem.py:200)

```python
    def test_nearly_equal_clearance_and_feeding_radii_still_allow_contact(self):
        cfg = config(population(6, 0, 20, [(2.5, 0.5)]),
                     population(6, 0, 20, [(0.5, 0.5)]),
                     reproduction=False, consumer_speed=10, perception_radius=4,
                     feeding_radius=0.11)
        habitat = Habitat(rules(), cfg)
```

</details>

<details><summary><code>test_dark_extinction_is_not_reseeded_and_recycling_can_be_disabled()</code></summary>

Checks dark extinction is not reseeded and recycling can be disabled.

[Source lines 211–216](C:/Thought/Projects/World/tests/test_ecosystem.py:211)

```python
    def test_dark_extinction_is_not_reseeded_and_recycling_can_be_disabled(self):
        cfg = config(population(10, 0, 0, [(0.5, 0.5)]), reproduction=False,
                     decomposition=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance(50)
        row = habitat.summary()
```

</details>

<details><summary><code>test_growth_metrics_include_producers_and_preserve_resources()</code></summary>

Checks growth metrics include producers and preserve resources.

[Source lines 225–230](C:/Thought/Projects/World/tests/test_ecosystem.py:225)

```python
    def test_growth_metrics_include_producers_and_preserve_resources(self):
        cfg = config(population(2, 0.5, 5, [(0.5, 0.5)]), reproduction=False,
                     decomposition=False)
        habitat = Habitat(rules(), cfg)
        habitat.advance()
        row = habitat.summary()
```

</details>

<details><summary><code>test_checkpoint_preserves_partial_windows_and_future_trajectories()</code></summary>

Checks checkpoint preserves partial windows and future trajectories.

[Source lines 237–242](C:/Thought/Projects/World/tests/test_ecosystem.py:237)

```python
    def test_checkpoint_preserves_partial_windows_and_future_trajectories(self):
        world_rules = json.loads((ROOT / "configs/ecosystem_rules.json").read_text())
        cfg = json.loads((ROOT / "configs/ecosystem_habitat.json").read_text())
        uninterrupted = Habitat(world_rules, cfg)
        uninterrupted.advance(137)
        resumed = restore(payload(uninterrupted))
```

</details>

<details><summary><code>test_atomic_save_overwrites_and_failed_destinations_preserve_previous_file()</code></summary>

Checks atomic save overwrites and failed destinations preserve previous file.

[Source lines 253–258](C:/Thought/Projects/World/tests/test_ecosystem.py:253)

```python
    def test_atomic_save_overwrites_and_failed_destinations_preserve_previous_file(self):
        habitat = Habitat(rules(), config())
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory) / "saved"
            path = folder / "world.json"
            habitat.save(path)
```

</details>

<details><summary><code>test_corrupted_checkpoints_are_rejected()</code></summary>

Checks corrupted checkpoints are rejected.

[Source lines 268–273](C:/Thought/Projects/World/tests/test_ecosystem.py:268)

```python
    def test_corrupted_checkpoints_are_rejected(self):
        habitat = self.feeding_case()
        habitat.advance()
        baseline = payload(habitat)
        changes = [
            lambda p: p.update(schema_version=2), lambda p: p.update(kind="inspection"),
```

</details>

<details><summary><code>test_offspring_id_overflow_rolls_back_everything()</code></summary>

Checks offspring id overflow rolls back everything.

[Source lines 299–304](C:/Thought/Projects/World/tests/test_ecosystem.py:299)

```python
    def test_offspring_id_overflow_rolls_back_everything(self):
        value = payload(Habitat(rules(), config()))
        value["next_id"] = 2**64 - 1
        habitat = restore(value)
        before = habitat.snapshot()
        with self.assertRaises(ValueError):
```

</details>

<details><summary><code>test_inspection_snapshot_is_not_accepted_as_a_checkpoint()</code></summary>

Checks inspection snapshot is not accepted as a checkpoint.

[Source lines 308–311](C:/Thought/Projects/World/tests/test_ecosystem.py:308)

```python
    def test_inspection_snapshot_is_not_accepted_as_a_checkpoint(self):
        habitat = Habitat(rules(), config())
        with self.assertRaises(ValueError):
            NativeHabitat.from_checkpoint_json(json.dumps(habitat.snapshot()))
```

</details>

<details><summary><code>test_ecology_config_rejects_invalid_costs_radii_and_limits()</code></summary>

Checks ecology config rejects invalid costs radii and limits.

[Source lines 313–318](C:/Thought/Projects/World/tests/test_ecosystem.py:313)

```python
    def test_ecology_config_rejects_invalid_costs_radii_and_limits(self):
        for bad in [{"consumer_speed": -1}, {"minimum_spacing": 1}, {"max_population": 0},
                    {"movement_energy_per_distance": 0}, {"perception_radius": 0.1},
                    {"decomposition_fraction_per_time": -1}, {"consumer_speed": True},
                    {"reproduction": 1}, {"unknown": 1}]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
```

</details>

**Standalone functions**

<details><summary><code>rules()</code></summary>

Builds or checks the controlled rules scenario used by the surrounding tests.

[Source lines 15–17](C:/Thought/Projects/World/tests/test_ecosystem.py:15)

```python
def rules():
    return {**RULES, "world_width": 4.0, "world_height": 2.0, "grid_width": 4, "grid_height": 2,
            "physics_dt": 1.0, "ecology_interval_ticks": 1, "minimum_reproduction_age": 1.0}
```

</details>

<details><summary><code>population()</code></summary>

Builds or checks the controlled population scenario used by the surrounding tests.

[Source lines 20–22](C:/Thought/Projects/World/tests/test_ecosystem.py:20)

```python
def population(tissue, nutrient, energy, positions):
    return {"count": len(positions), "tissue_nutrient": tissue, "reserve_nutrient": nutrient,
            "reserve_energy": energy, "positions": [{"x": x, "y": y} for x, y in positions]}
```

</details>

<details><summary><code>config()</code></summary>

Builds or checks the controlled config scenario used by the surrounding tests.

[Source lines 25–30](C:/Thought/Projects/World/tests/test_ecosystem.py:25)

```python
def config(producers=None, consumers=None, minerals=0.0, light=0.0, **ecology):
    producers = producers or population(6, 2.5, 12, [(1, 1)])
    groups = [producers] + ([consumers] if consumers else [])
    return {"schema_version": 1, "seed": 73,
            "initial_nutrient": minerals * 8 + sum(p["count"]*(p["tissue_nutrient"]+p["reserve_nutrient"]) for p in groups),
            "initial_energy": sum(p["count"]*(2*p["tissue_nutrient"]+p["reserve_energy"]) for p in groups),
```

</details>

<details><summary><code>payload()</code></summary>

Builds or checks the controlled payload scenario used by the surrounding tests.

[Source lines 38–42](C:/Thought/Projects/World/tests/test_ecosystem.py:38)

```python
def payload(habitat):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "checkpoint.json"
        habitat.save(path)
        return json.loads(path.read_text(encoding="utf-8"))
```

</details>

<details><summary><code>restore()</code></summary>

Builds or checks the controlled restore scenario used by the surrounding tests.

[Source lines 45–49](C:/Thought/Projects/World/tests/test_ecosystem.py:45)

```python
def restore(value):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "checkpoint.json"
        path.write_text(json.dumps(value, allow_nan=False), encoding="utf-8")
        return Habitat.load(path)
```

</details>

</details>

## Step 7: Observer and complete habitat continuation

The Bevy application observes the same headless core. Pause, single ticks, speed, field display, selection, and save/load are observer controls. The complete habitat checkpoint includes every state component used by this diagnostic ecosystem; the separate PyTorch fixture is not yet connected.

<details><summary><code>crates/world_core/src/checkpoint.rs</code></summary>

**Purpose:** Saves and restores the complete Rust-owned habitat, validating a candidate before acceptance and replacing files after successful writes.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use crate::{
    VERSION,
    ecology::{EcologyReport, EcologyState},
    environment::{EnvironmentGrid, Position},
    habitat::{Habitat, WindowReport},
    habitat_config::HabitatConfig,
    ids::OrganismId,
    resources::{AccountingError, Result},
    rules::WorldRules,
    scheduler::Scheduler,
    transactions::{PopulationState, ResourceLedger},
};
```

- `std` supplies maps/sets, paths/files, timing, and checked native utilities without extra crates.
- `serde` defines typed data serialization; strict records reject unsupported fields.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.

**Structure / enum: `Placement`**

Associates an exact integer organism ID with its saved position. A list permits explicit duplicate detection on load.

[Source lines 23–28](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:23)

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    id: OrganismId,
    position: Position,
}
```

**Structure / enum: `Motion`**

Associates an exact integer consumer ID with its saved heading; duplicate entries are rejected.

[Source lines 30–35](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:30)

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Motion {
    id: OrganismId,
    heading: f64,
}
```

**Structure / enum: `Checkpoint`**

Defines the strict versioned data-only habitat format, including pools, genomes, geometry, clocks, headings, counters, and latest receipt.

[Source lines 37–44](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:37)

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema_version: u32,
    kind: String,
    engine_version: String,
    rules: WorldRules,
    config: HabitatConfig,
```

**Structure / enum: `Temporary`**

Owns a temporary checkpoint path so cleanup runs automatically when its scope ends, including failed writes.

[Source lines 283–283](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:283)

```rust
struct Temporary(PathBuf);
```

**Methods — open the one you are reading**

<details><summary><code>drop()</code></summary>

Removes the temporary path if still present; after a successful rename it is already gone.

[Source lines 286–288](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:286)

```rust
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
```

</details>

**Methods on `Habitat` — implemented here**

<details><summary><code>checkpoint_json()</code></summary>

Audits the current habitat and serializes every continuation-relevant component into the strict checkpoint record.

[Source lines 60–66](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:60)

```rust
    pub fn checkpoint_json(&self) -> Result<String> {
        self.audit()?;
        let checkpoint = Checkpoint {
            schema_version: 1,
            kind: "world.habitat".into(),
            engine_version: VERSION.into(),
            rules: self.rules.clone(),
```

</details>

<details><summary><code>from_checkpoint_json()</code></summary>

Validates format, identities, geometry, budgets, clocks, counters, and resource conservation before returning a reconstructed habitat.

[Source lines 99–105](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:99)

```rust
    pub fn from_checkpoint_json(text: &str) -> Result<Self> {
        let checkpoint: Checkpoint = serde_json::from_str(text)
            .map_err(|e| AccountingError::Invalid(format!("invalid habitat checkpoint: {e}")))?;
        if checkpoint.schema_version != 1
            || checkpoint.kind != "world.habitat"
            || checkpoint.engine_version != VERSION
        {
```

</details>

<details><summary><code>save()</code></summary>

Serializes and audits first, then delegates safe destination replacement to write_atomic.

[Source lines 220–222](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:220)

```rust
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        write_atomic(path.as_ref(), self.checkpoint_json()?.as_bytes())
    }
```

</details>

<details><summary><code>load()</code></summary>

Reads UTF-8 JSON from the supplied path and reconstructs a validated candidate habitat.

[Source lines 224–227](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:224)

```rust
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|e| AccountingError::Invalid(e.to_string()))?;
        Self::from_checkpoint_json(&text)
    }
```

</details>

**Standalone functions**

<details><summary><code>write_atomic()</code></summary>

Creates an exclusive sibling temporary file, writes and flushes it, then replaces the destination after success.

[Source lines 230–236](C:/Thought/Projects/World/crates/world_core/src/checkpoint.rs:230)

```rust
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| AccountingError::Invalid(e.to_string()))?;
    let filename = path
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `#[derive]` generates ordinary cloning/debug/serialization implementations.
- `skip_serializing_if` keeps absent extension state out of legacy JSON formats.
- `Option<T>` represents an absent/present value; `Result<T>` carries success/error.

</details>

<details><summary><code>crates/world_viewer/Cargo.toml</code></summary>

**Purpose:** Defines the native observer executable and its Bevy/core/JSON dependencies; the core does not depend on this crate.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/Cargo.toml)

[Source lines 1–8](C:/Thought/Projects/World/crates/world_viewer/Cargo.toml:1)

```toml
[package]
name = "world_viewer"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
description = "Observer and controls for the headless World habitat"

```

This file is declarative data rather than runtime classes/functions. Workspace declarations share exact reviewed package/dependency settings; native genetics/core remain independent of the viewer.

</details>

<details><summary><code>crates/world_viewer/src/main.rs</code></summary>

**Purpose:** Starts the native Bevy application, connects controls to the authoritative world, and schedules fixed simulation ticks independently of rendering.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/main.rs)

**Imports / uses and why**

```rust
use std::{num::NonZeroU8, path::PathBuf};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    winit::WinitSettings,
};
use world_core::habitat::Habitat;
use model::{Action, ViewerModel};
use options::Options;
```

- `bevy`: Bevy supplies the application loop, ECS resources, input, camera, meshes, and native rendering.
- `model`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `options`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `AppModel`**

Wraps the viewer model as a Bevy resource so systems access the same observer and camera state.

[Source lines 21–26](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:21)

```rust
pub struct AppModel(pub ViewerModel);

#[derive(Resource)]
struct Runtime {
    stop_at: Option<u64>,
    capture: Option<PathBuf>,
```

**Struct: `Runtime`**

Tracks bounded viewer capture/shutdown requests and owner-save acknowledgement, separately from the biological world.

[Source lines 24–29](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:24)

```rust
struct Runtime {
    stop_at: Option<u64>,
    capture: Option<PathBuf>,
    frames: u64,
    requested: bool,
    failed: bool,
```

**Functions outside impl blocks**

<details><summary><code>main()</code></summary>

Prints help or starts the observer, returning a failing process exit status when startup/runtime fails.

[Source lines 33–38](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:33)

```rust
fn main() {
    if std::env::args().any(|arg| arg == "--help") {
        println!(
            "World viewer\n--rules PATH --config PATH --load PATH --checkpoint PATH --paused\n--run-for-ticks N --capture PATH.bmp --observe PATH --commands DIR\nWASD/right-drag: pan | wheel: zoom | Home: fit | F: follow | G: fields | B: brain\nSpace: pause | N: one tick | +/-: speed | F5: save | F9: load | L: field | Click: inspect"
        );
        return;
```

</details>

<details><summary><code>start()</code></summary>

Captures the actual rules, configuration and source fingerprints, and writes an initial complete autosave before further biological decisions.

[Source lines 47–52](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:47)

```rust
fn start() -> Result<(), String> {
    let options = Options::parse(std::env::args().skip(1))?;
    let mut remote = None;
    let habitat = if let Some(path) = &options.observe {
        let commands = options
            .commands
```

</details>

<details><summary><code>keyboard_actions()</code></summary>

Translates newly pressed control keys into testable observer commands.

[Source lines 179–184](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:179)

```rust
fn keyboard_actions(keys: &ButtonInput<KeyCode>) -> Vec<Action> {
    let mut actions = Vec::new();
    for (key, action) in [
        (KeyCode::Space, Action::TogglePause),
        (KeyCode::KeyN, Action::Advance),
        (KeyCode::Equal, Action::Faster),
```

</details>

<details><summary><code>input()</code></summary>

Applies keyboard commands, adjusts the fixed timestep after load, and maps a click to the nearest selectable living identity.

[Source lines 199–204](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:199)

```rust
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<scene::WorldCamera>>,
    mut window: Single<&mut Window>,
    mut app: ResMut<AppModel>,
```

</details>

<details><summary><code>refresh()</code></summary>

Accepts a new validated owner publication for display while preserving observer camera/selection preferences.

[Source lines 281–286](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:281)

```rust
fn refresh(mut app: ResMut<AppModel>) {
    if let Err(error) = app.0.refresh() {
        app.0.status = format!("Observer update rejected: {error}");
    }
}

```

</details>

<details><summary><code>navigation()</code></summary>

Applies pan, zoom, and drag to observer camera preferences, independently of simulation updates.

[Source lines 287–292](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:287)

```rust
fn navigation(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    area: Res<scene::ViewArea>,
```

</details>

<details><summary><code>advance()</code></summary>

Advances the shared native habitat; a rejected update pauses it and reports the error.

[Source lines 352–357](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:352)

```rust
fn advance(mut app: ResMut<AppModel>, mut runtime: ResMut<Runtime>) {
    if let Err(error) = app.0.advance_frame(runtime.stop_at) {
        app.0.status = error.to_string();
        app.0.paused = true;
        error!("Habitat update rejected: {error}");
        if runtime.stop_at.is_some() {
```

</details>

<details><summary><code>finish()</code></summary>

Completes an optional bounded smoke run by saving its state, optionally capturing a rendered BMP, and exiting with the correct status.

[Source lines 363–368](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:363)

```rust
fn finish(
    mut commands: Commands,
    mut runtime: ResMut<Runtime>,
    mut app: ResMut<AppModel>,
    mut exit: MessageWriter<AppExit>,
) {
```

</details>

<details><summary><code>keyboard_routes_pause_tick_save_and_load()</code></summary>

Supports keyboard routes pause tick save and load within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 444–449](C:/Thought/Projects/World/crates/world_viewer/src/main.rs:444)

```rust
    fn keyboard_routes_pause_tick_save_and_load() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        keys.press(KeyCode::KeyN);
        keys.press(KeyCode::F5);
        keys.press(KeyCode::F9);
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_viewer/src/model.rs</code></summary>

**Purpose:** Keeps viewer controls separate from rendering so pause, speed, save/load, and selection can be tested without a window.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/model.rs)

**Imports / uses and why**

```rust
use std::path::PathBuf;
use world_core::{habitat::Habitat, ids::OrganismId, resources::Result};
```

- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Enum: `Action`**

Lists viewer commands explicitly, allowing keyboard input to use the same local or remote command handling.

[Source lines 5–10](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:5)

```rust
pub enum Action {
    TogglePause,
    Advance,
    Faster,
    Slower,
    Save,
```

**Struct: `ViewerModel`**

Owns observer state, selection and camera preferences. Biological decisions remain with the runtime owner when observing a neural run.

[Source lines 16–21](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:16)

```rust
pub struct ViewerModel {
    pub habitat: Habitat,
    pub display: Option<world_core::display::DisplayFrame>,
    pub revision: u64,
    pub paused: bool,
    pub speed: u32,
```

**Methods in `impl ViewerModel`**

<details><summary><code>new()</code></summary>

Initializes observer controls around an already constructed habitat.

[Source lines 40–45](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:40)

```rust
    pub fn new(habitat: Habitat, checkpoint: PathBuf, paused: bool) -> Self {
        let (width, height, _, _) = habitat.grid().dimensions();
        let summary =
            serde_json::from_str(&habitat.summary_json().unwrap_or_default()).unwrap_or_default();
        Self {
            habitat,
```

</details>

<details><summary><code>apply()</code></summary>

Applies pause/tick/speed/selection/field/save/load commands; a failed load cannot replace the current world.

[Source lines 71–76](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:71)

```rust
    pub fn apply(&mut self, action: Action) -> Result<()> {
        if let Some(observer) = &mut self.observer
            && !matches!(action, Action::ToggleLayer | Action::Select(_))
        {
            observer.send(&action, &self.checkpoint)?;
            self.status = "Command sent to biological runtime".into();
```

</details>

<details><summary><code>advance_frame()</code></summary>

Runs the requested native tick count only while unpaused, respecting an optional bounded-run endpoint.

[Source lines 120–125](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:120)

```rust
    pub fn advance_frame(&mut self, stop_at: Option<u64>) -> Result<()> {
        if self.observer.is_some() {
            return Ok(());
        }
        if self.paused {
            return Ok(());
```

</details>

<details><summary><code>refresh()</code></summary>

Accepts a new validated owner publication for display while preserving observer camera/selection preferences.

[Source lines 137–142](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:137)

```rust
    pub fn refresh(&mut self) -> Result<()> {
        if let Some(observer) = &mut self.observer
            && let Some(update) = observer.poll()?
        {
            let old_dimensions = self.dimensions();
            if let Some(habitat) = update.habitat {
```

</details>

<details><summary><code>fit()</code></summary>

Resets only camera position/zoom/following to fit the habitat; biological state is unchanged.

[Source lines 160–165](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:160)

```rust
    pub fn fit(&mut self) {
        let (width, height, _, _) = self.dimensions();
        self.center = world_core::environment::Position {
            x: width * 0.5,
            y: height * 0.5,
        };
```

</details>

<details><summary><code>update_summary()</code></summary>

Supports update summary within this component's responsibility; the source link opens the complete implementation.

[Source lines 169–174](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:169)

```rust
    fn update_summary(&mut self) -> Result<()> {
        if let Some(display) = &self.display {
            self.summary = display.summary.clone();
            return Ok(());
        }
        self.summary = serde_json::from_str(&self.habitat.summary_json()?)
```

</details>

<details><summary><code>dimensions()</code></summary>

Handles dimensions for `ViewerModel`.

[Source lines 179–184](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:179)

```rust
    pub fn dimensions(&self) -> (f64, f64, usize, usize) {
        self.display.as_ref().map_or_else(
            || self.habitat.grid().dimensions(),
            |d| (d.width, d.height, d.columns, d.rows),
        )
    }
```

</details>

<details><summary><code>tick()</code></summary>

Handles tick for `ViewerModel`.

[Source lines 186–191](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:186)

```rust
    pub fn tick(&self) -> u64 {
        self.display
            .as_ref()
            .map_or_else(|| self.habitat.tick(), |d| d.tick)
    }

```

</details>

<details><summary><code>position()</code></summary>

Handles position for `ViewerModel`.

[Source lines 192–197](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:192)

```rust
    pub fn position(&self, id: OrganismId) -> Option<world_core::environment::Position> {
        if let Some(display) = &self.display {
            display
                .organisms
                .iter()
                .find(|o| o.id == id.0)
```

</details>

**Functions outside impl blocks**

<details><summary><code>model()</code></summary>

Builds or checks the controlled model scenario used by the surrounding tests.

[Source lines 209–214](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:209)

```rust
    fn model() -> ViewerModel {
        let habitat = Habitat::from_json(
            include_str!("../../../configs/ecosystem_rules.json"),
            include_str!("../../../configs/ecosystem_habitat.json"),
        )
        .unwrap();
```

</details>

<details><summary><code>pause_tick_speed_and_layer_controls_preserve_native_time()</code></summary>

Supports pause tick speed and layer controls preserve native time within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 223–228](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:223)

```rust
    fn pause_tick_speed_and_layer_controls_preserve_native_time() {
        let mut model = model();
        model.apply(Action::TogglePause).unwrap();
        model.advance_frame(None).unwrap();
        assert_eq!(model.habitat.tick(), 0);
        model.apply(Action::Advance).unwrap();
```

</details>

<details><summary><code>save_load_restores_the_world_and_failed_load_preserves_it()</code></summary>

Supports save load restores the world and failed load preserves it within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 244–249](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:244)

```rust
    fn save_load_restores_the_world_and_failed_load_preserves_it() {
        let mut model = model();
        for _ in 0..12 {
            model.advance_frame(None).unwrap();
        }
        model.apply(Action::Save).unwrap();
```

</details>

<details><summary><code>camera_preferences_and_fit_do_not_change_biological_trajectories()</code></summary>

Supports camera preferences and fit do not change biological trajectories within this component's responsibility; the source link opens the complete implementation.

[Source lines 262–267](C:/Thought/Projects/World/crates/world_viewer/src/model.rs:262)

```rust
    fn camera_preferences_and_fit_do_not_change_biological_trajectories() {
        let mut first = model();
        let mut second = model();
        first.zoom = 32.0;
        first.center = world_core::environment::Position { x: -10.0, y: 50.0 };
        first.show_neural = true;
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_viewer/src/options.rs</code></summary>

**Purpose:** Validates command-line paths, starting state, and optional bounded smoke-run/capture settings.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/options.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::{collections::VecDeque, path::PathBuf};
use super::*;
```

- `std` supplies maps/sets, paths/files, timing, and checked native utilities without extra crates.
- `Path` makes configuration/checkpoint/artifact paths explicit.

**Structure / enum: `Options`**

Stores validated command-line choices before the application creates a world or window.

[Source lines 3–10](C:/Thought/Projects/World/crates/world_viewer/src/options.rs:3)

```rust
pub struct Options {
    pub rules: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub load: Option<PathBuf>,
    pub checkpoint: PathBuf,
    pub paused: bool,
    pub run_for_ticks: Option<u64>,
    pub capture: Option<PathBuf>,
```

**Methods — open the one you are reading**

<details><summary><code>parse()</code></summary>

Accepts known options, validates unsigned tick lengths and BMP output, and rejects incomplete or unsupported inputs.

[Source lines 20–26](C:/Thought/Projects/World/crates/world_viewer/src/options.rs:20)

```rust
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut args: VecDeque<_> = args.collect();
        let mut options = Self {
            rules: None,
            config: None,
            load: None,
            checkpoint: "artifacts/world-checkpoint.json".into(),
```

</details>

**Standalone functions**

<details><summary><code>options_reject_missing_unknown_and_invalid_values()</code></summary>

Supports options reject missing unknown and invalid values within this file's responsibility; the source excerpt shows its input and initial work.

[Source lines 105–111](C:/Thought/Projects/World/crates/world_viewer/src/options.rs:105)

```rust
    fn options_reject_missing_unknown_and_invalid_values() {
        for args in [
            vec!["--rules"],
            vec!["--unknown", "x"],
            vec!["--run-for-ticks", "-1"],
            vec!["--capture", "x.png"],
        ] {
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `Option<T>` represents an absent/present value; `Result<T>` carries success/error.
- `#[test]` registers native contract checks; failed assertions fail validation.

</details>

<details><summary><code>crates/world_viewer/src/scene.rs</code></summary>

**Purpose:** Renders the independent world camera, real module anatomy/injury, selected neural values, resource layers, sidebar, and minimap.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs)

**Imports / uses and why**

```rust
use crate::AppModel;
use bevy::{
    camera::{ScalingMode, Viewport, visibility::RenderLayers},
    prelude::*,
    sprite::Anchor,
    text::{FontSize, TextBounds},
};
use std::collections::{BTreeMap, BTreeSet};
use world_core::{agents, ids::OrganismId};
use world_genetics::{body::ModuleKind, neural::NodeKind};
```

- `bevy`: Bevy supplies the application loop, ECS resources, input, camera, meshes, and native rendering.
- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `WorldCamera`**

Handles WorldCamera at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 13–18](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:13)

```rust
pub struct WorldCamera;
#[derive(Component)]
pub struct UiElement;
#[derive(Component)]
pub struct CellGlyph;
#[derive(Component)]
```

**Struct: `UiElement`**

Handles UiElement at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 15–20](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:15)

```rust
pub struct UiElement;
#[derive(Component)]
pub struct CellGlyph;
#[derive(Component)]
pub struct BodyGlyph;
#[derive(Component)]
```

**Struct: `CellGlyph`**

Handles CellGlyph at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 17–22](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:17)

```rust
pub struct CellGlyph;
#[derive(Component)]
pub struct BodyGlyph;
#[derive(Component)]
pub struct Panel;
#[derive(Component)]
```

**Struct: `BodyGlyph`**

Handles BodyGlyph at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 19–24](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:19)

```rust
pub struct BodyGlyph;
#[derive(Component)]
pub struct Panel;
#[derive(Component)]
pub struct Heading;
#[derive(Component)]
```

**Struct: `Panel`**

Handles Panel at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 21–26](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:21)

```rust
pub struct Panel;
#[derive(Component)]
pub struct Heading;
#[derive(Component)]
pub struct Subtitle;
#[derive(Component)]
```

**Struct: `Heading`**

Handles Heading at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 23–28](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:23)

```rust
pub struct Heading;
#[derive(Component)]
pub struct Subtitle;
#[derive(Component)]
pub struct Legend;
#[derive(Component)]
```

**Struct: `Subtitle`**

Handles Subtitle at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 25–30](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:25)

```rust
pub struct Subtitle;
#[derive(Component)]
pub struct Legend;
#[derive(Component)]
pub struct Background;

```

**Struct: `Legend`**

Handles Legend at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 27–32](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:27)

```rust
pub struct Legend;
#[derive(Component)]
pub struct Background;

pub(crate) struct Glyph {
    pub(crate) entity: Entity,
```

**Struct: `Background`**

Handles Background at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 29–34](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:29)

```rust
pub struct Background;

pub(crate) struct Glyph {
    pub(crate) entity: Entity,
    pub(crate) material: Handle<ColorMaterial>,
}
```

**Struct: `Glyph`**

Handles Glyph at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 31–36](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:31)

```rust
pub(crate) struct Glyph {
    pub(crate) entity: Entity,
    pub(crate) material: Handle<ColorMaterial>,
}

#[derive(Resource)]
```

**Struct: `SceneCache`**

Keeps mesh/material handles and existing organism entities so a new publication updates reusable render objects instead of rebuilding the scene every frame.

[Source lines 37–42](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:37)

```rust
pub struct SceneCache {
    pub(crate) revision: Option<(u64, bool, bool)>,
    pub(crate) cells: Vec<Entity>,
    pub(crate) bodies: BTreeMap<(OrganismId, u64), Glyph>,
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) injury_mesh: Handle<Mesh>,
```

**Struct: `ViewArea`**

Shares the camera scale and UI width with navigation and drawing; resizing these view values does not resize the biological world.

[Source lines 48–53](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:48)

```rust
pub struct ViewArea {
    pub ui_width: f32,
    pub world_height: f32,
    pub aspect: f32,
}

```

**Type alias: `UiQuery`**

A reusable name for the following type expression. In viewer systems, aliases keep component queries readable; lifetime parameters describe the borrowed world/system data.

[Source lines 167–172](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:167)

```rust
type UiQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
```

**Type alias: `CellSprites`**

A reusable name for the following type expression. In viewer systems, aliases keep component queries readable; lifetime parameters describe the borrowed world/system data.

[Source lines 181–186](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:181)

```rust
pub(crate) type CellSprites<'w, 's> = Query<
    'w,
    's,
    (&'static mut Sprite, &'static mut Visibility),
    (With<CellGlyph>, Without<BodyGlyph>),
>;
```

**Type alias: `BodyTransforms`**

A reusable name for the following type expression. In viewer systems, aliases keep component queries readable; lifetime parameters describe the borrowed world/system data.

[Source lines 187–192](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:187)

```rust
pub(crate) type BodyTransforms<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<BodyGlyph>, Without<CellGlyph>),
>;
```

**Functions outside impl blocks**

<details><summary><code>setup()</code></summary>

Creates the camera, frame, labels, sidebar, and shared draw assets.

[Source lines 54–59](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:54)

```rust
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
```

</details>

<details><summary><code>caption()</code></summary>

Handles caption at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 154–159](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:154)

```rust
pub fn caption(app: Res<AppModel>, mut texts: Query<&mut Text2d, With<Subtitle>>) {
    let value = if app.0.display.is_some() {
        "Green producers | blue consumers | white dot facing | red injury | funded body size"
    } else {
        "Green capture | gold intake | cyan sensing | blue propulsion | red injury"
    };
```

</details>

<details><summary><code>layout()</code></summary>

Decodes the connected body/camera layout; physical body decoding preserves attachments, while viewer layout only changes observation geometry.

[Source lines 194–199](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:194)

```rust
pub fn layout(
    window: Single<&Window>,
    mut app: ResMut<AppModel>,
    mut area: ResMut<ViewArea>,
    mut camera: Single<(&mut Camera, &mut Projection, &mut Transform), With<WorldCamera>>,
    mut ui: UiQuery<'_, '_>,
```

</details>

<details><summary><code>colour()</code></summary>

Supports colour within this component's responsibility; the source link opens the complete implementation.

[Source lines 267–272](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:267)

```rust
fn colour(kind: ModuleKind, producer: bool, predator: bool, integrity: f64) -> Color {
    let (r, g, b) = match kind {
        ModuleKind::Core => {
            if producer {
                (0.25, 0.72, 0.43)
            } else if predator {
```

</details>

<details><summary><code>sync()</code></summary>

Refreshes grid field colors and living glyphs from native state, removing dead draw entities and rebuilding after load.

[Source lines 289–294](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:289)

```rust
pub fn sync(
    mut commands: Commands,
    mut app: ResMut<AppModel>,
    mut cache: ResMut<SceneCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cells: CellSprites<'_, '_>,
```

</details>

<details><summary><code>rectangle()</code></summary>

Supports rectangle within this component's responsibility; the source link opens the complete implementation.

[Source lines 485–490](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:485)

```rust
fn rectangle(gizmos: &mut Gizmos, low: Vec2, high: Vec2, color: Color) {
    for (a, b) in [
        (low, Vec2::new(high.x, low.y)),
        (Vec2::new(high.x, low.y), high),
        (high, Vec2::new(low.x, high.y)),
        (Vec2::new(low.x, high.y), low),
```

</details>

<details><summary><code>panel()</code></summary>

Displays audited totals and selected-organism details and draws the selection ring.

[Source lines 496–501](C:/Thought/Projects/World/crates/world_viewer/src/scene.rs:496)

```rust
pub fn panel(
    app: Res<AppModel>,
    area: Res<ViewArea>,
    mut panels: Query<(&mut Text2d, &mut TextBounds), With<Panel>>,
    mut gizmos: Gizmos,
) {
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>scripts/run_viewer.ps1</code></summary>

**Purpose:** Prepares local compiler paths, audits the lock, builds from cached sources, launches the observer, and restores temporary environment settings.

[Open the file](C:/Thought/Projects/World/scripts/run_viewer.ps1)

[Source lines 1–6](C:/Thought/Projects/World/scripts/run_viewer.ps1:1)

```powershell
param(
    [string]$Rules,
    [string]$Config,
    [string]$Load,
    [string]$Checkpoint = 'artifacts/world-checkpoint.json',
    [switch]$Paused
```

**Execution and syntax:** `param` declares arguments; `$env:` accesses process environment; `&` invokes a command; `try/finally` restores the caller's environment. Native exit codes are checked, and `plain_command.py` captures UTF-8 output before PowerShell decoding.

</details>

## Step 8: Inherited controllers and inspectable foundations

Rust supplies permitted local inputs and complete action validation; PyTorch proposes sparse recurrent values. Newborns inherit encoded parameters and reset memory. Diagnostics, functional presets, fixed liquid candidates, coordinated continuation, and the Python-owned observer are included.

<details><summary><code>configs/evolution_habitat.json</code></summary>

**Purpose:** Declares fully funded differentiated founder genotypes and explicit evolving-mode options; starting brain parameters are encoded here.

[Open the file](C:/Thought/Projects/World/configs/evolution_habitat.json)

[Source lines 1–8](C:/Thought/Projects/World/configs/evolution_habitat.json:1)

```json
{
  "schema_version": 1,
  "seed": 73,
  "initial_nutrient": 3821.0,
  "initial_energy": 2844.0,
  "mineral_nutrient_per_cell": 3.0,
  "light_energy_per_cell_per_time": 8.0,
  "light_gradient": 0.5,
```

This file is declarative data rather than runtime classes/functions. Named fields configure validated rules, funded founder bodies/brains, and explicit costs/limits. Saved configuration includes resolved defaults.

</details>

<details><summary><code>configs/evolution_rules.json</code></summary>

**Purpose:** Retains biological base coefficients with the evolving preset's 32 x 32 habitat and grid.

[Open the file](C:/Thought/Projects/World/configs/evolution_rules.json)

[Source lines 1–8](C:/Thought/Projects/World/configs/evolution_rules.json:1)

```json
{
  "schema_version": 1,
  "model_id": "world-rules-v0.1",
  "world_width": 32.0,
  "world_height": 32.0,
  "grid_width": 32,
  "grid_height": 32,
  "physics_dt": 0.016666666666666666,
```

This file is declarative data rather than runtime classes/functions. Named fields configure validated rules, funded founder bodies/brains, and explicit costs/limits. Saved configuration includes resolved defaults.

</details>

<details><summary><code>crates/world_core/src/agents.rs</code></summary>

**Purpose:** Builds permitted local observations and validates whole neural-action batches before atomically advancing a native candidate.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/agents.rs)

**Imports / uses and why**

```rust
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
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `PortValue`**

Handles PortValue at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 19–24](C:/Thought/Projects/World/crates/world_core/src/agents.rs:19)

```rust
pub struct PortValue {
    pub port: u64,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize)]
```

**Struct: `OutputPort`**

Handles OutputPort at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 25–30](C:/Thought/Projects/World/crates/world_core/src/agents.rs:25)

```rust
pub struct OutputPort {
    pub port: u64,
    pub channel: u8,
    pub active: bool,
}

```

**Struct: `AgentObservation`**

Handles AgentObservation at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/src/agents.rs:32)

```rust
pub struct AgentObservation {
    pub id: OrganismId,
    pub version: u64,
    pub brain: NeuralGenome,
    pub inputs: Vec<PortValue>,
    pub outputs: Vec<OutputPort>,
```

**Struct: `AgentFrame`**

Handles AgentFrame at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 45–50](C:/Thought/Projects/World/crates/world_core/src/agents.rs:45)

```rust
pub struct AgentFrame {
    pub schema_version: u32,
    pub tick: u64,
    pub duration: f64,
    pub agents: Vec<AgentObservation>,
    pub online_learning: bool,
```

**Struct: `LearnedParameters`**

Carries validated live weights and biases by gene ID when a parent is eligible to pass them to a newborn.

[Source lines 56–61](C:/Thought/Projects/World/crates/world_core/src/agents.rs:56)

```rust
pub struct LearnedParameters {
    pub weights: Vec<(u64, f64)>,
    pub biases: Vec<(u64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
```

**Struct: `AgentAction`**

Handles AgentAction at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 63–68](C:/Thought/Projects/World/crates/world_core/src/agents.rs:63)

```rust
pub struct AgentAction {
    pub id: OrganismId,
    pub version: u64,
    pub values: Vec<PortValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learned: Option<LearnedParameters>,
```

**Struct: `ActionBatch`**

Handles ActionBatch at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 73–78](C:/Thought/Projects/World/crates/world_core/src/agents.rs:73)

```rust
pub struct ActionBatch {
    pub schema_version: u32,
    pub tick: u64,
    pub agents: Vec<AgentAction>,
}

```

**Struct: `Controls`**

Handles Controls at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 79–84](C:/Thought/Projects/World/crates/world_core/src/agents.rs:79)

```rust
pub(crate) struct Controls {
    pub values: BTreeMap<OrganismId, BTreeMap<u64, f64>>,
    pub authorized: BTreeSet<OrganismId>,
    pub learned: BTreeMap<OrganismId, LearnedParameters>,
}

```

**Struct: `Row`**

Handles Row at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 227–232](C:/Thought/Projects/World/crates/world_core/src/agents.rs:227)

```rust
        struct Row<'a> {
            id: OrganismId,
            version: u64,
            #[serde(skip_serializing_if = "Option::is_none")]
            brain: Option<&'a NeuralGenome>,
            inputs: &'a [PortValue],
```

**Methods in `impl Habitat`**

<details><summary><code>events_since_json()</code></summary>

Exports the incremental event receipt, rejecting a future cursor or already-evicted events.

[Source lines 196–201](C:/Thought/Projects/World/crates/world_core/src/agents.rs:196)

```rust
    pub fn events_since_json(&self, sequence: u64) -> Result<String> {
        let diagnostics = self
            .diagnostics
            .as_ref()
            .ok_or_else(|| AccountingError::Invalid("no evolution events".into()))?;
        if sequence > diagnostics.next_event
```

</details>

<details><summary><code>agent_frame_delta_json()</code></summary>

Omits unchanged neural genes by known organism/version identity while keeping current sensory observations.

[Source lines 222–227](C:/Thought/Projects/World/crates/world_core/src/agents.rs:222)

```rust
    pub fn agent_frame_delta_json(&self, known: &str) -> Result<String> {
        let versions: BTreeMap<u64, u64> =
            serde_json::from_str(known).map_err(|e| AccountingError::Invalid(e.to_string()))?;
        let frame = self.agent_frame()?;
        #[derive(Serialize)]
        struct Row<'a> {
```

</details>

<details><summary><code>agent_frame_json()</code></summary>

Supports agent frame json within this component's responsibility; the source link opens the complete implementation.

[Source lines 262–267](C:/Thought/Projects/World/crates/world_core/src/agents.rs:262)

```rust
    pub fn agent_frame_json(&self) -> Result<String> {
        serde_json::to_string(&self.agent_frame()?)
            .map_err(|e| AccountingError::Invalid(e.to_string()))
    }

    pub fn agent_frame(&self) -> Result<AgentFrame> {
```

</details>

<details><summary><code>agent_frame()</code></summary>

Builds bounded local/internal observations and actual port permissions at one decision boundary, without supplying global role/target instructions.

[Source lines 267–272](C:/Thought/Projects/World/crates/world_core/src/agents.rs:267)

```rust
    pub fn agent_frame(&self) -> Result<AgentFrame> {
        // Constructors and committed updates audit the private state.
        let config = self.config.evolution.as_ref().ok_or_else(|| {
            AccountingError::Invalid("habitat has no biological controller interface".into())
        })?;
        if !self
```

</details>

<details><summary><code>advance_controlled()</code></summary>

Validates the complete action cohort, advances a cloned native window, audits it, and replaces the world only after success.

[Source lines 512–517](C:/Thought/Projects/World/crates/world_core/src/agents.rs:512)

```rust
    pub fn advance_controlled(&mut self, text: &str) -> Result<()> {
        let batch: ActionBatch = serde_json::from_str(text)
            .map_err(|e| AccountingError::Invalid(format!("invalid neural actions: {e}")))?;
        let frame = self.agent_frame()?;
        let controls = validate_actions(&frame, &batch)?;
        let mut candidate = self.clone();
```

</details>

**Functions outside impl blocks**

<details><summary><code>circles()</code></summary>

Supports circles within this component's responsibility; the source link opens the complete implementation.

[Source lines 85–90](C:/Thought/Projects/World/crates/world_core/src/agents.rs:85)

```rust
pub fn circles(organism: &Organism, position: Position, heading: f64) -> Result<Vec<Circle>> {
    match (&organism.genome.body, &organism.body) {
        (Some(g), Some(b)) => b.circles(g, position, heading),
        _ => Ok(vec![Circle {
            id: 0,
            kind: ModuleKind::Core,
```

</details>

<details><summary><code>edible()</code></summary>

Supports edible within this component's responsibility; the source link opens the complete implementation.

[Source lines 99–104](C:/Thought/Projects/World/crates/world_core/src/agents.rs:99)

```rust
pub fn edible(consumer: &Organism, target: &Organism) -> bool {
    if !target.alive || consumer.id == target.id || !consumer.genome.can_ingest() {
        return false;
    }
    match consumer
        .genome
```

</details>

<details><summary><code>available_food()</code></summary>

Computes harvestable tissue after diet compatibility and declared inaccessible grazing tissue. Sensors and feeding contacts use the same availability rule.

[Source lines 116–121](C:/Thought/Projects/World/crates/world_core/src/agents.rs:116)

```rust
pub fn available_food(consumer: &Organism, target: &Organism, residual_fraction: f64) -> f64 {
    if !edible(consumer, target) {
        return 0.0;
    }
    let protected = if target.genome.photosynthetic()
        && consumer
```

</details>

<details><summary><code>port_active()</code></summary>

Supports port active within this component's responsibility; the source link opens the complete implementation.

[Source lines 134–139](C:/Thought/Projects/World/crates/world_core/src/agents.rs:134)

```rust
pub fn port_active(organism: &Organism, port: &Port) -> bool {
    if port.channel == 102 || port.channel == 18 {
        return false;
    }
    organism
        .body
```

</details>

<details><summary><code>neural_cost()</code></summary>

Supports neural cost within this component's responsibility; the source link opens the complete implementation.

[Source lines 145–150](C:/Thought/Projects/World/crates/world_core/src/agents.rs:145)

```rust
pub fn neural_cost(
    organism: &Organism,
    config: &crate::evolution_config::EvolutionConfig,
    duration: f64,
) -> f64 {
    organism.genome.brain.as_ref().map_or(0.0, |b| {
```

</details>

<details><summary><code>sensor_cost()</code></summary>

Supports sensor cost within this component's responsibility; the source link opens the complete implementation.

[Source lines 170–175](C:/Thought/Projects/World/crates/world_core/src/agents.rs:170)

```rust
pub fn sensor_cost(organism: &Organism, base: f64, duration: f64) -> f64 {
    if let Some(circle) = organism
        .genome
        .body
        .as_ref()
        .and_then(|b| b.circle.as_ref())
```

</details>

<details><summary><code>validate_actions()</code></summary>

Rejects stale, missing, duplicate, unknown, non-finite, out-of-range, or inactive action entries before any world change.

[Source lines 532–537](C:/Thought/Projects/World/crates/world_core/src/agents.rs:532)

```rust
fn validate_actions(frame: &AgentFrame, batch: &ActionBatch) -> Result<Controls> {
    if batch.schema_version != 1
        || batch.tick != frame.tick
        || batch.agents.len() != frame.agents.len()
    {
        return Err(AccountingError::Invalid(
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/diagnostics.rs</code></summary>

**Purpose:** Records finite expenditures, interaction/birth/death counters, and bounded sequenced events for export and explanation.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs)

**Imports / uses and why**

```rust
use crate::{
    ids::OrganismId,
    resources::{AccountingError, Result, finite},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use world_genetics::mutation::Edit;
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `EvolutionEvent`**

Handles EvolutionEvent at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:11)

```rust
pub struct EvolutionEvent {
    pub sequence: u64,
    pub tick: u64,
    pub kind: String,
    pub actor: OrganismId,
    pub target: Option<OrganismId>,
```

**Struct: `Diagnostics`**

Stores cumulative expenditure, role-specific birth/death outcomes, recent behaviour and a bounded event history inside complete native checkpoints.

[Source lines 25–30](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:25)

```rust
pub struct Diagnostics {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub behavior: BTreeMap<OrganismId, Behavior>,
    pub costs: BTreeMap<String, f64>,
    pub encounters: u64,
    pub blocked_moves: u64,
```

**Struct: `Behavior`**

Carries the last decision's local reward, actual locomotion and blocked-contact signal for sensing and learning feedback.

[Source lines 55–60](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:55)

```rust
pub struct Behavior {
    pub reward: f64,
    pub distance: f64,
    pub blocked: bool,
}

```

**Methods in `impl Diagnostics`**

<details><summary><code>birth_block()</code></summary>

Counts a rejected reproduction attempt both globally and by the parent's ecological role, preserving its reason.

[Source lines 62–67](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:62)

```rust
    pub fn birth_block(&mut self, role: &str, reason: &str) -> Result<()> {
        increment(&mut self.failed_births, 1)?;
        increment(self.birth_blocks.entry(reason.into()).or_default(), 1)?;
        increment(
            self.birth_blocks_by_role
                .entry(role.into())
```

</details>

<details><summary><code>cost()</code></summary>

Supports cost within this component's responsibility; the source link opens the complete implementation.

[Source lines 74–79](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:74)

```rust
    pub fn cost(&mut self, kind: &str, amount: f64) -> Result<()> {
        finite(amount, "expenditure", false)?;
        let value = self.costs.entry(kind.into()).or_default();
        *value += amount;
        finite(*value, "cumulative expenditure", false)
    }
```

</details>

<details><summary><code>event()</code></summary>

Supports event within this component's responsibility; the source link opens the complete implementation.

[Source lines 80–85](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:80)

```rust
    pub fn event(&mut self, mut event: EvolutionEvent, limit: usize) -> Result<()> {
        event.sequence = self.next_event;
        self.next_event = self
            .next_event
            .checked_add(1)
            .ok_or_else(|| AccountingError::Invalid("event counter overflow".into()))?;
```

</details>

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 92–97](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:92)

```rust
    pub fn validate(&self, limit: usize) -> Result<()> {
        for behavior in self.behavior.values() {
            if !behavior.reward.is_finite() {
                return Err(AccountingError::Invalid(
                    "non-finite learning feedback".into(),
                ));
```

</details>

**Functions outside impl blocks**

<details><summary><code>increment()</code></summary>

Supports increment within this component's responsibility; the source link opens the complete implementation.

[Source lines 128–133](C:/Thought/Projects/World/crates/world_core/src/diagnostics.rs:128)

```rust
pub(crate) fn increment(value: &mut u64, amount: u64) -> Result<()> {
    *value = value
        .checked_add(amount)
        .ok_or_else(|| AccountingError::Invalid("diagnostic counter overflow".into()))?;
    Ok(())
}
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/evolution_config.rs</code></summary>

**Purpose:** Defines explicit neural/body/attack/repair/construction costs, mutation limits, drag/work conversion, and history bounds.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/evolution_config.rs)

**Imports / uses and why**

```rust
use crate::resources::{AccountingError, Result, finite};
use serde::{Deserialize, Serialize};
use world_genetics::mutation::MutationConfig;
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `EvolutionConfig`**

Declares mutation settings, paid neural/body activity, repair and inheritance costs; validation rejects invalid rates and limits.

[Source lines 7–12](C:/Thought/Projects/World/crates/world_core/src/evolution_config.rs:7)

```rust
pub struct EvolutionConfig {
    pub online_learning: bool,
    pub maintenance_reserve_time: f64,
    pub mutation: MutationConfig,
    pub neuron_energy_per_time: f64,
    pub connection_energy_per_time: f64,
```

**Methods in `impl Default for EvolutionConfig`**

<details><summary><code>default()</code></summary>

Supports default within this component's responsibility; the source link opens the complete implementation.

[Source lines 27–32](C:/Thought/Projects/World/crates/world_core/src/evolution_config.rs:27)

```rust
    fn default() -> Self {
        Self {
            online_learning: false,
            maintenance_reserve_time: 0.0,
            mutation: MutationConfig::default(),
            neuron_energy_per_time: 0.002,
```

</details>

**Methods in `impl EvolutionConfig`**

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 50–55](C:/Thought/Projects/World/crates/world_core/src/evolution_config.rs:50)

```rust
    pub fn validate(&self) -> Result<()> {
        finite(
            self.plasticity_energy_per_connection_time,
            "plasticity cost",
            false,
        )?;
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_genetics/src/neural.rs</code></summary>

**Purpose:** Defines immutable birth neurons/connections and synchronizes body ports without replacing surviving inherited parameters.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```rust
use std::collections::BTreeSet;
use crate::body::Port;
use serde::{Deserialize, Serialize};
```

- `std` supplies maps/sets, paths/files, timing, and checked native utilities without extra crates.
- `serde` defines typed data serialization; strict records reject unsupported fields.
- `crate` imports neighboring native modules so biology reuses one accounting implementation.

**Structure / enum: `NodeKind`**

Distinguishes physical input nodes, recurrent hidden nodes, and physical output nodes.

[Source lines 6–12](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:6)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Input,
    Hidden,
    Output,
}
```

**Structure / enum: `NodeGene`**

An inherited neuron identity/kind, optional port, bias, and bounded per-decision leak coefficient.

[Source lines 14–21](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:14)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeGene {
    pub id: u64,
    pub kind: NodeKind,
    pub port: Option<u64>,
    pub bias: f64,
    pub leak: f64,
```

**Structure / enum: `ConnectionGene`**

An inherited edge identity, source/target neuron IDs, and encoded weight.

[Source lines 24–31](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:24)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionGene {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub weight: f64,
}
```

**Structure / enum: `NeuralGenome`**

A versioned sparse graph with unused identity counters; compatible genes survive local structural edits.

[Source lines 33–40](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:33)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeuralGenome {
    pub version: u64,
    pub next_node: u64,
    pub next_connection: u64,
    pub nodes: Vec<NodeGene>,
    pub connections: Vec<ConnectionGene>,
```

**Methods — open the one you are reading**

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 44–50](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:44)

```rust
    pub fn validate(&self, ports: &[Port]) -> Result<(), &'static str> {
        if self.nodes.len() > 4096 || self.connections.len() > 65536 || self.version == 0 {
            return Err("neural graph exceeds limits");
        }
        let mut ids = BTreeSet::new();
        let mut seen_ports = BTreeSet::new();
        for n in &self.nodes {
```

</details>

<details><summary><code>sync_ports()</code></summary>

Removes obsolete physical port nodes/edges and creates only missing ports; all surviving encoded parameters retain their values.

[Source lines 106–112](C:/Thought/Projects/World/crates/world_genetics/src/neural.rs:106)

```rust
    pub fn sync_ports(&mut self, ports: &[Port]) -> Result<(), &'static str> {
        self.nodes.retain(|n| {
            n.kind == NodeKind::Hidden
                || ports
                    .iter()
                    .any(|p| Some(p.id) == n.port && p.output == (n.kind == NodeKind::Output))
        });
```

</details>

**Useful syntax in this file**

- `?` returns a failure immediately so staged candidates can be discarded.
- `#[derive]` generates ordinary cloning/debug/serialization implementations.
- `Option<T>` represents an absent/present value; `Result<T>` carries success/error.
- `checked_add` rejects identity/counter overflow instead of wrapping.

</details>

<details><summary><code>crates/world_viewer/src/observer.rs</code></summary>

**Purpose:** Reads coordinated Python-owned world/neural publications and sends session-tagged commands; the viewer never advances that world itself.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs)

**Imports / uses and why**

```rust
use crate::model::Action;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use world_core::{
    habitat::Habitat,
    resources::{AccountingError, Result},
};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `Envelope`**

Handles Envelope at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 16–21](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:16)

```rust
struct Envelope {
    schema_version: u32,
    kind: String,
    session: String,
    sequence: u64,
    paused: bool,
```

**Struct: `Observed`**

Handles Observed at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 30–35](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:30)

```rust
pub struct Observed {
    pub habitat: Option<Habitat>,
    pub display: Option<world_core::display::DisplayFrame>,
    pub paused: bool,
    pub speed: u32,
    pub actual_speed: Option<f64>,
```

**Struct: `Observer`**

Handles Observer at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 39–44](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:39)

```rust
pub struct Observer {
    pub stream: PathBuf,
    pub commands: PathBuf,
    pub session: String,
    sequence: u64,
    command: u64,
```

**Methods in `impl Observer`**

<details><summary><code>open()</code></summary>

Supports open within this component's responsibility; the source link opens the complete implementation.

[Source lines 53–58](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:53)

```rust
    pub fn open(stream: PathBuf, commands: PathBuf) -> Result<(Self, Observed)> {
        let mut client = Self {
            stream,
            commands,
            session: String::new(),
            sequence: 0,
```

</details>

<details><summary><code>poll()</code></summary>

Supports poll within this component's responsibility; the source link opens the complete implementation.

[Source lines 72–77](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:72)

```rust
    pub fn poll(&mut self) -> Result<Option<Observed>> {
        if self.last_poll.elapsed() < Duration::from_millis(70) {
            return Ok(None);
        }
        self.last_poll = Instant::now();
        self.read()
```

</details>

<details><summary><code>read()</code></summary>

Supports read within this component's responsibility; the source link opens the complete implementation.

[Source lines 80–85](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:80)

```rust
    fn read(&mut self) -> Result<Option<Observed>> {
        let text = match fs::read_to_string(&self.stream) {
            Ok(text) => text,
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    && !self.session.is_empty() =>
```

</details>

<details><summary><code>save_published()</code></summary>

Saves the last validated coordinated owner publication rather than losing neural memory in a raw habitat export.

[Source lines 187–192](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:187)

```rust
    pub fn save_published(&self, path: &std::path::Path) -> Result<()> {
        if self.state.is_null() {
            return Err(AccountingError::Invalid(
                "display frames require an owner save command".into(),
            ));
        }
```

</details>

<details><summary><code>send()</code></summary>

Writes one unique session-tagged observer request; it does not apply actions directly to the displayed world copy.

[Source lines 196–201](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:196)

```rust
    pub fn send(&mut self, action: &Action, path: &std::path::Path) -> Result<()> {
        let kind = match action {
            Action::TogglePause => "pause",
            Action::Advance => "advance",
            Action::Faster => "faster",
            Action::Slower => "slower",
```

</details>

**Functions outside impl blocks**

<details><summary><code>observer_commands_and_bad_frames_cannot_mutate_the_published_world()</code></summary>

Supports observer commands and bad frames cannot mutate the published world within this component's responsibility; the source link opens the complete implementation.

[Source lines 238–243](C:/Thought/Projects/World/crates/world_viewer/src/observer.rs:238)

```rust
    fn observer_commands_and_bad_frames_cannot_mutate_the_published_world() {
        let world = Habitat::from_json(
            include_str!("../../../configs/evolution_rules.json"),
            include_str!("../../../configs/evolution_habitat.json"),
        )
        .unwrap();
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>examples/controller_comparison.py</code></summary>

**Purpose:** Runs fixed liquid candidates on the same local interface and reports inheritance/continuation and compute checks.

[Open the file](C:/Thought/Projects/World/examples/controller_comparison.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import argparse
import json
from pathlib import Path
from world.brains.candidates import compare_candidates
from world.brains.evolving import NeuralPopulation
from world.ecosystem import EcosystemRuntime, write_json
```

- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `time` measures/paces wall-clock work; native model time remains tick-based.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `argparse` validates user-facing run/device/seed/output options.

**Standalone functions**

<details><summary><code>main()</code></summary>

Validates CLI inputs, runs this file's documented workflow, and writes its declared outputs/errors.

[Source lines 10–16](C:/Thought/Projects/World/examples/controller_comparison.py:10)

```python
def main():
    parser = argparse.ArgumentParser(description="Compare the sparse recurrent interface with fixed wired liquid candidates.")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--iterations", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("artifacts/controller-comparison.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
```

</details>

**Useful syntax in this file**


</details>

<details><summary><code>examples/ecology_diagnostics.py</code></summary>

**Purpose:** Provides the CLI for the original consumer-extinction report before coefficient calibration.

[Open the file](C:/Thought/Projects/World/examples/ecology_diagnostics.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import argparse
import json
from pathlib import Path
from world.ecosystem import write_json
from world.experiments.diagnostics import diagnose_legacy
```

- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `argparse` validates user-facing run/device/seed/output options.

**Standalone functions**

<details><summary><code>main()</code></summary>

Validates CLI inputs, runs this file's documented workflow, and writes its declared outputs/errors.

[Source lines 9–15](C:/Thought/Projects/World/examples/ecology_diagnostics.py:9)

```python
def main():
    parser = argparse.ArgumentParser(description="Inspect consumer extinction in the original diagnostic ecosystem.")
    parser.add_argument("--ticks", type=int, default=12000)
    parser.add_argument("--sample-every", type=int, default=300)
    parser.add_argument("--output", type=Path, default=Path("artifacts/consumer-diagnostics.json"))
    args = parser.parse_args()
    if args.ticks < 0 or args.sample_every < 1:
```

</details>

**Useful syntax in this file**


</details>

<details><summary><code>examples/evolution_demo.py</code></summary>

**Purpose:** Runs headless or interactive inherited neural ecology, exports sequenced events, and saves coordinated state on completion.

[Open the file](C:/Thought/Projects/World/examples/evolution_demo.py)

**Imports / uses and why**

```python
import argparse
import json
from pathlib import Path
import subprocess
import time
from world.control import RunControl, RunLease
from world.ecosystem import EcosystemRuntime, write_json
from world.telemetry import RunRecorder
```

- `argparse`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `json`: Reads and writes the versioned configuration and state protocols.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `subprocess`: Starts a child command with an argument list and captures its output and exit status.
- `time`: Measures wall time for pacing and achieved-speed reporting, separately from biological model time.
- `world`: Imports this project's native world, controller coordination, or experiment helpers.

**Functions and nested helpers**

<details><summary><code>main()</code></summary>

Validates CLI inputs, runs this file's documented workflow, and writes its declared outputs/errors.

[Source lines 15–21](C:/Thought/Projects/World/examples/evolution_demo.py:15)

```python
def main():
    parser = argparse.ArgumentParser(
        description="Run inherited neural organisms and birth-time evolution."
    )
    parser.add_argument(
        "--rules", type=Path, default=ROOT / "configs/circle_rules.json"
    )
```

</details>

<details><summary><code>run()</code></summary>

Coordinates this executable operation, its declared inputs, and its saved or displayed outputs.

[Source lines 61–67](C:/Thought/Projects/World/examples/evolution_demo.py:61)

```python
def run(args):
    executable = ROOT / "target/debug/world_viewer.exe"
    if args.viewer and not executable.exists():
        raise FileNotFoundError(
            "viewer executable is missing; run scripts/run_evolution.ps1"
        )
    if args.resume:
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>python/world/brains/candidates.py</code></summary>

**Purpose:** Compares fixed wired CfC/LTC interfaces, inherited parameters, memory reset, and continuation; these are separate from the topology-evolving baseline.

[Open the file](C:/Thought/Projects/World/python/world/brains/candidates.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
from copy import deepcopy
import math
import time
import torch
from ncps.torch import CfC, LTC
from ncps.wirings import FullyConnected
```

- `torch` supplies tensor execution, sparse indexed sums, device transfers, and explicit recurrent state.
- `ncps` supplies the fixed wired CfC/LTC candidates used in separate interface/continuation experiments.
- `math` supplies geometry/probability arithmetic and an independent scalar recurrence reference.
- `copy` creates independent configs/genomes or reuses immutable graph arrays while assigning new memory.
- `time` measures/paces wall-clock work; native model time remains tick-based.

**Class: `LiquidCandidate`**

A fixed wired ncps controller with named parameters and independent recurrent memory, used for candidate checks.

[Source lines 10–12](C:/Thought/Projects/World/python/world/brains/candidates.py:10)

```python
class LiquidCandidate:
    def __init__(self, family, inputs, outputs, units=16, seed=73, device="cpu"):
        if family not in {"cfc", "ltc"} or inputs < 1 or outputs < 1 or units < outputs:
```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Initializes this object's owned state and its identity/layout mappings; later methods preserve those invariants.

[Source lines 11–17](C:/Thought/Projects/World/python/world/brains/candidates.py:11)

```python
    def __init__(self, family, inputs, outputs, units=16, seed=73, device="cpu"):
        if family not in {"cfc", "ltc"} or inputs < 1 or outputs < 1 or units < outputs:
            raise ValueError("invalid liquid candidate interface")
        self.spec = {"family": family, "inputs": inputs, "outputs": outputs, "units": units, "seed": seed}
        self.device = torch.device(device)
        if self.device.type not in {"cpu", "cuda"} or self.device.type == "cuda" and not torch.cuda.is_available():
            raise ValueError("unsupported or unavailable candidate device")
```

</details>

<details><summary><code>propose()</code></summary>

Computes candidate outputs and memory without committing them; the owner accepts them only after native validation.

[Source lines 28–34](C:/Thought/Projects/World/python/world/brains/candidates.py:28)

```python
    @torch.inference_mode()
    def propose(self, agent, duration):
        values = [v["value"] for v in agent["inputs"]]
        if len(values) != self.spec["inputs"] or len(agent["outputs"]) != self.spec["outputs"] or not math.isfinite(duration) or duration <= 0:
            raise ValueError("candidate input/output interface mismatch")
        if not agent["authorized"]:
            return [{"port": p["port"], "value": 0.0} for p in agent["outputs"]], self.memory
```

</details>

<details><summary><code>snapshot()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 50–54](C:/Thought/Projects/World/python/world/brains/candidates.py:50)

```python
    def snapshot(self):
        return {"schema_version": 1, "kind": "fixed_liquid_candidate", "spec": deepcopy(self.spec),
                "parameters": {name: {"shape": list(value.shape), "values": value.cpu().tolist()}
                               for name, value in self.model.state_dict().items()},
                "memory": None if self.memory is None else self.memory.cpu().tolist()}
```

</details>

<details><summary><code>restore()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 56–62](C:/Thought/Projects/World/python/world/brains/candidates.py:56)

```python
    @classmethod
    def restore(cls, payload, device="cpu", newborn=False):
        if not isinstance(payload, dict) or set(payload) != {"schema_version", "kind", "spec", "parameters", "memory"} or type(payload["schema_version"]) is not int or payload["schema_version"] != 1 or payload["kind"] != "fixed_liquid_candidate":
            raise ValueError("invalid liquid candidate checkpoint")
        if set(payload["spec"]) != {"family", "inputs", "outputs", "units", "seed"}:
            raise ValueError("invalid liquid candidate specification")
        result = cls(**payload["spec"], device=device)
```

</details>

**Standalone functions**

<details><summary><code>compare_candidates()</code></summary>

Feeds the same permitted ports to fixed liquid candidates and checks inherited parameters, newborn reset, continuation, bounded output, and completed compute timing.

[Source lines 82–88](C:/Thought/Projects/World/python/world/brains/candidates.py:82)

```python
def compare_candidates(frame, iterations=100, device="cpu"):
    if iterations < 1 or not frame["agents"]:
        raise ValueError("candidate comparison needs active observations and positive iterations")
    agent = frame["agents"][0]
    reports = []
    for family in ["cfc", "ltc"]:
        candidate = LiquidCandidate(family, len(agent["inputs"]), len(agent["outputs"]), units=16, device=device)
```

</details>

**Useful syntax in this file**

- `@classmethod` reconstructs an instance through the class without creating fresh founders.
- `inference_mode` avoids retaining lifetime autograd graphs; this controller has no lifetime weight-learning rule yet.

</details>

<details><summary><code>python/world/brains/evolving.py</code></summary>

**Purpose:** Executes ragged inherited neural graphs as flat sparse PyTorch tensors and preserves memory by organism/neuron identity.

[Open the file](C:/Thought/Projects/World/python/world/brains/evolving.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import math
from copy import copy
import torch
```

- `torch` supplies tensor execution, sparse indexed sums, device transfers, and explicit recurrent state.
- `math` supplies geometry/probability arithmetic and an independent scalar recurrence reference.
- `copy` creates independent configs/genomes or reuses immutable graph arrays while assigning new memory.

**Class: `NeuralPopulation`**

Owns flat ragged PyTorch graph arrays and recurrent memory, keyed by organism and neuron IDs.

[Source lines 7–9](C:/Thought/Projects/World/python/world/brains/evolving.py:7)

```python
class NeuralPopulation:
    kind = "sparse_leaky_coo_v1"

```

**Methods — open the one you are reading**

<details><summary><code>__init__()</code></summary>

Initializes this object's owned state and its identity/layout mappings; later methods preserve those invariants.

[Source lines 10–16](C:/Thought/Projects/World/python/world/brains/evolving.py:10)

```python
    def __init__(self, frame: dict, device="cpu", previous=None, memory=None):
        self.device = torch.device(device)
        if self.device.type not in {"cpu", "cuda"}:
            raise ValueError("supported neural devices are cpu and cuda")
        if self.device.type == "cuda" and not torch.cuda.is_available():
            raise RuntimeError("CUDA was requested but is unavailable")
        self.tick = frame["tick"]
```

</details>

<details><summary><code>_signature()</code></summary>

Identifies the exact cohort, graph layout, and encoded parameters so cached tensors cannot be reused for another graph.

[Source lines 56–59](C:/Thought/Projects/World/python/world/brains/evolving.py:56)

```python
    @staticmethod
    def _signature(frame):
        return tuple((a["id"], a["version"], tuple((n["id"], n["kind"], n["port"], n["bias"], n["leak"]) for n in a["brain"]["nodes"]),
                      tuple((e["id"], e["source"], e["target"], e["weight"]) for e in a["brain"]["connections"])) for a in frame["agents"])
```

</details>

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 61–65](C:/Thought/Projects/World/python/world/brains/evolving.py:61)

```python
    def validate(self, frame):
        if frame["schema_version"] != 1 or frame["tick"] != self.tick or self._signature(frame) != self.signature:
            raise ValueError("neural population does not match world tick/genomes")
        if self.memory.shape != (len(self.keys),) or not torch.isfinite(self.memory).all() or torch.any(self.memory.abs() > 1):
            raise ValueError("invalid recurrent state")
```

</details>

<details><summary><code>propose()</code></summary>

Computes candidate outputs and memory without committing them; the owner accepts them only after native validation.

[Source lines 67–73](C:/Thought/Projects/World/python/world/brains/evolving.py:67)

```python
    @torch.inference_mode()
    def propose(self, frame):
        self.validate(frame)
        previous = self.memory.clone()
        indices, observations = [], []
        authorized = torch.zeros(len(self.keys), dtype=torch.bool, device=self.device)
        for agent, (_, begin, end, inputs, _) in zip(frame["agents"], self.rows, strict=True):
```

</details>

<details><summary><code>reconcile()</code></summary>

Keeps compatible surviving recurrent values by ID, drops dead/deleted identities, and initializes only newly born/added nodes to zero.

[Source lines 105–111](C:/Thought/Projects/World/python/world/brains/evolving.py:105)

```python
    def reconcile(self, frame, memory):
        if self._signature(frame) == self.signature:
            result = copy(self)
            result.tick = frame["tick"]
            result.memory = memory
            result.validate(frame)
            return result
```

</details>

<details><summary><code>snapshot()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 116–118](C:/Thought/Projects/World/python/world/brains/evolving.py:116)

```python
    def snapshot(self):
        return {"schema_version": 1, "kind": self.kind, "tick": self.tick,
                "keys": [list(key) for key in self.keys], "memory": self.memory.cpu().tolist()}
```

</details>

<details><summary><code>restore()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 120–126](C:/Thought/Projects/World/python/world/brains/evolving.py:120)

```python
    @classmethod
    def restore(cls, frame, payload, device="cpu"):
        if not isinstance(payload, dict) or set(payload) != {"schema_version", "kind", "tick", "keys", "memory"}:
            raise ValueError("unexpected neural checkpoint fields")
        if type(payload["schema_version"]) is not int or payload["schema_version"] != 1 or payload["kind"] != cls.kind or type(payload["tick"]) is not int or payload["tick"] != frame["tick"]:
            raise ValueError("unsupported or mismatched neural checkpoint")
        result = cls(frame, device)
```

</details>

**Standalone functions**

<details><summary><code>reference_update()</code></summary>

Independently evaluates one recurrent graph with Python scalar arithmetic to check the sparse tensor implementation.

[Source lines 140–146](C:/Thought/Projects/World/python/world/brains/evolving.py:140)

```python
def reference_update(agent, memory):
    previous = {n["id"]: memory.get(n["id"], 0.0) for n in agent["brain"]["nodes"]}
    if not agent["authorized"]:
        return previous
    inputs = {v["port"]: v["value"] for v in agent["inputs"]}
    for node in agent["brain"]["nodes"]:
        if node["kind"] == "input":
```

</details>

**Useful syntax in this file**

- `@classmethod` reconstructs an instance through the class without creating fresh founders.
- `inference_mode` avoids retaining lifetime autograd graphs; this controller has no lifetime weight-learning rule yet.
- `index_add_` sums sparse edge contributions at destination neuron indices.

</details>

<details><summary><code>python/world/control.py</code></summary>

**Purpose:** Owns real-time pacing and observer commands. It locks each run directory, chooses a compatible observation protocol, and saves the complete coordinated learning state.

[Open the file](C:/Thought/Projects/World/python/world/control.py)

**Imports / uses and why**

```python
from pathlib import Path
import os
import time
import uuid
from .ecosystem import EcosystemRuntime, read_json, write_json
                import msvcrt
                import fcntl
                import msvcrt
                import fcntl
```

- `ecosystem`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `fcntl`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `msvcrt`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `os`: Selects the operating-system locking primitive for exclusive run ownership.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `time`: Measures wall time for pacing and achieved-speed reporting, separately from biological model time.
- `uuid`: Gives each observer session a distinct identity so stale commands are rejected.

**Class: `RunLease`**

Holds an operating-system file lock so two processes cannot overwrite the same run's stream and checkpoints.

[Source lines 9–15](C:/Thought/Projects/World/python/world/control.py:9)

```python
class RunLease:
    def __init__(self, stream):
        self.path = Path(str(stream) + ".lock")
        self.file = None

    def __enter__(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
```

<details><summary><code>__init__()</code></summary>

Initializes this object's owned state and its identity/layout mappings; later methods preserve those invariants.

[Source lines 10–12](C:/Thought/Projects/World/python/world/control.py:10)

```python
    def __init__(self, stream):
        self.path = Path(str(stream) + ".lock")
        self.file = None
```

</details>

<details><summary><code>__enter__()</code></summary>

Acquires exclusive ownership of the run before output files are written; competing owners receive an error.

[Source lines 14–20](C:/Thought/Projects/World/python/world/control.py:14)

```python
    def __enter__(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.file = self.path.open("a+b")
        self.file.seek(0, 2)
        if not self.file.tell():
            self.file.write(b"0")
            self.file.flush()
```

</details>

<details><summary><code>__exit__()</code></summary>

Releases the operating-system lock and closes its file, including when the run exits through an exception.

[Source lines 38–44](C:/Thought/Projects/World/python/world/control.py:38)

```python
    def __exit__(self, *error):
        try:
            self.file.seek(0)
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(self.file.fileno(), msvcrt.LK_UNLCK, 1)
```

</details>

**Class: `RunControl`**

Owns pacing, pause/single-decision commands, compatible observer publication and coordinated save/load. Camera operations do not advance biology.

[Source lines 53–59](C:/Thought/Projects/World/python/world/control.py:53)

```python
class RunControl:
    def __init__(
        self,
        runtime,
        checkpoint,
        stream=None,
        commands=None,
```

<details><summary><code>__init__()</code></summary>

Initializes this object's owned state and its identity/layout mappings; later methods preserve those invariants.

[Source lines 54–60](C:/Thought/Projects/World/python/world/control.py:54)

```python
    def __init__(
        self,
        runtime,
        checkpoint,
        stream=None,
        commands=None,
        paused=False,
```

</details>

<details><summary><code>publish()</code></summary>

Writes a complete world/neural publication plus owner session/status; publication timing does not change biological time.

[Source lines 90–96](C:/Thought/Projects/World/python/world/control.py:90)

```python
    def publish(self):
        if self.stream:
            now = time.perf_counter()
            elapsed = now - self._rate_wall
            model_time = self.runtime.elapsed_time
            if self.paused or model_time < self._rate_time:
                self.actual_speed = 0.0
```

</details>

<details><summary><code>apply()</code></summary>

Supports apply within this component's responsibility; the source link opens the complete implementation.

[Source lines 120–126](C:/Thought/Projects/World/python/world/control.py:120)

```python
    def apply(self, command):
        if not isinstance(command, dict) or set(command) != {
            "schema_version",
            "session",
            "id",
            "kind",
            "path",
```

</details>

<details><summary><code>receive()</code></summary>

Consumes session-tagged commands, preserves the current world on failure, and reports rejection to the observer.

[Source lines 199–205](C:/Thought/Projects/World/python/world/control.py:199)

```python
    def receive(self):
        if not self.commands:
            return
        for path in sorted(self.commands.glob("cmd-*.json")):
            try:
                command = read_json(path)
                if command.get("session") != self.session:
```

</details>

<details><summary><code>update()</code></summary>

Supports update within this component's responsibility; the source link opens the complete implementation.

[Source lines 214–220](C:/Thought/Projects/World/python/world/control.py:214)

```python
    def update(self, now=None):
        self.receive()
        if now is not None:
            if self._next_decision_wall is None or self._pacing_speed != self.speed:
                self._next_decision_wall = now
                self._pacing_speed = self.speed
            if self.paused and not self.single:
```

</details>

<details><summary><code>write_interventions()</code></summary>

Supports write interventions within this component's responsibility; the source link opens the complete implementation.

[Source lines 237–243](C:/Thought/Projects/World/python/world/control.py:237)

```python
    def write_interventions(self, path):
        write_json(
            path,
            {
                "schema_version": 1,
                "session": self.session,
                "recorded_at_unix": time.time(),
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>python/world/ecosystem.py</code></summary>

**Purpose:** Coordinates the native world and its PyTorch controller as one transaction. It selects fixed or plastic brains, caches unchanged genes, restores checkpoints, and publishes current learned inspection data.

[Open the file](C:/Thought/Projects/World/python/world/ecosystem.py)

**Imports / uses and why**

```python
import json
import os
from pathlib import Path
import tempfile
import time
from ._native import Habitat as NativeHabitat, core_version
from .brains.evolving import NeuralPopulation
from .brains.plastic import PlasticPopulation
```

- `_native`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `brains`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `json`: Reads and writes the versioned configuration and state protocols.
- `os`: Selects the operating-system locking primitive for exclusive run ownership.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `tempfile`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `time`: Measures wall time for pacing and achieved-speed reporting, separately from biological model time.

**Class: `EcosystemRuntime`**

Owns the native world and independent neural population together, so a failed world or learning operation commits neither half.

[Source lines 52–58](C:/Thought/Projects/World/python/world/ecosystem.py:52)

```python
class EcosystemRuntime:
    def __init__(self, world, brains):
        self.world = world
        self.brains = brains
        self._frame = self.frame()
        self.brains.validate(self._frame)

```

<details><summary><code>__init__()</code></summary>

Initializes this object's owned state and its identity/layout mappings; later methods preserve those invariants.

[Source lines 53–57](C:/Thought/Projects/World/python/world/ecosystem.py:53)

```python
    def __init__(self, world, brains):
        self.world = world
        self.brains = brains
        self._frame = self.frame()
        self.brains.validate(self._frame)
```

</details>

<details><summary><code>create()</code></summary>

Supports create within this component's responsibility; the source link opens the complete implementation.

[Source lines 60–66](C:/Thought/Projects/World/python/world/ecosystem.py:60)

```python
    def create(cls, rules, config, device="cpu"):
        world = NativeHabitat(
            json.dumps(rules, allow_nan=False), json.dumps(config, allow_nan=False)
        )
        frame = json.loads(world.agent_frame_json())
        return cls(
            world,
```

</details>

<details><summary><code>tick()</code></summary>

Supports tick within this component's responsibility; the source link opens the complete implementation.

[Source lines 73–74](C:/Thought/Projects/World/python/world/ecosystem.py:73)

```python
    def tick(self):
        return self.world.tick
```

</details>

<details><summary><code>decision_duration()</code></summary>

Reads the fixed biological decision duration directly, avoiding a complete observation/audit merely to pace the display loop.

[Source lines 77–78](C:/Thought/Projects/World/python/world/ecosystem.py:77)

```python
    def decision_duration(self):
        return self.world.decision_duration
```

</details>

<details><summary><code>elapsed_time()</code></summary>

Reads authoritative elapsed model time for measured wall-clock speed; camera frames do not define biological time.

[Source lines 81–82](C:/Thought/Projects/World/python/world/ecosystem.py:81)

```python
    def elapsed_time(self):
        return self.world.elapsed_time
```

</details>

<details><summary><code>next_event()</code></summary>

Reads the monotonic event cursor without serializing the complete diagnostic history before every decision.

[Source lines 85–86](C:/Thought/Projects/World/python/world/ecosystem.py:85)

```python
    def next_event(self):
        return self.world.next_event
```

</details>

<details><summary><code>frame()</code></summary>

Supports frame within this component's responsibility; the source link opens the complete implementation.

[Source lines 88–89](C:/Thought/Projects/World/python/world/ecosystem.py:88)

```python
    def frame(self):
        return json.loads(self.world.agent_frame_json())
```

</details>

<details><summary><code>advance()</code></summary>

Runs the requested complete decision work while staging native and neural candidates before joint acceptance.

[Source lines 91–97](C:/Thought/Projects/World/python/world/ecosystem.py:91)

```python
    def advance(self, decisions=1):
        if type(decisions) is not int or decisions < 0:
            raise ValueError("decisions must be a nonnegative integer")
        for _ in range(decisions):
            frame = self._frame
            if frame["tick"] != self.tick:
                raise ValueError("world was advanced outside the coordinator")
```

</details>

<details><summary><code>summary()</code></summary>

Supports summary within this component's responsibility; the source link opens the complete implementation.

[Source lines 120–121](C:/Thought/Projects/World/python/world/ecosystem.py:120)

```python
    def summary(self):
        return json.loads(self.world.summary_json())
```

</details>

<details><summary><code>telemetry()</code></summary>

Returns population roles, reserve/tissue compartments, local mineral access and diagnostic counters without modifying the world or including neural genomes.

[Source lines 123–124](C:/Thought/Projects/World/python/world/ecosystem.py:123)

```python
    def telemetry(self):
        return json.loads(self.world.telemetry_json())
```

</details>

<details><summary><code>events_since()</code></summary>

Reads only events newer than a declared cursor. It rejects a history gap instead of silently dropping events.

[Source lines 126–127](C:/Thought/Projects/World/python/world/ecosystem.py:126)

```python
    def events_since(self, sequence):
        return json.loads(self.world.events_since_json(sequence))
```

</details>

<details><summary><code>audit()</code></summary>

Supports audit within this component's responsibility; the source link opens the complete implementation.

[Source lines 129–130](C:/Thought/Projects/World/python/world/ecosystem.py:129)

```python
    def audit(self):
        return json.loads(self.world.audit_json())
```

</details>

<details><summary><code>display()</code></summary>

Adds a bounded view of the selected organism's current activations and learned weights to the native rendering packet.

[Source lines 132–138](C:/Thought/Projects/World/python/world/ecosystem.py:132)

```python
    def display(self, selected=None):
        payload = json.loads(self.world.display_json(selected))
        if selected is not None and isinstance(self.brains, PlasticPopulation):
            agent = next(
                (a for a in self._frame["agents"] if a["id"] == selected), None
            )
            if agent:
```

</details>

<details><summary><code>snapshot()</code></summary>

Serializes the complete coordinated state needed to resume, including live learned parameters and learning history rather than only birth genes.

[Source lines 186–192](C:/Thought/Projects/World/python/world/ecosystem.py:186)

```python
    def snapshot(self):
        world = json.loads(self.world.checkpoint_json())
        agents = [
            {
                "id": o["id"],
                "version": o["genome"]["brain"]["version"],
                "brain": o["genome"]["brain"],
```

</details>

<details><summary><code>save()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 206–207](C:/Thought/Projects/World/python/world/ecosystem.py:206)

```python
    def save(self, path):
        write_json(path, self.snapshot())
```

</details>

<details><summary><code>from_snapshot()</code></summary>

Validates saved state before creating a runtime. Malformed shapes, identities, bounds, or controller/world mismatches are rejected.

[Source lines 210–216](C:/Thought/Projects/World/python/world/ecosystem.py:210)

```python
    def from_snapshot(cls, payload, device="cpu"):
        if not isinstance(payload, dict) or set(payload) != {
            "schema_version",
            "kind",
            "engine_version",
            "world",
            "neural",
```

</details>

<details><summary><code>load()</code></summary>

Serializes or reconstructs this component's complete declared state, validating compatibility and identity before continuation.

[Source lines 238–239](C:/Thought/Projects/World/python/world/ecosystem.py:238)

```python
    def load(cls, path, device="cpu"):
        return cls.from_snapshot(read_json(path), device)
```

</details>

**Functions and nested helpers**

<details><summary><code>write_json()</code></summary>

Serializes data before creating a sibling temporary file, flushes it, and atomically replaces the destination with bounded Windows sharing retries.

[Source lines 12–18](C:/Thought/Projects/World/python/world/ecosystem.py:12)

```python
def write_json(path, payload):
    text = json.dumps(payload, allow_nan=False, separators=(",", ":"))
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
```

</details>

<details><summary><code>read_json()</code></summary>

Reads data-only JSON, retrying short Windows read-sharing conflicts while leaving format errors explicit.

[Source lines 42–48](C:/Thought/Projects/World/python/world/ecosystem.py:42)

```python
def read_json(path):
    for attempt in range(40):
        try:
            return json.loads(Path(path).read_text(encoding="utf-8"))
        except PermissionError:
            if os.name != "nt" or attempt == 39:
                raise
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>python/world/experiments/__init__.py</code></summary>

**Purpose:** Provides this component's configuration/build inputs.

[Open the file](C:/Thought/Projects/World/python/world/experiments/__init__.py)

[Source lines 1–1](C:/Thought/Projects/World/python/world/experiments/__init__.py:1)

```python
"""Seeded ecological controls and candidate comparisons."""
```

These declarations expose modules or describe the package; no independent runtime algorithm is defined here.

**Useful syntax in this file**


</details>

<details><summary><code>python/world/experiments/diagnostics.py</code></summary>

**Purpose:** Measures the original scripted consumer decline with last living states, accessible-food distances, and explicit energy-flow estimates.

[Open the file](C:/Thought/Projects/World/python/world/experiments/diagnostics.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import math
from ..habitat import Habitat
```

- `math` supplies geometry/probability arithmetic and an independent scalar recurrence reference.

**Standalone functions**

<details><summary><code>inspect_consumers()</code></summary>

Collects observer-only last-living states and food distances for diagnosis, not for neural input.

[Source lines 6–12](C:/Thought/Projects/World/python/world/experiments/diagnostics.py:6)

```python
def inspect_consumers(snapshot, perception, feeding):
    organisms = snapshot["organisms"]
    producers = [r for r in organisms if r["organism"]["alive"] and r["organism"]["genome"]["photosynthetic"]]
    rows = []
    for record in organisms:
        organism = record["organism"]
        if not organism["genome"]["can_ingest"]:
```

</details>

<details><summary><code>diagnose_legacy()</code></summary>

Measures the original scripted consumer decline and actual energy-flow estimates without silently changing its coefficients.

[Source lines 24–30](C:/Thought/Projects/World/python/world/experiments/diagnostics.py:24)

```python
def diagnose_legacy(rules, config, ticks=3000, sample_every=300):
    habitat = Habitat(rules, config)
    perception = config["ecology"]["perception_radius"]
    feeding = config["ecology"]["feeding_radius"]
    last_alive, death_records, samples = {}, {}, []
    duration = rules["physics_dt"] * rules["ecology_interval_ticks"]
    while habitat.tick < ticks:
```

</details>

**Useful syntax in this file**


</details>

<details><summary><code>python/world/presets.py</code></summary>

**Purpose:** Builds reproducible founder genomes and declared experiments. The default circle founders have mutable food, contact, and wall reflexes; legacy compound presets remain available.

[Open the file](C:/Thought/Projects/World/python/world/presets.py)

**Imports / uses and why**

```python
import math
from copy import deepcopy
```

- `copy`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `math`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Functions and nested helpers**

<details><summary><code>body_blueprint()</code></summary>

Builds a declared functional founder tree with explicit geometry, investment, and food compatibility.

[Source lines 5–11](C:/Thought/Projects/World/python/world/presets.py:5)

```python
def body_blueprint(kind="grazer"):
    def module(identity, parent, category, radius, angle=0.0, investment=1.0):
        return {
            "id": identity,
            "parent": parent,
            "kind": category,
            "radius": radius,
```

</details>

<details><summary><code>module()</code></summary>

Supports module within this component's responsibility; the source link opens the complete implementation.

[Source lines 6–12](C:/Thought/Projects/World/python/world/presets.py:6)

```python
    def module(identity, parent, category, radius, angle=0.0, investment=1.0):
        return {
            "id": identity,
            "parent": parent,
            "kind": category,
            "radius": radius,
            "angle": angle,
```

</details>

<details><summary><code>body_ports()</code></summary>

Mirrors the native module/channel vocabulary when constructing founder neural genes; native validation checks agreement.

[Source lines 47–53](C:/Thought/Projects/World/python/world/presets.py:47)

```python
def body_ports(body):
    if body.get("circle"):
        root = body["modules"][0]["id"]
        return [
            {"id": root * 256 + c, "channel": c, "module": root, "output": out}
            for out, channels in [
                (False, [0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20]),
```

</details>

<details><summary><code>neural_blueprint()</code></summary>

Encodes stable founder neurons/edges, sparse large-graph initialization, and labeled reflex seeds without randomizing offspring controllers.

[Source lines 80–86](C:/Thought/Projects/World/python/world/presets.py:80)

```python
def neural_blueprint(body, hidden=8, seed=73, kind="grazer"):
    if type(hidden) is not int or not 0 <= hidden <= 3000:
        raise ValueError("hidden neuron count is out of bounds")
    counter = seed & (2**64 - 1)

    def unit():
        nonlocal counter
```

</details>

<details><summary><code>unit()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 85–91](C:/Thought/Projects/World/python/world/presets.py:85)

```python
    def unit():
        nonlocal counter
        counter = (counter + 0x9E3779B97F4A7C15) & (2**64 - 1)
        value = counter
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & (2**64 - 1)
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & (2**64 - 1)
        return ((value ^ (value >> 31)) >> 11) / 9007199254740992
```

</details>

<details><summary><code>edge()</code></summary>

Supports edge within this component's responsibility; the source link opens the complete implementation.

[Source lines 132–138](C:/Thought/Projects/World/python/world/presets.py:132)

```python
    def edge(source, target, weight):
        existing = edge_lookup.get((source, target))
        if existing:
            existing["weight"] = weight
        else:
            item = {
                "id": len(connections) + 1,
```

</details>

<details><summary><code>founder_genome()</code></summary>

Combines the chosen body and optional founder brain into the native inherited-genome format.

[Source lines 203–209](C:/Thought/Projects/World/python/world/presets.py:203)

```python
def founder_genome(kind="grazer", hidden=8, seed=73, neural=True):
    body = body_blueprint(kind)
    result = {
        "photosynthetic": kind == "producer",
        "can_ingest": kind != "producer",
        "initial_weights": [],
        "body": body,
```

</details>

<details><summary><code>evolution_preset()</code></summary>

Builds differentiated starting populations, explicit mutation settings, and exactly funded world inventories.

[Source lines 216–222](C:/Thought/Projects/World/python/world/presets.py:216)

```python
def evolution_preset(rules, seed=73, mutation=True, structural=True):
    rules = deepcopy(rules)
    rules.update(world_width=32.0, world_height=32.0, grid_width=32, grid_height=32)
    groups = []
    for index, (kind, count, tissue, nutrient, energy) in enumerate(
        [
            ("producer", 48, 8.0, 3.0, 25.0),
```

</details>

<details><summary><code>circle_genome()</code></summary>

Creates a single-core body and optional sparse recurrent brain. Initial reflexes are explicit mutable genes, not evidence of learned behavior.

[Source lines 278–284](C:/Thought/Projects/World/python/world/presets.py:278)

```python
def circle_genome(producer=False, hidden=8, seed=73, neural=True, learning_rate=0.04):
    body = {
        "modules": [
            {
                "id": 1,
                "parent": None,
                "kind": "core",
```

</details>

<details><summary><code>circle_preset()</code></summary>

Builds the declared circle experiment with finite nutrients, external light, paid online learning, funded births, and bounded population.

[Source lines 313–319](C:/Thought/Projects/World/python/world/presets.py:313)

```python
def circle_preset(rules, seed=73, consumers=20, producers=80):
    rules = deepcopy(rules)
    rules.update(
        model_id="circle-world-v0.3",
        world_width=40.0,
        world_height=22.5,
        grid_width=40,
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>scripts/run_evolution.ps1</code></summary>

**Purpose:** Audits/builds cached native inputs, launches the Python owner and observer, and restores temporary process settings.

[Open the file](C:/Thought/Projects/World/scripts/run_evolution.ps1)

[Source lines 1–6](C:/Thought/Projects/World/scripts/run_evolution.ps1:1)

```powershell
param(
    [ValidateSet('cpu', 'cuda')][string]$Device = 'cpu',
    [string]$Load,
    [string]$RunDirectory = 'artifacts',
    [switch]$Paused
)
```

**Execution and syntax:** `param` declares arguments; `$env:` accesses process environment; `&` invokes a command; `try/finally` restores the caller's environment. Native exit codes are checked, and `plain_command.py` captures UTF-8 output before PowerShell decoding.

</details>

<details><summary><code>tests/test_evolution.py</code></summary>

**Purpose:** Checks inherited neural control, sparse reference agreement, atomicity, topology migration, resource-funded anatomy, and observer/candidate contracts.

[Open the file](C:/Thought/Projects/World/tests/test_evolution.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
from copy import deepcopy
import json
import math
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import torch
from world._native import Habitat as NativeHabitat
from world.brains.evolving import NeuralPopulation, reference_update
from world.brains.candidates import LiquidCandidate
from world.control import RunControl
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import compact_preset, fund
from world.presets import founder_genome, neural_blueprint
```

- `torch` supplies tensor execution, sparse indexed sums, device transfers, and explicit recurrent state.
- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `math` supplies geometry/probability arithmetic and an independent scalar recurrence reference.
- `copy` creates independent configs/genomes or reuses immutable graph arrays while assigning new memory.
- `time` measures/paces wall-clock work; native model time remains tick-based.
- `tempfile` creates exclusive sibling save candidates or isolated test-owned files.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `unittest` and targeted mocks check independent outcomes and rollback, not only successful execution.

**Class: `EvolutionTests`**

Groups the related typed state used by the methods below.

[Source lines 72–74](C:/Thought/Projects/World/tests/test_evolution.py:72)

```python
class EvolutionTests(unittest.TestCase):
    def test_existing_and_new_resource_budgets_are_funded(self):
        rules, config = setup()
```

**Methods — open the one you are reading**

<details><summary><code>test_existing_and_new_resource_budgets_are_funded()</code></summary>

Checks existing and new resource budgets are funded.

[Source lines 73–79](C:/Thought/Projects/World/tests/test_evolution.py:73)

```python
    def test_existing_and_new_resource_budgets_are_funded(self):
        rules, config = setup()
        runtime = EcosystemRuntime.create(rules, config)
        audit = runtime.audit()
        self.assertAlmostEqual(audit["nutrient"], config["initial_nutrient"])
        self.assertAlmostEqual(audit["energy"], config["initial_energy"])
        self.assertTrue(runtime.frame()["agents"])
```

</details>

<details><summary><code>test_sparse_batch_matches_an_independent_scalar_reference()</code></summary>

Checks sparse batch matches an independent scalar reference.

[Source lines 83–89](C:/Thought/Projects/World/tests/test_evolution.py:83)

```python
    def test_sparse_batch_matches_an_independent_scalar_reference(self):
        runtime = EcosystemRuntime.create(*setup())
        for _ in range(3):
            frame = runtime.frame()
            _, proposed = runtime.brains.propose(frame)
            for agent in frame["agents"]:
                previous = {node: runtime.brains.memory[index].item() for (identity, node), index in runtime.brains.index.items() if identity == agent["id"]}
```

</details>

<details><summary><code>test_neural_parameters_are_inherited_and_newborn_memory_resets()</code></summary>

Checks neural parameters are inherited and newborn memory resets.

[Source lines 95–101](C:/Thought/Projects/World/tests/test_evolution.py:95)

```python
    def test_neural_parameters_are_inherited_and_newborn_memory_resets(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(22)
        state = runtime.snapshot()
        population = {o["id"]: o for o in state["world"]["ledger"]["organisms"]}
        children = [o for o in population.values() if o["parent_id"] is not None]
        self.assertTrue(children)
```

</details>

<details><summary><code>test_birth_mutation_records_variation_and_preserves_ancestors()</code></summary>

Checks birth mutation records variation and preserves ancestors.

[Source lines 110–116](C:/Thought/Projects/World/tests/test_evolution.py:110)

```python
    def test_birth_mutation_records_variation_and_preserves_ancestors(self):
        runtime = EcosystemRuntime.create(*setup(mutate=True))
        initial = {o["id"]: deepcopy(o["genome"]) for o in runtime.snapshot()["world"]["ledger"]["organisms"]}
        runtime.advance(25)
        state = runtime.snapshot()
        for o in state["world"]["ledger"]["organisms"]:
            if o["id"] in initial:
```

</details>

<details><summary><code>test_full_checkpoint_preserves_neural_memory_and_future_births()</code></summary>

Checks full checkpoint preserves neural memory and future births.

[Source lines 121–127](C:/Thought/Projects/World/tests/test_evolution.py:121)

```python
    def test_full_checkpoint_preserves_neural_memory_and_future_births(self):
        runtime = EcosystemRuntime.create(*setup(mutate=True))
        runtime.advance(17)
        resumed = EcosystemRuntime.from_snapshot(runtime.snapshot())
        self.assertEqual(runtime.snapshot(), resumed.snapshot())
        for count in [1, 5, 17]:
            runtime.advance(count)
```

</details>

<details><summary><code>test_native_and_cross_language_failures_preserve_world_and_memory()</code></summary>

Checks native and cross language failures preserve world and memory.

[Source lines 131–137](C:/Thought/Projects/World/tests/test_evolution.py:131)

```python
    def test_native_and_cross_language_failures_preserve_world_and_memory(self):
        runtime = EcosystemRuntime.create(*setup())
        before = runtime.snapshot()
        frame = runtime.frame()
        invalid = actions(frame)
        invalid["agents"][0]["version"] += 1
        with self.assertRaises(ValueError):
```

</details>

<details><summary><code>test_duplicate_unknown_nonfinite_and_inactive_actions_are_rejected()</code></summary>

Checks duplicate unknown nonfinite and inactive actions are rejected.

[Source lines 145–151](C:/Thought/Projects/World/tests/test_evolution.py:145)

```python
    def test_duplicate_unknown_nonfinite_and_inactive_actions_are_rejected(self):
        runtime = EcosystemRuntime.create(*setup())
        baseline = runtime.snapshot()
        frame = runtime.frame()
        variants = []
        value = actions(frame); value["agents"].append(deepcopy(value["agents"][0])); variants.append(value)
        value = actions(frame); value["agents"][0]["id"] = 999; variants.append(value)
```

</details>

<details><summary><code>test_malformed_coordinated_checkpoints_are_rejected()</code></summary>

Checks malformed coordinated checkpoints are rejected.

[Source lines 160–166](C:/Thought/Projects/World/tests/test_evolution.py:160)

```python
    def test_malformed_coordinated_checkpoints_are_rejected(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance()
        baseline = runtime.snapshot()
        changes = [lambda p: p.update(extra=1), lambda p: p.update(kind="wrong"),
                   lambda p: p["neural"].update(tick=0), lambda p: p["neural"]["memory"].append(0),
                   lambda p: p["neural"]["memory"].__setitem__(0, True),
```

</details>

<details><summary><code>test_topology_migration_preserves_survivors_and_initializes_only_new_nodes()</code></summary>

Checks topology migration preserves survivors and initializes only new nodes.

[Source lines 174–180](C:/Thought/Projects/World/tests/test_evolution.py:174)

```python
    def test_topology_migration_preserves_survivors_and_initializes_only_new_nodes(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        runtime.brains.memory.fill_(0.25)
        changed = deepcopy(frame)
        agent = changed["agents"][0]
        new_id = agent["brain"]["next_node"]
```

</details>

<details><summary><code>test_heterogeneous_and_thousand_neuron_graphs_execute_without_dense_padding()</code></summary>

Checks heterogeneous and thousand neuron graphs execute without dense padding.

[Source lines 193–199](C:/Thought/Projects/World/tests/test_evolution.py:193)

```python
    def test_heterogeneous_and_thousand_neuron_graphs_execute_without_dense_padding(self):
        rules, config = setup()
        config["evolution"]["mutation"]["max_nodes"] = 4096
        config["populations"][0].update(count=1, genome=founder_genome("grazer", hidden=1000))
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        population = runtime.brains
```

</details>

<details><summary><code>test_zero_neuron_population_runs_without_scripted_foraging()</code></summary>

Checks zero neuron population runs without scripted foraging.

[Source lines 207–213](C:/Thought/Projects/World/tests/test_evolution.py:207)

```python
    def test_zero_neuron_population_runs_without_scripted_foraging(self):
        rules, config = setup()
        for group in config["populations"]:
            group["genome"].pop("brain", None)
        runtime = EcosystemRuntime.create(rules, config)
        before = runtime.snapshot()["world"]["positions"]
        self.assertEqual(runtime.frame()["agents"], [])
```

</details>

<details><summary><code>test_contact_damage_is_local_and_repair_is_resource_funded()</code></summary>

Checks contact damage is local and repair is resource funded.

[Source lines 218–224](C:/Thought/Projects/World/tests/test_evolution.py:218)

```python
    def test_contact_damage_is_local_and_repair_is_resource_funded(self):
        world = contact_world()
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(json.dumps(actions(frame, {(2, 121): 1.0})))
        first = json.loads(world.checkpoint_json())
        target = first["ledger"]["organisms"][0]
        self.assertAlmostEqual(target["tissue"]["nutrient"], 7.9)
```

</details>

<details><summary><code>test_biting_routes_local_waste_and_preserves_resource_accounting()</code></summary>

Checks biting routes local waste and preserves resource accounting.

[Source lines 237–243](C:/Thought/Projects/World/tests/test_evolution.py:237)

```python
    def test_biting_routes_local_waste_and_preserves_resource_accounting(self):
        world = contact_world()
        frame = json.loads(world.agent_frame_json())
        world.advance_controlled(json.dumps(actions(frame, {(2, 120): 1.0})))
        payload = json.loads(world.checkpoint_json())
        self.assertAlmostEqual(payload["ledger"]["organisms"][0]["tissue"]["nutrient"], 7.8)
        self.assertAlmostEqual(payload["ledger"]["organisms"][1]["reserves"]["nutrient"], 0.65)
```

</details>

<details><summary><code>test_growth_preserves_absolute_injury_instead_of_repairing_for_free()</code></summary>

Checks growth preserves absolute injury instead of repairing for free.

[Source lines 249–255](C:/Thought/Projects/World/tests/test_evolution.py:249)

```python
    def test_growth_preserves_absolute_injury_instead_of_repairing_for_free(self):
        world = contact_world()
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(2, 121): 1.0})))
        before = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
        missing = sum(m["developed"] - m["nutrient"] for m in before["body"]["modules"])
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(1, 100): 1.0})))
        after = json.loads(world.checkpoint_json())["ledger"]["organisms"][0]
```

</details>

<details><summary><code>test_storage_capacity_is_coupled_to_funded_functional_tissue()</code></summary>

Checks storage capacity is coupled to funded functional tissue.

[Source lines 259–265](C:/Thought/Projects/World/tests/test_evolution.py:259)

```python
    def test_storage_capacity_is_coupled_to_funded_functional_tissue(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        target = payload["ledger"]["organisms"][0]
        module = {"id": 3, "parent": 1, "kind": "storage", "radius": 0.13, "angle": -math.pi / 2,
                  "orientation": 0.0, "investment": 1.0}
        target["genome"]["body"]["modules"].append(module)
```

</details>

<details><summary><code>test_unreachable_attacks_cannot_damage_remote_organisms()</code></summary>

Checks unreachable attacks cannot damage remote organisms.

[Source lines 283–289](C:/Thought/Projects/World/tests/test_evolution.py:283)

```python
    def test_unreachable_attacks_cannot_damage_remote_organisms(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        payload["positions"][1]["position"] = {"x": 3.0, "y": 6.0}
        world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
        world.advance_controlled(json.dumps(actions(json.loads(world.agent_frame_json()), {(2, 121): 1.0})))
        payload = json.loads(world.checkpoint_json())
```

</details>

<details><summary><code>test_diagnostic_counter_overflow_rejects_the_whole_window()</code></summary>

Checks diagnostic counter overflow rejects the whole window.

[Source lines 293–299](C:/Thought/Projects/World/tests/test_evolution.py:293)

```python
    def test_diagnostic_counter_overflow_rejects_the_whole_window(self):
        world = contact_world()
        payload = json.loads(world.checkpoint_json())
        payload["diagnostics"]["encounters"] = 2**64 - 1
        world = NativeHabitat.from_checkpoint_json(json.dumps(payload))
        before = world.checkpoint_json()
        with self.assertRaises(ValueError):
```

</details>

<details><summary><code>test_liquid_candidates_use_the_same_ports_and_preserve_inherited_parameters()</code></summary>

Checks liquid candidates use the same ports and preserve inherited parameters.

[Source lines 303–309](C:/Thought/Projects/World/tests/test_evolution.py:303)

```python
    def test_liquid_candidates_use_the_same_ports_and_preserve_inherited_parameters(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        agent = frame["agents"][0]
        for family in ["cfc", "ltc"]:
            with self.subTest(family=family):
                model = LiquidCandidate(family, len(agent["inputs"]), len(agent["outputs"]))
```

</details>

<details><summary><code>test_observer_controls_save_load_pause_and_failed_load_are_transactional()</code></summary>

Checks observer controls save load pause and failed load are transactional.

[Source lines 325–331](C:/Thought/Projects/World/tests/test_evolution.py:325)

```python
    def test_observer_controls_save_load_pause_and_failed_load_are_transactional(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory) / "checkpoint.json"
            control = RunControl(runtime, checkpoint, Path(directory) / "stream.json", paused=True)
            command_number = 0
            def command(kind, path=""):
```

</details>

<details><summary><code>test_sparse_cuda_proposal_agrees_with_cpu()</code></summary>

Checks sparse cuda proposal agrees with cpu.

[Source lines 389–395](C:/Thought/Projects/World/tests/test_evolution.py:389)

```python
    @unittest.skipUnless(torch.cuda.is_available(), "CUDA is unavailable")
    def test_sparse_cuda_proposal_agrees_with_cpu(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        cpu, _ = runtime.brains.propose(frame)
        gpu, _ = NeuralPopulation(frame, "cuda").propose(frame)
        for a, b in zip(cpu["agents"], gpu["agents"], strict=True):
```

</details>

**Standalone functions**

<details><summary><code>setup()</code></summary>

Supports setup within this component's responsibility; the source link opens the complete implementation.

[Source lines 24–30](C:/Thought/Projects/World/tests/test_evolution.py:24)

```python
def setup(seed=73, mutate=False):
    rules, config = compact_preset(RULES, seed)
    config["producers"]["count"] = 4
    config["populations"][0]["count"] = 2
    config["populations"][1]["count"] = 1
    config["ecology"]["max_population"] = 32
    if not mutate:
```

</details>

<details><summary><code>actions()</code></summary>

Supports actions within this component's responsibility; the source link opens the complete implementation.

[Source lines 36–41](C:/Thought/Projects/World/tests/test_evolution.py:36)

```python
def actions(frame, requests=None):
    requests = requests or {}
    return {"schema_version": 1, "tick": frame["tick"], "agents": [
        {"id": a["id"], "version": a["version"], "values": [
            {"port": p["port"], "value": requests.get((a["id"], p["channel"]), 0.0) if p["active"] else 0.0}
            for p in a["outputs"]]} for a in frame["agents"]]}
```

</details>

<details><summary><code>contact_world()</code></summary>

Supports contact world within this component's responsibility; the source link opens the complete implementation.

[Source lines 44–50](C:/Thought/Projects/World/tests/test_evolution.py:44)

```python
def contact_world():
    rules, config = setup()
    rules.update(world_width=14, world_height=12, grid_width=14, grid_height=12)
    producer = founder_genome("producer")
    producer["brain"] = neural_blueprint(producer["body"], hidden=0, kind="producer")
    consumer = founder_genome("grazer")
    consumer["body"]["modules"] = consumer["body"]["modules"][:2]
```

</details>

<details><summary><code>scale()</code></summary>

Supports scale within this component's responsibility; the source link opens the complete implementation.

[Source lines 63–64](C:/Thought/Projects/World/tests/test_evolution.py:63)

```python
    def scale(genome, tissue):
        return math.sqrt(tissue / sum(math.pi * m["radius"]**2 * 12 for m in genome["body"]["modules"]))
```

</details>

<details><summary><code>command()</code></summary>

Supports command within this component's responsibility; the source link opens the complete implementation.

[Source lines 331–334](C:/Thought/Projects/World/tests/test_evolution.py:331)

```python
            def command(kind, path=""):
                nonlocal command_number
                command_number += 1
                return {"schema_version": 1, "session": control.session, "id": f"{kind}-{command_number}", "kind": kind, "path": path}
```

</details>

**Useful syntax in this file**


<details><summary><code>test_realtime_pacing_respects_speed_pause_and_single_decisions()</code></summary>

Checks fixed decision deadlines at 1x and 16x, pause/resume, and exactly one explicit advance using a controlled clock.

[Source lines 348–354](C:/Thought/Projects/World/tests/test_evolution.py:348)

```python
    def test_realtime_pacing_respects_speed_pause_and_single_decisions(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            control = RunControl(runtime, Path(directory) / "checkpoint.json")
            def command(kind, identity):
                return {"schema_version": 1, "session": control.session, "id": identity, "kind": kind, "path": ""}
            self.assertTrue(control.update(now=0.0))
```

</details>

<details><summary><code>test_published_speed_reports_elapsed_simulation_time()</code></summary>

Checks that achieved speed is calculated from elapsed world time and wall time, independently of a requested 16x setting, without changing the published world.

[Source lines 374–380](C:/Thought/Projects/World/tests/test_evolution.py:374)

```python
    def test_published_speed_reports_elapsed_simulation_time(self):
        runtime = EcosystemRuntime.create(*setup())
        with tempfile.TemporaryDirectory() as directory:
            stream = Path(directory) / "stream.json"
            control = RunControl(runtime, Path(directory) / "checkpoint.json", stream)
            control.speed = 16
            control._rate_wall, control._rate_time = 0.0, 0.0
```

</details>

</details>

## Step 9: Birth-time variation and controlled pilots

Mutation operates at funded births, preserving parents and unaffected identities. Clone/brain/body/combined pilots and held-out ancestry-matched copies report mixed outcomes without asserting intelligence gains.

<details><summary><code>crates/world_genetics/src/mutation.rs</code></summary>

**Purpose:** Applies bounded, seeded birth-time scalar/body/topology proposals while retaining stable identities and recording invalid attempts.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs)

**Imports / uses and why**

```rust
use std::collections::{BTreeMap, BTreeSet};
use crate::{
    body::{BodyGenome, ModuleGene, ModuleKind},
    genome::Genome,
    neural::{ConnectionGene, NodeGene, NodeKind},
};
use serde::{Deserialize, Serialize};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.

**Struct: `MutationConfig`**

Declares mutation distributions and structural ceilings. The genetic operators use this record to generate bounded, validated offspring variation.

[Source lines 12–17](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:12)

```rust
pub struct MutationConfig {
    pub parameter_edits: f64,
    pub body_edits: f64,
    pub sigma: f64,
    pub body_structure_rate: f64,
    pub neural_structure_rate: f64,
```

**Struct: `Edit`**

Handles Edit at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 67–72](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:67)

```rust
pub struct Edit {
    pub kind: String,
    pub gene: u64,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub accepted: bool,
```

**Struct: `BirthRandom`**

Handles BirthRandom at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 76–81](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:76)

```rust
pub struct BirthRandom(u64);
impl BirthRandom {
    pub fn new(seed: u64, parent: u64, child: u64, tick: u64) -> Self {
        let mut rng =
            Self(seed ^ parent.rotate_left(17) ^ child.rotate_left(39) ^ tick.rotate_left(7));
        rng.bits();
```

**Methods in `impl Default for MutationConfig`**

<details><summary><code>default()</code></summary>

Supports default within this component's responsibility; the source link opens the complete implementation.

[Source lines 25–30](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:25)

```rust
    fn default() -> Self {
        Self {
            parameter_edits: 1.0,
            body_edits: 1.0,
            sigma: 0.15,
            body_structure_rate: 0.01,
```

</details>

**Methods in `impl MutationConfig`**

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 41–46](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:41)

```rust
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.parameter_edits.is_finite()
            || !(0.0..=16.0).contains(&self.parameter_edits)
            || !self.body_edits.is_finite()
            || !(0.0..=16.0).contains(&self.body_edits)
            || !self.sigma.is_finite()
```

</details>

**Methods in `impl BirthRandom`**

<details><summary><code>new()</code></summary>

Supports new within this component's responsibility; the source link opens the complete implementation.

[Source lines 78–83](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:78)

```rust
    pub fn new(seed: u64, parent: u64, child: u64, tick: u64) -> Self {
        let mut rng =
            Self(seed ^ parent.rotate_left(17) ^ child.rotate_left(39) ^ tick.rotate_left(7));
        rng.bits();
        rng
    }
```

</details>

<details><summary><code>bits()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 84–89](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:84)

```rust
    pub fn bits(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
```

</details>

<details><summary><code>unit()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 91–96](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:91)

```rust
    pub fn unit(&mut self) -> f64 {
        (self.bits() >> 11) as f64 / 9007199254740992.0
    }
    pub fn index(&mut self, count: usize) -> usize {
        (self.bits() % count as u64) as usize
    }
```

</details>

<details><summary><code>index()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 94–99](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:94)

```rust
    pub fn index(&mut self, count: usize) -> usize {
        (self.bits() % count as u64) as usize
    }
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(f64::MIN_POSITIVE).ln()).sqrt()
            * (std::f64::consts::TAU * self.unit()).cos()
```

</details>

<details><summary><code>normal()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 97–102](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:97)

```rust
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(f64::MIN_POSITIVE).ln()).sqrt()
            * (std::f64::consts::TAU * self.unit()).cos()
    }
    fn poisson(&mut self, mean: f64) -> usize {
        if mean == 0.0 {
```

</details>

<details><summary><code>poisson()</code></summary>

Draws the declared deterministic birth/founder random quantity; this does not evaluate a hidden fitness score.

[Source lines 101–106](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:101)

```rust
    fn poisson(&mut self, mean: f64) -> usize {
        if mean == 0.0 {
            return 0;
        }
        let limit = (-mean).exp();
        let mut product = 1.0;
```

</details>

**Functions outside impl blocks**

<details><summary><code>mutate()</code></summary>

Copies the inherited genome, applies separately budgeted valid proposals, preserves unaffected genes, records rejections, and versions changed encoding.

[Source lines 117–122](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:117)

```rust
pub fn mutate(
    parent: &Genome,
    config: &MutationConfig,
    seed: u64,
    parent_id: u64,
    child_id: u64,
```

</details>

<details><summary><code>body_structure()</code></summary>

Proposes one module addition/subtree deletion/duplication/reattachment; validation decides whether that single proposal is accepted.

[Source lines 383–388](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:383)

```rust
fn body_structure(
    body: &mut BodyGenome,
    rng: &mut BirthRandom,
) -> Result<(String, u64), &'static str> {
    let index = rng.index(body.modules.len());
    let chosen = body.modules[index].clone();
```

</details>

<details><summary><code>subtree()</code></summary>

Supports subtree within this component's responsibility; the source link opens the complete implementation.

[Source lines 465–470](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:465)

```rust
fn subtree(body: &BodyGenome, root: u64) -> BTreeSet<u64> {
    let mut ids = BTreeSet::from([root]);
    loop {
        let n = ids.len();
        for m in &body.modules {
            if m.parent.is_some_and(|p| ids.contains(&p)) {
```

</details>

<details><summary><code>neural_structure()</code></summary>

Proposes a hidden-neuron insertion/deletion/duplication or edge addition/deletion with stable new identities.

[Source lines 481–486](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:481)

```rust
fn neural_structure(
    brain: &mut crate::neural::NeuralGenome,
    rng: &mut BirthRandom,
) -> Result<(String, u64), &'static str> {
    match rng.index(5) {
        0 => {
```

</details>

<details><summary><code>add_edge()</code></summary>

Supports add edge within this component's responsibility; the source link opens the complete implementation.

[Source lines 592–597](C:/Thought/Projects/World/crates/world_genetics/src/mutation.rs:592)

```rust
fn add_edge(
    brain: &mut crate::neural::NeuralGenome,
    source: u64,
    target: u64,
    weight: f64,
) -> Result<(), &'static str> {
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_genetics/tests/evolution_contract.rs</code></summary>

**Purpose:** Checks immutable clone inheritance, deterministic structural proposals, stable unaffected parameters, and malformed gene rejection.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/tests/evolution_contract.rs)

**Imports / uses and why**

```rust
use world_genetics::{
    body::{BodyGenome, Diet, ModuleGene, ModuleKind},
    genome::Genome,
    mutation::{MutationConfig, mutate},
    neural::{ConnectionGene, NeuralGenome, NodeGene, NodeKind},
};
```

- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Functions outside impl blocks**

<details><summary><code>genome()</code></summary>

Supports genome within this component's responsibility; the source link opens the complete implementation.

[Source lines 8–13](C:/Thought/Projects/World/crates/world_genetics/tests/evolution_contract.rs:8)

```rust
fn genome() -> Genome {
    let mut genome = Genome::new(false, true, vec![0.25, -0.5]).unwrap();
    let body = BodyGenome {
        circle: None,
        modules: vec![
            ModuleGene {
```

</details>

<details><summary><code>disabled_mutation_is_exact_inheritance_and_never_changes_the_parent()</code></summary>

Supports disabled mutation is exact inheritance and never changes the parent within this component's responsibility; the source link opens the complete implementation.

[Source lines 102–107](C:/Thought/Projects/World/crates/world_genetics/tests/evolution_contract.rs:102)

```rust
fn disabled_mutation_is_exact_inheritance_and_never_changes_the_parent() {
    let parent = genome();
    let before = parent.clone();
    let config = MutationConfig {
        parameter_edits: 0.0,
        body_edits: 0.0,
```

</details>

<details><summary><code>structural_proposals_preserve_validity_identity_and_unedited_parameters()</code></summary>

Supports structural proposals preserve validity identity and unedited parameters within this component's responsibility; the source link opens the complete implementation.

[Source lines 119–124](C:/Thought/Projects/World/crates/world_genetics/tests/evolution_contract.rs:119)

```rust
fn structural_proposals_preserve_validity_identity_and_unedited_parameters() {
    let parent = genome();
    let config = MutationConfig {
        parameter_edits: 0.0,
        body_edits: 0.0,
        body_structure_rate: 1.0,
```

</details>

<details><summary><code>malformed_bodies_and_neural_references_are_rejected()</code></summary>

Supports malformed bodies and neural references are rejected within this component's responsibility; the source link opens the complete implementation.

[Source lines 179–184](C:/Thought/Projects/World/crates/world_genetics/tests/evolution_contract.rs:179)

```rust
fn malformed_bodies_and_neural_references_are_rejected() {
    let original = genome();
    let mut bad = original.clone();
    bad.body.as_mut().unwrap().modules[1].parent = Some(2);
    assert!(bad.validate().is_err());
    let mut bad = original.clone();
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>examples/adaptation_demo.py</code></summary>

**Purpose:** Provides the CLI for seeded mutation controls and held-out ancestral/descendant pilot comparisons.

[Open the file](C:/Thought/Projects/World/examples/adaptation_demo.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import argparse
import json
from pathlib import Path
from world.ecosystem import write_json
from world.experiments.adaptation import run_controls
```

- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `argparse` validates user-facing run/device/seed/output options.

**Standalone functions**

<details><summary><code>main()</code></summary>

Validates CLI inputs, runs this file's documented workflow, and writes its declared outputs/errors.

[Source lines 9–15](C:/Thought/Projects/World/examples/adaptation_demo.py:9)

```python
def main():
    parser = argparse.ArgumentParser(description="Run mutation-disabled, brain-only, body-only, and combined pilot controls.")
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--decisions", type=int, default=120)
    parser.add_argument("--evaluation-decisions", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("artifacts/adaptation-controls.json"))
    args = parser.parse_args()
```

</details>

**Useful syntax in this file**


</details>

<details><summary><code>python/world/experiments/adaptation.py</code></summary>

**Purpose:** Runs declared clone/brain/body/combined controls, archives ancestry-matched genomes, and compares copies in held-out common environments.

[Open the file](C:/Thought/Projects/World/python/world/experiments/adaptation.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
from copy import deepcopy
import hashlib
import json
from ..ecosystem import EcosystemRuntime
from ..presets import evolution_preset, founder_genome
```

- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `copy` creates independent configs/genomes or reuses immutable graph arrays while assigning new memory.
- `time` measures/paces wall-clock work; native model time remains tick-based.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `hashlib` fingerprints archived encoded genomes for comparison and provenance.

**Standalone functions**

<details><summary><code>compact_preset()</code></summary>

Supports compact preset within this component's responsibility; the source link opens the complete implementation.

[Source lines 9–15](C:/Thought/Projects/World/python/world/experiments/adaptation.py:9)

```python
def compact_preset(rules, seed=73):
    rules, config = evolution_preset(rules, seed)
    rules.update(world_width=20.0, world_height=20.0, grid_width=20, grid_height=20)
    config["producers"]["count"] = 12
    config["populations"] = config["populations"][:2]
    config["populations"][0]["count"] = 4
    config["populations"][1]["count"] = 2
```

</details>

<details><summary><code>fund()</code></summary>

Recomputes the declared starting material/energy from actual cell and founder inventories after a controlled preset edit.

[Source lines 21–25](C:/Thought/Projects/World/python/world/experiments/adaptation.py:21)

```python
def fund(rules, config):
    groups = [config["producers"]] + config.get("populations", [])
    config["initial_nutrient"] = (config["mineral_nutrient_per_cell"] * rules["grid_width"] * rules["grid_height"] +
                                  sum(g["count"] * (g["tissue_nutrient"] + g["reserve_nutrient"]) for g in groups))
    config["initial_energy"] = sum(g["count"] * (g["tissue_nutrient"] * rules["tissue_energy_per_nutrient"] + g["reserve_energy"]) for g in groups)
```

</details>

<details><summary><code>metrics()</code></summary>

Supports metrics within this component's responsibility; the source link opens the complete implementation.

[Source lines 28–34](C:/Thought/Projects/World/python/world/experiments/adaptation.py:28)

```python
def metrics(runtime):
    state = json.loads(runtime.world.snapshot_json())
    living = [r["organism"] for r in state["organisms"] if r["organism"]["alive"]]
    active = [o for o in living if o["genome"].get("brain")]
    summary = runtime.summary()
    return {"tick": runtime.tick, "living": len(living), "active_living": len(active),
            "births": summary["ecology"]["births"], "deaths": summary["ecology"]["deaths"],
```

</details>

<details><summary><code>archive_pair()</code></summary>

Selects a born neural candidate by a declared observational rule and follows its ancestry to archive an immutable founder/descendant pair.

[Source lines 45–51](C:/Thought/Projects/World/python/world/experiments/adaptation.py:45)

```python
def archive_pair(runtime):
    organisms = {o["id"]: o for o in runtime.snapshot()["world"]["ledger"]["organisms"]}
    descendants = {identity: 0 for identity in organisms}
    for organism in organisms.values():
        parent = organism["parent_id"]
        while parent is not None:
            descendants[parent] += 1
```

</details>

<details><summary><code>item()</code></summary>

Supports item within this component's responsibility; the source link opens the complete implementation.

[Source lines 60–63](C:/Thought/Projects/World/python/world/experiments/adaptation.py:60)

```python
    def item(organism):
        encoded = json.dumps(organism["genome"], sort_keys=True, separators=(",", ":"))
        return {"organism_id": organism["id"], "genome": deepcopy(organism["genome"]),
                "sha256": hashlib.sha256(encoded.encode()).hexdigest()}
```

</details>

<details><summary><code>common_environment()</code></summary>

Evaluates a birth-state genome copy with matched starting resources/placement and held-out environmental seeds.

[Source lines 68–74](C:/Thought/Projects/World/python/world/experiments/adaptation.py:68)

```python
def common_environment(rules, genome, seed, decisions=100):
    rules, config = compact_preset(rules, seed)
    config["ecology"]["reproduction"] = False
    config["evolution"]["mutation"].update(parameter_edits=0.0, body_edits=0.0, body_structure_rate=0.0, neural_structure_rate=0.0)
    food = deepcopy(config["producers"])
    food["count"] = 8
    food["positions"] = [{"x": x, "y": y} for x, y in [(3, 3), (10, 3), (17, 3), (3, 10), (17, 10), (3, 17), (10, 17), (17, 17)]]
```

</details>

<details><summary><code>run_controls()</code></summary>

Runs all declared seeds under clone, brain-only, body-only, and combined scalar variation, retaining failed or unimproved outcomes.

[Source lines 92–98](C:/Thought/Projects/World/python/world/experiments/adaptation.py:92)

```python
def run_controls(rules, seeds=(73, 89, 107), decisions=120, evaluation_decisions=100):
    rows = []
    for seed in seeds:
        for condition in ["clone", "brain_only", "body_only", "combined"]:
            world_rules, config = compact_preset(rules, seed)
            config["evolution"]["mutation"].update(parameter_edits=0.0 if condition in {"clone", "body_only"} else 1.0,
                                                   body_edits=0.0 if condition in {"clone", "brain_only"} else 1.0,
```

</details>

**Useful syntax in this file**


</details>

## Step 10: Funded anatomy, contact injury, and repair

Module trees develop from real tissue. Compound geometry, mass/rotational resistance, work costs, local attacks/bites, impairment, storage, growth, and repair share one conservative inventory.

<details><summary><code>crates/world_core/src/anatomy.rs</code></summary>

**Purpose:** Develops funded module tissue into compound-circle geometry, tracks real injury, and couples module integrity to function and storage.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs)

**Imports / uses and why**

```rust
use crate::{
    environment::Position,
    resources::{AccountingError, Result, finite},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use world_genetics::body::{BodyGenome, ModuleKind};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `ModuleState`**

Handles ModuleState at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:11)

```rust
pub struct ModuleState {
    pub id: u64,
    pub nutrient: f64,
    pub developed: f64,
}

```

**Struct: `BodyState`**

Handles BodyState at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 19–24](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:19)

```rust
pub struct BodyState {
    pub modules: Vec<ModuleState>,
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
```

**Struct: `Circle`**

Handles Circle at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 24–29](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:24)

```rust
pub struct Circle {
    pub id: u64,
    pub kind: ModuleKind,
    pub center: Position,
    pub radius: f64,
    pub nutrient: f64,
```

**Methods in `impl BodyState`**

<details><summary><code>new()</code></summary>

Supports new within this component's responsibility; the source link opens the complete implementation.

[Source lines 60–65](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:60)

```rust
    pub fn new(genome: &BodyGenome, nutrient: f64) -> Result<Self> {
        finite(nutrient, "developed body tissue", false)?;
        let total = genome.target_nutrient();
        finite(total, "body target tissue", true)?;
        let mut remaining = nutrient;
        let mut modules = Vec::new();
```

</details>

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 84–89](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:84)

```rust
    pub fn validate(&self, genome: &BodyGenome, nutrient: f64) -> Result<()> {
        if self.modules.len() != genome.modules.len() {
            return Err(AccountingError::Invariant(
                "body module inventory mismatch".into(),
            ));
        }
```

</details>

<details><summary><code>total()</code></summary>

Supports total within this component's responsibility; the source link opens the complete implementation.

[Source lines 122–127](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:122)

```rust
    pub fn total(&self) -> f64 {
        self.modules.iter().map(|m| m.nutrient).sum()
    }
    pub fn developed(&self) -> f64 {
        self.modules.iter().map(|m| m.developed).sum()
    }
```

</details>

<details><summary><code>developed()</code></summary>

Supports developed within this component's responsibility; the source link opens the complete implementation.

[Source lines 125–130](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:125)

```rust
    pub fn developed(&self) -> f64 {
        self.modules.iter().map(|m| m.developed).sum()
    }
    pub fn missing(&self) -> f64 {
        self.modules
            .iter()
```

</details>

<details><summary><code>missing()</code></summary>

Supports missing within this component's responsibility; the source link opens the complete implementation.

[Source lines 128–133](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:128)

```rust
    pub fn missing(&self) -> f64 {
        self.modules
            .iter()
            .map(|m| (m.developed - m.nutrient).max(0.0))
            .sum()
    }
```

</details>

<details><summary><code>integrity()</code></summary>

Supports integrity within this component's responsibility; the source link opens the complete implementation.

[Source lines 134–139](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:134)

```rust
    pub fn integrity(&self, id: u64) -> f64 {
        self.modules.iter().find(|m| m.id == id).map_or(0.0, |m| {
            if m.developed > 0.0 {
                (m.nutrient / m.developed).clamp(0.0, 1.0)
            } else {
                0.0
```

</details>

<details><summary><code>functional()</code></summary>

Combines funded local tissue integrity with the integrity of its attachment ancestors to determine available capability.

[Source lines 144–149](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:144)

```rust
    pub fn functional(&self, genome: &BodyGenome, id: u64) -> f64 {
        let mut current = Some(id);
        let mut value = 1.0_f64;
        while let Some(id) = current {
            let Some(gene) = genome.modules.iter().find(|g| g.id == id) else {
                return 0.0;
```

</details>

<details><summary><code>grow()</code></summary>

Supports grow within this component's responsibility; the source link opens the complete implementation.

[Source lines 163–168](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:163)

```rust
    pub fn grow(&mut self, genome: &BodyGenome, amount: f64) -> Result<()> {
        let addition = Self::new(genome, amount)?;
        for m in &mut self.modules {
            let share = addition
                .modules
                .iter()
```

</details>

<details><summary><code>remove()</code></summary>

Removes a bounded local or proportional module allocation without creating or duplicating structural resources.

[Source lines 178–183](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:178)

```rust
    pub fn remove(&mut self, amount: f64, module: Option<u64>) -> Result<f64> {
        finite(amount, "removed module tissue", false)?;
        let mut remaining = amount.min(self.total());
        let initial = remaining;
        if let Some(id) = module {
            let m = self
```

</details>

<details><summary><code>repair()</code></summary>

Restores at most missing developed tissue; the caller must first fund the added material and energy.

[Source lines 209–214](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:209)

```rust
    pub fn repair(&mut self, amount: f64) -> f64 {
        let mut remaining = amount;
        for m in &mut self.modules {
            let restored = remaining.min((m.developed - m.nutrient).max(0.0));
            m.nutrient += restored;
            remaining -= restored;
```

</details>

<details><summary><code>circles()</code></summary>

Supports circles within this component's responsibility; the source link opens the complete implementation.

[Source lines 219–224](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:219)

```rust
    pub fn circles(
        &self,
        genome: &BodyGenome,
        position: Position,
        heading: f64,
    ) -> Result<Vec<Circle>> {
```

</details>

<details><summary><code>inertia()</code></summary>

Computes geometry-derived rotational resistance from current module tissue and offsets.

[Source lines 268–273](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:268)

```rust
    pub fn inertia(&self, genome: &BodyGenome) -> Result<f64> {
        Ok(self
            .circles(genome, Position { x: 0.0, y: 0.0 }, 0.0)?
            .iter()
            .map(|c| {
                c.nutrient
```

</details>

**Functions outside impl blocks**

<details><summary><code>reserve_capacity()</code></summary>

Derives reserve limits from funded tissue and functional invested storage, with overflow handled by the existing conservative transfer rules.

[Source lines 33–38](C:/Thought/Projects/World/crates/world_core/src/anatomy.rs:33)

```rust
pub fn reserve_capacity(
    nutrient: f64,
    genome: Option<&BodyGenome>,
    body: Option<&BodyState>,
    rules: &crate::rules::WorldRules,
) -> Result<crate::resources::Resources> {
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/evolution.rs</code></summary>

**Purpose:** Coordinates neural-driven biology: acquisition, paid body movement, contacts, injury, funded development/repair, and mutated local births.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/evolution.rs)

**Imports / uses and why**

```rust
use crate::{
    agents::{self, Controls},
    anatomy::BodyState,
    diagnostics::{Diagnostics, EvolutionEvent},
    ecology::{EcologyState, initial_heading},
    environment::{EnvironmentGrid, Position},
    habitat::WindowReport,
    habitat_config::{HabitatConfig, seeded_position},
    ids::OrganismId,
    lifecycle,
    organism::{DeathCause, Organism},
    physiology,
    resources::{AccountingError, Resources, Result, finite},
    spatial::{distance, segment_distance},
    transactions::Draft,
};
use std::collections::BTreeMap;
use world_genetics::{body::ModuleKind, genome::Genome, mutation::mutate};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_genetics`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Struct: `PopulationWindow`**

Borrows one transaction draft's environment, organism positions, controls and diagnostics to execute a biological decision atomically.

[Source lines 20–25](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:20)

```rust
pub(crate) struct PopulationWindow<'a> {
    pub grid: &'a mut EnvironmentGrid,
    pub positions: &'a mut BTreeMap<OrganismId, Position>,
    pub ecology: &'a mut EcologyState,
    pub diagnostics: &'a mut Diagnostics,
    pub config: &'a HabitatConfig,
```

**Methods in `impl PopulationWindow<'_>`**

<details><summary><code>run()</code></summary>

Supports run within this component's responsibility; the source link opens the complete implementation.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:32)

```rust
    pub fn run(&mut self, draft: &mut Draft<'_>) -> Result<WindowReport> {
        let mut report = WindowReport {
            tick: self.tick,
            duration: self.duration,
            ..WindowReport::default()
        };
```

</details>

<details><summary><code>action()</code></summary>

Supports action within this component's responsibility; the source link opens the complete implementation.

[Source lines 175–180](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:175)

```rust
    pub(crate) fn action(&self, o: &Organism, module: u64, channel: u8) -> f64 {
        if o.genome.brain.is_none() {
            return if matches!(channel, 100 | 120) {
                1.0
            } else {
                0.0
```

</details>

<details><summary><code>move_one()</code></summary>

Converts expressed actuator effort into force/torque, checks swept compound geometry, and pays effort plus actual dissipated work.

[Source lines 191–196](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:191)

```rust
    fn move_one(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
```

</details>

<details><summary><code>interact()</code></summary>

Resolves paid local attacks and compatible bites against remaining module tissue, routing debris, carcasses, and digestive waste once.

[Source lines 312–317](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:312)

```rust
    fn interact(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
```

</details>

<details><summary><code>contact()</code></summary>

Supports contact within this component's responsibility; the source link opens the complete implementation.

[Source lines 482–487](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:482)

```rust
    pub(crate) fn contact(
        &self,
        draft: &Draft<'_>,
        id: OrganismId,
        center: Position,
        reach: f64,
```

</details>

<details><summary><code>develop()</code></summary>

Attempts separately funded repair and growth only when the resulting compound geometry remains valid and clear.

[Source lines 526–531](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:526)

```rust
    fn develop(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
```

</details>

<details><summary><code>births()</code></summary>

Checks mature parents, proposes inherited mutations, funds complexity-dependent juvenile tissue, and commits a clear local child without replacement grants.

[Source lines 700–705](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:700)

```rust
    fn births(&mut self, draft: &mut Draft<'_>, report: &mut WindowReport) -> Result<()> {
        if !self.config.ecology.reproduction {
            return Ok(());
        }
        let mut parents: Vec<_> = draft
            .organisms()
```

</details>

<details><summary><code>event()</code></summary>

Supports event within this component's responsibility; the source link opens the complete implementation.

[Source lines 889–894](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:889)

```rust
    pub(crate) fn event(
        &mut self,
        kind: &str,
        actor: OrganismId,
        affected: (Option<OrganismId>, Option<u64>),
        amount: f64,
```

</details>

<details><summary><code>death()</code></summary>

Supports death within this component's responsibility; the source link opens the complete implementation.

[Source lines 921–926](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:921)

```rust
    pub(crate) fn death(
        &mut self,
        draft: &Draft<'_>,
        id: OrganismId,
        source: Option<OrganismId>,
    ) -> Result<()> {
```

</details>

**Functions outside impl blocks**

<details><summary><code>capability()</code></summary>

Supports capability within this component's responsibility; the source link opens the complete implementation.

[Source lines 959–964](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:959)

```rust
pub fn capability(o: &Organism, kind: ModuleKind) -> f64 {
    if let Some(circle) = o.genome.body.as_ref().and_then(|b| b.circle.as_ref()) {
        return match kind {
            ModuleKind::Photosynthetic if circle.photosynthetic => o.tissue.nutrient,
            _ => 0.0,
        };
```

</details>

<details><summary><code>body_upkeep()</code></summary>

Supports body upkeep within this component's responsibility; the source link opens the complete implementation.

[Source lines 986–991](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:986)

```rust
fn body_upkeep(o: &Organism, rate: f64) -> f64 {
    o.genome
        .body
        .as_ref()
        .zip(o.body.as_ref())
        .map_or(0.0, |(g, b)| {
```

</details>

<details><summary><code>expend()</code></summary>

Supports expend within this component's responsibility; the source link opens the complete implementation.

[Source lines 1006–1011](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1006)

```rust
pub(crate) fn expend(draft: &mut Draft<'_>, id: OrganismId, energy: f64) -> Result<()> {
    if energy == 0.0 {
        return Ok(());
    }
    let mut o = draft.living(id)?;
    o.reserves = o.reserves.minus(Resources::new(0.0, energy)?)?;
```

</details>

<details><summary><code>reserve_floor()</code></summary>

Calculates the energy that must remain for the declared maintenance horizon, including body, sensing, and brain costs.

[Source lines 1017–1022](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1017)

```rust
pub(crate) fn reserve_floor(
    o: &Organism,
    rules: &crate::rules::WorldRules,
    config: &HabitatConfig,
) -> f64 {
    config.evolution.as_ref().map_or(0.0, |e| {
```

</details>

<details><summary><code>damage_module()</code></summary>

Removes tissue from the contacted module, deposits its resources locally, and applies vital-tissue/death rules.

[Source lines 1031–1036](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1031)

```rust
fn damage_module(draft: &mut Draft<'_>, id: OrganismId, module: u64, amount: f64) -> Result<()> {
    let mut o = draft.living(id)?;
    let removed = o
        .body
        .as_mut()
        .ok_or_else(|| AccountingError::Invariant("damage body missing".into()))?
```

</details>

<details><summary><code>offspring_tissue()</code></summary>

Adds declared module/neuron/connection material investment to the base juvenile tissue requirement.

[Source lines 1044–1049](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1044)

```rust
fn offspring_tissue(
    base: f64,
    g: &Genome,
    config: &crate::evolution_config::EvolutionConfig,
) -> f64 {
    base + g.body.as_ref().map_or(0.0, |b| {
```

</details>

<details><summary><code>clear_at()</code></summary>

Tests a candidate compound body's wall bounds and overlap with every relevant living body.

[Source lines 1057–1062](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1057)

```rust
pub(crate) fn clear_at(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    o: &Organism,
```

</details>

<details><summary><code>clear_growth()</code></summary>

Checks an expanded/repaired organism at its current pose before spending construction resources.

[Source lines 1095–1100](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1095)

```rust
fn clear_growth(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    o: &Organism,
```

</details>

<details><summary><code>clear_motion()</code></summary>

Checks swept module paths and conservative rotation arcs so a fast compound body cannot tunnel through another living body.

[Source lines 1113–1118](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1113)

```rust
fn clear_motion(
    draft: &Draft<'_>,
    grid: &EnvironmentGrid,
    positions: &BTreeMap<OrganismId, Position>,
    headings: &BTreeMap<OrganismId, f64>,
    id: OrganismId,
```

</details>

<details><summary><code>founder_position()</code></summary>

Chooses a deterministic in-bounds non-overlapping founder pose, rejecting invalid explicit placement or exhausted attempts.

[Source lines 1153–1158](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1153)

```rust
pub(crate) fn founder_position(
    grid: &EnvironmentGrid,
    organisms: &[Organism],
    positions: &BTreeMap<OrganismId, Position>,
    o: &Organism,
    seed: u64,
```

</details>

<details><summary><code>audit_geometry()</code></summary>

Uses spatial broad-phase queries and exact module distances to reject invalid saved/current anatomy bounds or overlap.

[Source lines 1195–1200](C:/Thought/Projects/World/crates/world_core/src/evolution.rs:1195)

```rust
pub(crate) fn audit_geometry(world: &crate::habitat::Habitat) -> Result<()> {
    let (width, height, _, _) = world.grid().dimensions();
    let mut shapes = BTreeMap::new();
    let mut extents = BTreeMap::new();
    let mut largest = 0.0_f64;
    for (id, o) in world.organisms().iter().filter(|(_, o)| o.alive) {
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_genetics/src/body.rs</code></summary>

**Purpose:** Defines inherited body trees, functional module kinds, diet compatibility, and stable physical/neural ports.

[Open the file](C:/Thought/Projects/World/crates/world_genetics/src/body.rs)

**Imports / uses and why**

```rust
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
```

- `serde`: Serde converts typed Rust records to and from validated JSON.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.

**Enum: `ModuleKind`**

Handles ModuleKind at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 7–12](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:7)

```rust
pub enum ModuleKind {
    Core,
    Propulsion,
    Sensor,
    Mouth,
    Armor,
```

**Struct: `ModuleGene`**

Handles ModuleGene at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 19–24](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:19)

```rust
pub struct ModuleGene {
    pub id: u64,
    pub parent: Option<u64>,
    pub kind: ModuleKind,
    pub radius: f64,
    pub angle: f64,
```

**Enum: `Diet`**

Handles Diet at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:32)

```rust
pub enum Diet {
    Grazer,
    Predator,
    Omnivore,
}

```

**Struct: `BodyGenome`**

Handles BodyGenome at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 40–45](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:40)

```rust
pub struct BodyGenome {
    pub modules: Vec<ModuleGene>,
    pub next_id: u64,
    pub diet: Diet,
    pub density: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
```

**Struct: `CircleTraits`**

Stores the single-core body's evolvable feeding, photosynthesis, speed, sensing, and learning-rate traits.

[Source lines 51–56](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:51)

```rust
pub struct CircleTraits {
    pub photosynthetic: bool,
    pub can_ingest: bool,
    pub speed: f64,
    pub perception: f64,
    pub learning_rate: f64,
```

**Struct: `Port`**

Handles Port at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 61–66](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:61)

```rust
pub struct Port {
    pub id: u64,
    pub module: u64,
    pub channel: u8,
    pub output: bool,
}
```

**Methods in `impl BodyGenome`**

<details><summary><code>validate()</code></summary>

Rejects inconsistent identities, unsupported parameters, non-finite values, and violated contracts before state is accepted.

[Source lines 69–74](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:69)

```rust
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(circle) = &self.circle
            && (self.modules.len() != 1
                || !circle.speed.is_finite()
                || !(0.1..=3.0).contains(&circle.speed)
                || !circle.perception.is_finite()
```

</details>

<details><summary><code>layout()</code></summary>

Decodes the connected body/camera layout; physical body decoding preserves attachments, while viewer layout only changes observation geometry.

[Source lines 123–128](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:123)

```rust
    pub fn layout(&self) -> Result<BTreeMap<u64, (f64, f64, f64)>, &'static str> {
        let mut positions = BTreeMap::new();
        for _ in 0..self.modules.len() {
            for m in &self.modules {
                if positions.contains_key(&m.id) {
                    continue;
```

</details>

<details><summary><code>ports()</code></summary>

Derives stable input/output IDs from module identity and declared channel meaning.

[Source lines 167–172](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:167)

```rust
    pub fn ports(&self) -> Result<Vec<Port>, &'static str> {
        let mut ports = Vec::new();
        for m in &self.modules {
            let (inputs, outputs): (&[u8], &[u8]) = if self.circle.is_some() {
                (
                    &[0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20],
```

</details>

<details><summary><code>target_nutrient()</code></summary>

Supports target nutrient within this component's responsibility; the source link opens the complete implementation.

[Source lines 205–210](C:/Thought/Projects/World/crates/world_genetics/src/body.rs:205)

```rust
    pub fn target_nutrient(&self) -> f64 {
        self.modules
            .iter()
            .map(|m| std::f64::consts::PI * m.radius * m.radius * self.density * 12.0)
            .sum()
    }
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

## Step 11: Neural topology and heterogeneous execution

Stable hidden-neuron/connection edits and body-port synchronization live in the shared genetics files above. The sparse executor preserves surviving memory, resets new identities, and supports zero/small/thousand-neuron cases. Lifetime neural growth and weight plasticity remain later work.

<details><summary><code>benchmarks/evolution.py</code></summary>

**Purpose:** Profiles actual biological observation/packing, sparse execution/transfer, native updating, cloning, and state reconciliation.

[Open the file](C:/Thought/Projects/World/benchmarks/evolution.py)

**Imports / uses — why they are here**

Shown together here, including test-local imports.

```python
import argparse
import json
from pathlib import Path
import time
import torch
from world.ecosystem import EcosystemRuntime, write_json
from world.experiments.adaptation import compact_preset
from world.presets import founder_genome
```

- `torch` supplies tensor execution, sparse indexed sums, device transfers, and explicit recurrent state.
- `json` converts data-only configurations, cohorts, checkpoints, and reports.
- `Path` makes configuration/checkpoint/artifact paths explicit.
- `time` measures/paces wall-clock work; native model time remains tick-based.
- `os` supplies flush/replacement operations and bounded Windows sharing-error handling.
- `argparse` validates user-facing run/device/seed/output options.

**Standalone functions**

<details><summary><code>main()</code></summary>

Validates CLI inputs, runs this file's documented workflow, and writes its declared outputs/errors.

[Source lines 13–19](C:/Thought/Projects/World/benchmarks/evolution.py:13)

```python
def main():
    parser = argparse.ArgumentParser(description="Profile the complete biological controller boundary and native world update.")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--decisions", type=int, default=10)
    parser.add_argument("--hidden", type=int, default=8)
    parser.add_argument("--output", type=Path, default=Path("artifacts/evolution-profile.json"))
    args = parser.parse_args()
```

</details>

**Useful syntax in this file**


</details>

## Circle survival, diagnosis and observation

The default is a circle experiment with online parameter learning and inherited variation. The original long-run collapse and failed intermediate interventions are retained. Current corrections bound intake and adult development, permit paid contact growth, move minerals conservatively, and declare grazing accessibility. The world uses the whole window with compact overlays. See the current verification report for measured outcomes and horizons. Growth, repair and reproduction remain automatic until the action interface is expanded; intelligence, a rich food web and large-population scaling are unfinished.

<details><summary><code>scripts/plain_command.py</code></summary>

**Purpose:** Captures native output as bytes, decodes UTF-8 before Windows PowerShell can corrupt it, removes decoration, and preserves failure exit codes.

[Open the file](C:/Thought/Projects/World/scripts/plain_command.py)

**Imports / uses and why**

```python
import re
import subprocess
import sys
```

- `re`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `subprocess`: Starts a child command with an argument list and captures its output and exit status.
- `sys`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Functions and nested helpers**

<details><summary><code>plain()</code></summary>

Handles plain at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 6–10](C:/Thought/Projects/World/scripts/plain_command.py:6)

```python
def plain(text):
    text = re.sub("[\U0001f000-\U0001faff\u2600-\u27bf\ufe0f]", "", text)
    if re.fullmatch(r"\s*(?:[=-]\s*){3,}", text):
        return ""
    return text.encode("ascii", "backslashreplace").decode("ascii")
```

</details>

<details><summary><code>main()</code></summary>

Coordinates this executable operation, its declared inputs, and its saved or displayed outputs.

[Source lines 13–19](C:/Thought/Projects/World/scripts/plain_command.py:13)

```python
def main():
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("A command is required.")
    with subprocess.Popen(
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>crates/world_core/src/circle_world.rs</code></summary>

**Purpose:** Implements the actual circle collider, swept movement, affordable turning, reachable food, and paid feeding actions.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/circle_world.rs)

**Imports / uses and why**

```rust
use crate::{
    agents,
    environment::Position,
    evolution::{PopulationWindow, expend},
    habitat::WindowReport,
    ids::OrganismId,
    physiology,
    resources::{AccountingError, Result},
    spatial::distance,
    transactions::Draft,
};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.

**Methods in `impl PopulationWindow<'_>`**

<details><summary><code>growth_position()</code></summary>

Finds space for a growing circle by shifting its own centre no farther than the radius increase. The new envelope contains the old body; crowded or unaffordable expansion still fails.

[Source lines 14–19](C:/Thought/Projects/World/crates/world_core/src/circle_world.rs:14)

```rust
    pub(crate) fn growth_position(
        &self,
        draft: &Draft<'_>,
        candidate: &crate::organism::Organism,
    ) -> Result<Option<Position>> {
        let id = candidate.id;
```

</details>

<details><summary><code>move_circle()</code></summary>

Sweeps the circle toward its destination, stops at first contact, and charges affordable motion and turning effort.

[Source lines 82–87](C:/Thought/Projects/World/crates/world_core/src/circle_world.rs:82)

```rust
    pub(crate) fn move_circle(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
```

</details>

<details><summary><code>feed_circle()</code></summary>

Uses the expressed circle radius and feeding reach to consume compatible nearby tissue with explicit digestion costs.

[Source lines 190–195](C:/Thought/Projects/World/crates/world_core/src/circle_world.rs:190)

```rust
    pub(crate) fn feed_circle(
        &mut self,
        draft: &mut Draft<'_>,
        id: OrganismId,
        report: &mut WindowReport,
    ) -> Result<()> {
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_core/src/display.rs</code></summary>

**Purpose:** Exports and validates a small rendering packet. It carries live circles and a bounded selected brain rather than copying the world and every genome.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/display.rs)

**Imports / uses and why**

```rust
use crate::{
    agents,
    environment::Position,
    habitat::Habitat,
    resources::{AccountingError, Result},
};
use serde::{Deserialize, Serialize};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde`: Serde converts typed Rust records to and from validated JSON.

**Struct: `DisplayOrganism`**

Carries only the fields the viewer needs for a circle and its selected statistics; it does not contain a genome or PyTorch model.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_core/src/display.rs:11)

```rust
pub struct DisplayOrganism {
    pub id: u64,
    pub parent: Option<u64>,
    pub alive: bool,
    pub position: Position,
    pub heading: f64,
```

**Struct: `DisplayFrame`**

Defines the rendering contract: dimensions, resource cells, compact organism records, summary, and an optional bounded brain inspection.

[Source lines 32–37](C:/Thought/Projects/World/crates/world_core/src/display.rs:32)

```rust
pub struct DisplayFrame {
    pub schema_version: u32,
    pub tick: u64,
    pub model_time: f64,
    pub physics_dt: f64,
    pub width: f64,
```

**Methods in `impl DisplayFrame`**

<details><summary><code>validate()</code></summary>

Rejects inconsistent or out-of-range data before it can enter a simulation or rendering operation.

[Source lines 49–54](C:/Thought/Projects/World/crates/world_core/src/display.rs:49)

```rust
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !self.width.is_finite()
            || self.width <= 0.0
            || !self.height.is_finite()
            || self.height <= 0.0
```

</details>

**Methods in `impl Habitat`**

<details><summary><code>display_json()</code></summary>

Builds a small packet of live organisms, optional inspected dead organism, resource fields, and summary. Complete checkpoints use another API.

[Source lines 173–178](C:/Thought/Projects/World/crates/world_core/src/display.rs:173)

```rust
    pub fn display_json(&self, selected: Option<u64>) -> Result<String> {
        let (width, height, columns, rows) = self.grid().dimensions();
        let mut organisms = Vec::new();
        for (id, o) in self
            .organisms()
            .iter()
```

</details>

**Functions outside impl blocks**

<details><summary><code>frame()</code></summary>

Handles frame at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 253–258](C:/Thought/Projects/World/crates/world_core/src/display.rs:253)

```rust
    fn frame() -> DisplayFrame {
        DisplayFrame {
            schema_version: 1,
            tick: 0,
            model_time: 0.0,
            physics_dt: 1.0 / 60.0,
```

</details>

<details><summary><code>display_rejects_invalid_geometry_and_duplicate_identities()</code></summary>

Handles display rejects invalid geometry and duplicate identities at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 289–294](C:/Thought/Projects/World/crates/world_core/src/display.rs:289)

```rust
    fn display_rejects_invalid_geometry_and_duplicate_identities() {
        let original = frame();
        original.validate().unwrap();
        let mut invalid = original.clone();
        invalid.organisms[0].radius = f64::NAN;
        assert!(invalid.validate().is_err());
```

</details>

<details><summary><code>display_rejects_unbounded_or_disconnected_neural_inspection()</code></summary>

Handles display rejects unbounded or disconnected neural inspection at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 305–310](C:/Thought/Projects/World/crates/world_core/src/display.rs:305)

```rust
    fn display_rejects_unbounded_or_disconnected_neural_inspection() {
        let mut invalid = frame();
        invalid.brain["edges"] = serde_json::json!([{"source":1,"target":999,"weight":1.0}]);
        assert!(invalid.validate().is_err());
        invalid = frame();
        invalid.brain["nodes"][0]["value"] = serde_json::json!(2.0);
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_viewer/src/circle_scene.rs</code></summary>

**Purpose:** Draws the new circle world, facing marks, injury color, resource fields, selected learning state, and brain graph. Geometry refreshes when a new packet arrives.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/circle_scene.rs)

**Imports / uses and why**

```rust
use crate::{
    AppModel,
    scene::{BodyTransforms, CellGlyph, CellSprites, Glyph, SceneCache},
};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use std::collections::BTreeSet;
use world_core::ids::OrganismId;
```

- `bevy`: Bevy supplies the application loop, ECS resources, input, camera, meshes, and native rendering.
- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.
- `world_core`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Functions outside impl blocks**

<details><summary><code>sync()</code></summary>

Handles sync at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 9–14](C:/Thought/Projects/World/crates/world_viewer/src/circle_scene.rs:9)

```rust
pub(crate) fn sync(
    commands: &mut Commands,
    app: &mut AppModel,
    cache: &mut SceneCache,
    materials: &mut Assets<ColorMaterial>,
    cells: &mut CellSprites,
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>python/world/brains/plastic.py</code></summary>

**Purpose:** Maintains independent sparse recurrent brains with online reward-modulated learning, inherited parameters, stable gene identities, and exact saved continuation.

[Open the file](C:/Thought/Projects/World/python/world/brains/plastic.py)

**Imports / uses and why**

```python
from copy import copy
from dataclasses import dataclass
import math
import numpy as np
import torch
from .evolving import NeuralPopulation
```

- `copy`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `dataclasses`: Declares small typed records for proposed or committed state.
- `evolving`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `math`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `numpy`: NumPy performs deterministic vectorized counter-noise arithmetic, independent of controller batch order.
- `torch`: PyTorch tensors execute independent sparse brains on the chosen CPU or CUDA device; no autograd graph is retained.

**Class: `Proposal`**

Holds tentative recurrent memory and eligibility traces plus the identities authorized to learn. The coordinator commits it only after the native decision succeeds.

[Source lines 12–16](C:/Thought/Projects/World/python/world/brains/plastic.py:12)

```python
class Proposal:
    memory: torch.Tensor
    edges: torch.Tensor
    biases: torch.Tensor
    authorized: set
```

**Class: `PlasticPopulation`**

Owns the live tensors and per-organism learning statistics. Stable organism and gene IDs keep unrelated births from resetting surviving brains.

[Source lines 19–25](C:/Thought/Projects/World/python/world/brains/plastic.py:19)

```python
class PlasticPopulation(NeuralPopulation):
    kind = "plastic_sparse_rnn_v1"
    exploration = 0.18

    def __init__(self, frame, device="cpu", previous=None, memory=None):
        super().__init__(frame, device, previous, memory)
        self.seed = frame["seed"]
```

<details><summary><code>__init__()</code></summary>

Initializes this object and its owned state so later methods share one consistent set of parameters.

[Source lines 23–29](C:/Thought/Projects/World/python/world/brains/plastic.py:23)

```python
    def __init__(self, frame, device="cpu", previous=None, memory=None):
        super().__init__(frame, device, previous, memory)
        self.seed = frame["seed"]
        self.edge_keys = [
            (a["id"], e["id"])
            for a in frame["agents"]
            for e in a["brain"]["connections"]
```

</details>

<details><summary><code>noise()</code></summary>

Generates independent exploration from seed, tick, organism ID, and neuron ID. Births and deaths elsewhere do not shift an organism's random stream.

[Source lines 46–52](C:/Thought/Projects/World/python/world/brains/plastic.py:46)

```python
    def noise(self, tick):
        with np.errstate(over="ignore"):
            value = (
                self.noise_ids[:, 0] * np.uint64(0x9E3779B97F4A7C15)
                ^ self.noise_ids[:, 1] * np.uint64(0xD1B54A32D192ED03)
                ^ np.uint64((self.seed + tick * 0x94D049BB133111EB) % (2**64))
            )
```

</details>

<details><summary><code>validate()</code></summary>

Rejects inconsistent or out-of-range data before it can enter a simulation or rendering operation.

[Source lines 70–76](C:/Thought/Projects/World/python/world/brains/plastic.py:70)

```python
    def validate(self, frame):
        super().validate(frame)
        if (
            self.weights.shape != (len(self.edge_keys),)
            or self.edge_trace.shape != self.weights.shape
            or self.biases.shape != (len(self.keys),)
            or self.bias_trace.shape != self.biases.shape
```

</details>

<details><summary><code>propose()</code></summary>

Calculates candidate neural outputs and memory without changing the committed brain. The caller can reject the whole decision safely.

[Source lines 89–95](C:/Thought/Projects/World/python/world/brains/plastic.py:89)

```python
    def propose(self, frame):
        self.validate(frame)
        previous = self.memory.clone()
        authorized = torch.zeros(len(self.keys), dtype=torch.bool, device=self.device)
        for agent, (_, begin, end, inputs, _) in zip(
            frame["agents"], self.rows, strict=True
        ):
```

</details>

<details><summary><code>reconcile()</code></summary>

Builds the next committed population, applies reward to eligible parameters, preserves surviving genes, and initializes newborn state from inherited genes.

[Source lines 163–169](C:/Thought/Projects/World/python/world/brains/plastic.py:163)

```python
    def reconcile(self, frame, proposal):
        if self._signature(frame) == self.signature:
            result = copy(self)
            result.tick = frame["tick"]
            result.memory = proposal.memory
            result.weights, result.biases = self.weights.clone(), self.biases.clone()
            result.edge_trace, result.bias_trace = (
```

</details>

<details><summary><code>snapshot()</code></summary>

Serializes the complete coordinated state needed to resume, including live learned parameters and learning history rather than only birth genes.

[Source lines 247–253](C:/Thought/Projects/World/python/world/brains/plastic.py:247)

```python
    def snapshot(self):
        payload = super().snapshot()
        payload.update(
            schema_version=2,
            edge_keys=[list(k) for k in self.edge_keys],
            weights=self.weights.cpu().tolist(),
            biases=self.biases.cpu().tolist(),
```

</details>

<details><summary><code>restore()</code></summary>

Handles restore for `PlasticPopulation`.

[Source lines 265–271](C:/Thought/Projects/World/python/world/brains/plastic.py:265)

```python
    def restore(cls, frame, payload, device="cpu"):
        result = cls(frame, device)
        input_biases = result.biases[result.input_indices].clone()
        expected = set(result.snapshot())
        if (
            not isinstance(payload, dict)
            or set(payload) != expected
```

</details>

**Functions and nested helpers**

<details><summary><code>mix()</code></summary>

Handles mix at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 54–57](C:/Thought/Projects/World/python/world/brains/plastic.py:54)

```python
            def mix(data):
                data = (data ^ (data >> np.uint64(30))) * np.uint64(0xBF58476D1CE4E5B9)
                data = (data ^ (data >> np.uint64(27))) * np.uint64(0x94D049BB133111EB)
                return data ^ (data >> np.uint64(31))
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>python/world/experiments/learning.py</code></summary>

**Purpose:** Runs a small feeding experiment with matched plastic and frozen controllers. It measures a particular learned feeding skill, not general intelligence.

[Open the file](C:/Thought/Projects/World/python/world/experiments/learning.py)

**Imports / uses and why**

```python
from copy import deepcopy
from ..ecosystem import EcosystemRuntime
from ..presets import circle_genome, circle_preset
from .adaptation import fund
```

- `adaptation`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `copy`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `ecosystem`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `presets`: Supports the file's declared operations; project-local imports share existing data types and rules.

**Functions and nested helpers**

<details><summary><code>feeding_task()</code></summary>

Places a consumer next to renewable producer tissue to isolate learning of the feeding output from navigation and reproduction.

[Source lines 8–14](C:/Thought/Projects/World/python/world/experiments/learning.py:8)

```python
def feeding_task(rules, seed=73, learning_rate=0.15):
    rules, config = circle_preset(rules, seed, consumers=1, producers=1)
    rules.update(
        world_width=16.0,
        world_height=16.0,
        grid_width=16,
        grid_height=16,
```

</details>

<details><summary><code>compare_learning()</code></summary>

Runs matched plastic and zero-learning-rate controllers under the same declared seeds and records food intake, learned changes, and resource audits.

[Source lines 32–38](C:/Thought/Projects/World/python/world/experiments/learning.py:32)

```python
def compare_learning(rules, seeds=(73, 89, 107), decisions=500):
    rows = []
    for seed in seeds:
        for condition, rate in [("plastic", 0.15), ("frozen", 0.0)]:
            task_rules, config = feeding_task(deepcopy(rules), seed, rate)
            runtime = EcosystemRuntime.create(task_rules, config)
            initial = runtime.brains.snapshot()
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>examples/learning_demo.py</code></summary>

**Purpose:** Runs the learning comparison from the terminal and writes its declared conditions, outcomes, and conservation checks.

[Open the file](C:/Thought/Projects/World/examples/learning_demo.py)

**Imports / uses and why**

```python
import argparse
import json
from pathlib import Path
from world.ecosystem import write_json
from world.experiments.learning import compare_learning
```

- `argparse`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `json`: Reads and writes the versioned configuration and state protocols.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `world`: Imports this project's native world, controller coordination, or experiment helpers.

**Functions and nested helpers**

<details><summary><code>main()</code></summary>

Coordinates this executable operation, its declared inputs, and its saved or displayed outputs.

[Source lines 9–15](C:/Thought/Projects/World/examples/learning_demo.py:9)

```python
def main():
    parser = argparse.ArgumentParser(
        description="Compare online plasticity with frozen-weight feeding controls."
    )
    parser.add_argument("--seeds", type=int, nargs="+", default=[73, 89, 107])
    parser.add_argument("--decisions", type=int, default=500)
    parser.add_argument(
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>configs/circle_rules.json</code></summary>

**Purpose:** Declares the current circle world's biological units and coefficients: external light, bounded absorption, metabolism, digestion, growth, and reproduction.

[Open the file](C:/Thought/Projects/World/configs/circle_rules.json)

[Source lines 1–6](C:/Thought/Projects/World/configs/circle_rules.json:1)

```json
{
  "schema_version": 1,
  "model_id": "circle-world-v0.3",
  "world_width": 40.0,
  "world_height": 22.5,
  "grid_width": 40,
```

The declared fields are read by the project or its build tools. JSON booleans are `true`/`false`; lockfiles pin exact dependency versions and checksums.

</details>

<details><summary><code>configs/circle_habitat.json</code></summary>

**Purpose:** Declares the current founders, one-core circle bodies, inherited neural genes, mutation rates, resource grid, and explicit population ceiling.

[Open the file](C:/Thought/Projects/World/configs/circle_habitat.json)

[Source lines 1–6](C:/Thought/Projects/World/configs/circle_habitat.json:1)

```json
{
  "schema_version": 1,
  "seed": 73,
  "initial_nutrient": 1982.0,
  "initial_energy": 4084.0,
  "mineral_nutrient_per_cell": 1.0,
```

The declared fields are read by the project or its build tools. JSON booleans are `true`/`false`; lockfiles pin exact dependency versions and checksums.

</details>

<details><summary><code>tests/test_circle_world.py</code></summary>

**Purpose:** Exercises the repaired behavior and failure boundaries: movement, contact feeding, learning, inheritance, rollback, valid observation, funded reproduction, and continuation.

[Open the file](C:/Thought/Projects/World/tests/test_circle_world.py)

**Imports / uses and why**

```python
from copy import deepcopy
import json
from pathlib import Path
import runpy
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import torch
from world._native import Habitat
from world.brains.plastic import PlasticPopulation
from world.ecosystem import EcosystemRuntime
from world.control import RunControl, RunLease
from world.experiments.adaptation import fund
from world.experiments.learning import compare_learning, feeding_task
from world.presets import circle_genome, circle_preset
        import tempfile
```

- `copy`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `json`: Reads and writes the versioned configuration and state protocols.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `runpy`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `subprocess`: Starts a child command with an argument list and captures its output and exit status.
- `sys`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `tempfile`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `torch`: PyTorch tensors execute independent sparse brains on the chosen CPU or CUDA device; no autograd graph is retained.
- `types`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `unittest`: Checks behavior and rejection paths using Python's standard test framework.
- `world`: Imports this project's native world, controller coordination, or experiment helpers.

**Class: `CircleWorldTests`**

Handles CircleWorldTests at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 41–47](C:/Thought/Projects/World/tests/test_circle_world.py:41)

```python
class CircleWorldTests(unittest.TestCase):
    def test_missing_viewer_fails_before_starting_or_publishing_a_world(self):
        entry = runpy.run_path(str(ROOT / "examples/evolution_demo.py"))["run"]
        with patch("pathlib.Path.exists", return_value=False):
            with self.assertRaisesRegex(
                FileNotFoundError, "viewer executable is missing"
            ):
```

<details><summary><code>test_missing_viewer_fails_before_starting_or_publishing_a_world()</code></summary>

Verifies missing viewer fails before starting or publishing a world, including the observable result or rejected input.

[Source lines 42–48](C:/Thought/Projects/World/tests/test_circle_world.py:42)

```python
    def test_missing_viewer_fails_before_starting_or_publishing_a_world(self):
        entry = runpy.run_path(str(ROOT / "examples/evolution_demo.py"))["run"]
        with patch("pathlib.Path.exists", return_value=False):
            with self.assertRaisesRegex(
                FileNotFoundError, "viewer executable is missing"
            ):
                entry(SimpleNamespace(viewer=True))
```

</details>

<details><summary><code>test_observer_chooses_a_valid_protocol_after_controller_family_changes()</code></summary>

Verifies observer chooses a valid protocol after controller family changes, including the observable result or rejected input.

[Source lines 50–56](C:/Thought/Projects/World/tests/test_circle_world.py:50)

```python
    def test_observer_chooses_a_valid_protocol_after_controller_family_changes(self):
        runtime = EcosystemRuntime.create(*setup())
        control = RunControl(runtime, ROOT / "artifacts/unused-checkpoint.json")
        self.assertTrue(control.lightweight)
        rules, config = setup()
        config["evolution"]["online_learning"] = False
        fixed = EcosystemRuntime.create(rules, config)
```

</details>

<details><summary><code>test_plain_output_decodes_raw_utf8_and_preserves_failure_code()</code></summary>

Verifies plain output decodes raw utf8 and preserves failure code, including the observable result or rejected input.

[Source lines 83–89](C:/Thought/Projects/World/tests/test_circle_world.py:83)

```python
    def test_plain_output_decodes_raw_utf8_and_preserves_failure_code(self):
        command = [
            sys.executable,
            str(ROOT / "scripts/plain_command.py"),
            "--",
            sys.executable,
            "-c",
```

</details>

<details><summary><code>test_circle_geometry_allows_approach_and_continued_feeding()</code></summary>

Verifies circle geometry allows approach and continued feeding, including the observable result or rejected input.

[Source lines 97–103](C:/Thought/Projects/World/tests/test_circle_world.py:97)

```python
    def test_circle_geometry_allows_approach_and_continued_feeding(self):
        rules, config = feeding_task(RULES)
        config["populations"][0]["positions"] = [{"x": 6.0, "y": 8.0}]
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        payload = runtime.snapshot()["world"]
        for item in payload["motion"]:
```

</details>

<details><summary><code>test_online_learning_improves_a_real_feeding_task()</code></summary>

Verifies online learning improves a real feeding task, including the observable result or rejected input.

[Source lines 165–171](C:/Thought/Projects/World/tests/test_circle_world.py:165)

```python
    def test_online_learning_improves_a_real_feeding_task(self):
        report = compare_learning(RULES, seeds=[73], decisions=400)
        plastic, frozen = report["runs"]
        self.assertTrue(plastic["alive"])
        self.assertGreater(plastic["parameter_change"], 0)
        self.assertEqual(frozen["parameter_change"], 0)
        self.assertGreater(plastic["food_ingested"], frozen["food_ingested"] * 1.25)
```

</details>

<details><summary><code>test_neural_controller_moves_to_food_from_a_separated_start()</code></summary>

Verifies neural controller moves to food from a separated start, including the observable result or rejected input.

[Source lines 173–179](C:/Thought/Projects/World/tests/test_circle_world.py:173)

```python
    def test_neural_controller_moves_to_food_from_a_separated_start(self):
        rules, config = setup()
        config["producers"].update(count=1, positions=[{"x": 8.0, "y": 8.0}])
        config["populations"][0].update(count=1, positions=[{"x": 6.0, "y": 8.0}])
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
        state = runtime.snapshot()
```

</details>

<details><summary><code>test_independent_noise_does_not_depend_on_batch_membership()</code></summary>

Verifies independent noise does not depend on batch membership, including the observable result or rejected input.

[Source lines 189–195](C:/Thought/Projects/World/tests/test_circle_world.py:189)

```python
    def test_independent_noise_does_not_depend_on_batch_membership(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        complete, _ = runtime.brains.propose(frame)
        for index, agent in enumerate(frame["agents"]):
            single = deepcopy(frame)
            single["agents"] = [agent]
```

</details>

<details><summary><code>test_birth_inherits_learned_weights_and_repacking_keeps_parent_learning()</code></summary>

Verifies birth inherits learned weights and repacking keeps parent learning, including the observable result or rejected input.

[Source lines 205–211](C:/Thought/Projects/World/tests/test_circle_world.py:205)

```python
    def test_birth_inherits_learned_weights_and_repacking_keeps_parent_learning(self):
        rules, config = setup(True)
        rules.update(minimum_reproduction_age=0.2, reproduction_cooldown=0.2)
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        runtime.brains.weights = runtime.brains.weights + 0.5
```

</details>

<details><summary><code>test_learning_failure_is_atomic_with_world_memory_weights_and_traces()</code></summary>

Verifies learning failure is atomic with world memory weights and traces, including the observable result or rejected input.

[Source lines 259–265](C:/Thought/Projects/World/tests/test_circle_world.py:259)

```python
    def test_learning_failure_is_atomic_with_world_memory_weights_and_traces(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(3)
        before = runtime.snapshot()
        with patch.object(
            PlasticPopulation,
            "reconcile",
```

</details>

<details><summary><code>test_conflicting_run_owners_are_rejected_and_lease_is_reusable()</code></summary>

Verifies conflicting run owners are rejected and lease is reusable, including the observable result or rejected input.

[Source lines 272–278](C:/Thought/Projects/World/tests/test_circle_world.py:272)

```python
    def test_conflicting_run_owners_are_rejected_and_lease_is_reusable(self):
        import tempfile

        with tempfile.TemporaryDirectory() as directory:
            stream = Path(directory) / "stream.json"
            with RunLease(stream):
                with self.assertRaises(RuntimeError):
```

</details>

<details><summary><code>test_bad_learned_inheritance_payload_cannot_change_the_world()</code></summary>

Verifies bad learned inheritance payload cannot change the world, including the observable result or rejected input.

[Source lines 284–290](C:/Thought/Projects/World/tests/test_circle_world.py:284)

```python
    def test_bad_learned_inheritance_payload_cannot_change_the_world(self):
        rules, config = setup(True)
        rules["minimum_reproduction_age"] = 0.1
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(1)
        batch, _ = runtime.brains.propose(runtime.frame())
```

</details>

<details><summary><code>test_learning_checkpoint_rejects_bad_parameter_state()</code></summary>

Verifies learning checkpoint rejects bad parameter state, including the observable result or rejected input.

[Source lines 306–312](C:/Thought/Projects/World/tests/test_circle_world.py:306)

```python
    def test_learning_checkpoint_rejects_bad_parameter_state(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(2)
        state = runtime.snapshot()
        for field, value in [
            ("weights", [float("nan")] * len(state["neural"]["weights"])),
            ("bias_trace", [11.0] * len(state["neural"]["bias_trace"])),
```

</details>

<details><summary><code>test_first_brain_is_created_at_a_funded_birth()</code></summary>

Verifies first brain is created at a funded birth, including the observable result or rejected input.

[Source lines 327–333](C:/Thought/Projects/World/tests/test_circle_world.py:327)

```python
    def test_first_brain_is_created_at_a_funded_birth(self):
        rules, config = setup(True)
        config["populations"] = []
        config["evolution"]["mutation"]["brain_creation_rate"] = 1.0
        rules.update(minimum_reproduction_age=0.2, reproduction_cooldown=0.2)
        fund(rules, config)
        runtime = EcosystemRuntime.create(rules, config)
```

</details>

<details><summary><code>test_reproduction_preserves_a_maintenance_reserve()</code></summary>

Verifies reproduction preserves a maintenance reserve, including the observable result or rejected input.

[Source lines 351–357](C:/Thought/Projects/World/tests/test_circle_world.py:351)

```python
    def test_reproduction_preserves_a_maintenance_reserve(self):
        counts = []
        for horizon in [24.0, 0.0]:
            rules, config = setup(True)
            config["producers"]["count"] = 0
            config["populations"][0].update(count=1, reserve_energy=14.5)
            rules.update(minimum_reproduction_age=0.1, reproduction_cooldown=5.0)
```

</details>

<details><summary><code>test_capacity_limited_birth_priority_does_not_permanently_favor_producers()</code></summary>

Verifies capacity limited birth priority does not permanently favor producers, including the observable result or rejected input.

[Source lines 366–372](C:/Thought/Projects/World/tests/test_circle_world.py:366)

```python
    def test_capacity_limited_birth_priority_does_not_permanently_favor_producers(self):
        rules, config = setup()
        rules["minimum_reproduction_age"] = 0.1
        config["producers"]["count"] = 2
        config["ecology"]["max_population"] = 5
        config["evolution"]["maintenance_reserve_time"] = 0.1
        fund(rules, config)
```

</details>

<details><summary><code>test_incremental_event_export_rejects_gaps_instead_of_dropping_events()</code></summary>

Verifies incremental event export rejects gaps instead of dropping events, including the observable result or rejected input.

[Source lines 387–393](C:/Thought/Projects/World/tests/test_circle_world.py:387)

```python
    def test_incremental_event_export_rejects_gaps_instead_of_dropping_events(self):
        rules, config = setup(True)
        rules["minimum_reproduction_age"] = 0.2
        config["evolution"]["maintenance_reserve_time"] = 0.1
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(3)
        receipt = runtime.events_since(0)
```

</details>

<details><summary><code>test_large_brain_cost_has_feasible_food_headroom()</code></summary>

Verifies large brain cost has feasible food headroom, including the observable result or rejected input.

[Source lines 409–415](C:/Thought/Projects/World/tests/test_circle_world.py:409)

```python
    def test_large_brain_cost_has_feasible_food_headroom(self):
        rules, config = setup()
        config["populations"][0]["genome"] = circle_genome(hidden=1000)
        fund(rules, config)
        brain = config["populations"][0]["genome"]["brain"]
        cost = (
            len(brain["nodes"]) * config["evolution"]["neuron_energy_per_time"]
```

</details>

<details><summary><code>test_display_packets_exclude_genomes_and_show_current_learned_parameters()</code></summary>

Verifies display packets exclude genomes and show current learned parameters, including the observable result or rejected input.

[Source lines 430–436](C:/Thought/Projects/World/tests/test_circle_world.py:430)

```python
    def test_display_packets_exclude_genomes_and_show_current_learned_parameters(self):
        runtime = EcosystemRuntime.create(*setup())
        runtime.advance(3)
        identity = runtime.frame()["agents"][0]["id"]
        packet = runtime.display(identity)
        self.assertNotIn("world", packet)
        self.assertNotIn("genome", json.dumps(packet))
```

</details>

<details><summary><code>test_plastic_cpu_cuda_proposals_match()</code></summary>

Verifies plastic cpu cuda proposals match, including the observable result or rejected input.

[Source lines 442–448](C:/Thought/Projects/World/tests/test_circle_world.py:442)

```python
    def test_plastic_cpu_cuda_proposals_match(self):
        runtime = EcosystemRuntime.create(*setup())
        frame = runtime.frame()
        cpu, proposal = runtime.brains.propose(frame)
        gpu, other = PlasticPopulation(frame, "cuda").propose(frame)
        self.assertTrue(
            torch.allclose(proposal.memory, other.memory.cpu(), atol=1e-11, rtol=1e-11)
```

</details>

**Functions and nested helpers**

<details><summary><code>setup()</code></summary>

Handles setup at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 26–32](C:/Thought/Projects/World/tests/test_circle_world.py:26)

```python
def setup(reproduction=False):
    rules, config = circle_preset(RULES, consumers=2, producers=4)
    rules.update(world_width=16, world_height=16, grid_width=16, grid_height=16)
    config["ecology"].update(reproduction=reproduction, max_population=32)
    config["evolution"]["mutation"].update(
        parameter_edits=0,
        body_edits=0,
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>crates/world_core/src/telemetry.rs</code></summary>

**Purpose:** Collects compact role-specific populations, resource compartments, local mineral access, birth/death diagnostics and computational limits without serializing genomes.

[Open the file](C:/Thought/Projects/World/crates/world_core/src/telemetry.rs)

**Imports / uses and why**

```rust
use crate::{habitat::Habitat, resources::Result};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
```

- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `serde_json`: JSON values carry the versioned bridge, checkpoint, and observer protocols.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.

**Methods in `impl Habitat`**

<details><summary><code>telemetry_json()</code></summary>

Serializes the compact native diagnostic receipt for the Python runner and lightweight observer.

[Source lines 6–11](C:/Thought/Projects/World/crates/world_core/src/telemetry.rs:6)

```rust
    pub fn telemetry_json(&self) -> Result<String> {
        serde_json::to_string(&self.telemetry()?)
            .map_err(|e| crate::resources::AccountingError::Invalid(e.to_string()))
    }

    pub(crate) fn telemetry(&self) -> Result<Value> {
```

</details>

<details><summary><code>telemetry()</code></summary>

Returns population roles, reserve/tissue compartments, local mineral access and diagnostic counters without modifying the world or including neural genomes.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_core/src/telemetry.rs:11)

```rust
    pub(crate) fn telemetry(&self) -> Result<Value> {
        let mut roles = BTreeMap::new();
        let mut producer_cells = BTreeSet::new();
        for (name, producer) in [("producer", true), ("consumer", false)] {
            let members: Vec<_> = self
                .organisms()
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>crates/world_viewer/src/hud.rs</code></summary>

**Purpose:** Lays out the full-window observer overlays, blocks world input under panels, and draws the selected live neural network with named node inspection.

[Open the file](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs)

**Imports / uses and why**

```rust
use crate::{AppModel, model::ViewerModel, scene::ViewArea};
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    sprite::Anchor,
    text::{FontSize, TextBounds},
};
use std::collections::BTreeMap;
```

- `bevy`: Bevy supplies the application loop, ECS resources, input, camera, meshes, and native rendering.
- `crate`: Uses neighboring Rust modules rather than duplicating their rules or data types.
- `std`: Uses Rust's standard collections, paths, timing, or filesystem support.

**Enum: `Part`**

Names the HUD panels so layout, visibility and pointer blocking use the same identities.

[Source lines 11–16](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:11)

```rust
enum Part {
    Stats,
    Selection,
    Brain,
    BrainFooter,
    Help,
```

**Struct: `Overlay`**

Marks one text or background entity with its panel identity; the HUD system can position and hide the corresponding entities together.

[Source lines 21–26](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:21)

```rust
pub(crate) struct Overlay {
    part: Part,
    background: bool,
}

type OverlayQuery<'w, 's> = Query<
```

**Type alias: `OverlayQuery`**

A reusable name for the following type expression. In viewer systems, aliases keep component queries readable; lifetime parameters describe the borrowed world/system data.

[Source lines 26–31](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:26)

```rust
type OverlayQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Overlay,
        &'static mut Transform,
```

**Functions outside impl blocks**

<details><summary><code>area()</code></summary>

Handles area at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 39–44](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:39)

```rust
fn area(part: Part, width: f32) -> Rect {
    let narrow = width < 760.0;
    let panel_width = (width - 32.0).min(340.0);
    match part {
        Part::Stats => Rect::from_corners(Vec2::splat(16.0), Vec2::new(16.0 + panel_width, 210.0)),
        Part::Selection => {
```

</details>

<details><summary><code>visible()</code></summary>

Handles visible at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 71–76](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:71)

```rust
fn visible(part: Part, model: &ViewerModel) -> bool {
    model.display.is_some()
        && match part {
            Part::Stats | Part::Hint => true,
            Part::Selection => model.selected.is_some() && !model.show_neural && !model.show_help,
            Part::Brain | Part::BrainFooter => model.show_neural,
```

</details>

<details><summary><code>blocks_cursor()</code></summary>

Maps a logical pointer position to overlay rectangles so clicks and wheel events do not interact with organisms behind a visible panel.

[Source lines 81–86](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:81)

```rust
pub(crate) fn blocks_cursor(model: &ViewerModel, size: Vec2, cursor: Vec2) -> bool {
    if model.display.is_none() {
        return false;
    }
    let scale = 840.0 / size.y.max(1.0);
    let p = cursor * scale;
```

</details>

<details><summary><code>setup()</code></summary>

Handles setup at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 99–104](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:99)

```rust
pub(crate) fn setup(mut commands: Commands) {
    for part in [
        Part::Stats,
        Part::Selection,
        Part::Brain,
        Part::BrainFooter,
```

</details>

<details><summary><code>text()</code></summary>

Handles text at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 139–144](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:139)

```rust
fn text(part: Part, model: &ViewerModel) -> String {
    let Some(frame) = &model.display else {
        return String::new();
    };
    let selected = model
        .selected
```

</details>

<details><summary><code>update()</code></summary>

Handles update at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 181–186](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:181)

```rust
pub(crate) fn update(
    window: Single<&Window>,
    app: Res<AppModel>,
    area_state: Res<ViewArea>,
    mut overlays: OverlayQuery<'_, '_>,
    mut gizmos: Gizmos,
```

</details>

<details><summary><code>draw_brain()</code></summary>

Handles draw brain at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 244–249](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:244)

```rust
fn draw_brain(brain: &serde_json::Value, rect: Rect, width: f32, gizmos: &mut Gizmos) {
    let Some(nodes) = brain["nodes"].as_array() else {
        return;
    };
    let positions = brain_positions(brain, rect, width);
    if let Some(edges) = brain["edges"].as_array() {
```

</details>

<details><summary><code>brain_positions()</code></summary>

Arranges bounded displayed inputs, hidden neurons and outputs into three columns shared by drawing and pointer inspection.

[Source lines 286–291](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:286)

```rust
fn brain_positions(brain: &serde_json::Value, rect: Rect, width: f32) -> BTreeMap<u64, Vec2> {
    let Some(nodes) = brain["nodes"].as_array() else {
        return BTreeMap::new();
    };
    let mut positions = BTreeMap::new();
    let top = 420.0 - rect.min.y - 300.0;
```

</details>

<details><summary><code>hover_description()</code></summary>

Identifies the displayed neuron under the pointer and shows its actual channel name and activation without touching biological state.

[Source lines 312–317](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:312)

```rust
fn hover_description(model: &ViewerModel, width: f32, cursor: Option<Vec2>) -> String {
    let default = "Hover a node for its name and activity.\nLinks: cyan positive / rust negative";
    let Some(frame) = &model.display else {
        return default.into();
    };
    let Some(cursor) = cursor else {
```

</details>

<details><summary><code>link_description()</code></summary>

Finds the displayed connection under the pointer, including a recurrent self-loop, and reports its endpoints and current weight.

[Source lines 365–370](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:365)

```rust
fn link_description(
    brain: &serde_json::Value,
    positions: &BTreeMap<u64, Vec2>,
    cursor: Vec2,
) -> Option<String> {
    let edges = brain["edges"].as_array()?;
```

</details>

<details><summary><code>overlays_fit_and_reflow_without_overlapping_stats()</code></summary>

Handles overlays fit and reflow without overlapping stats at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 397–402](C:/Thought/Projects/World/crates/world_viewer/src/hud.rs:397)

```rust
    fn overlays_fit_and_reflow_without_overlapping_stats() {
        for width in [500.0, 760.0, 1280.0, 2000.0] {
            let stats = area(Part::Stats, width);
            let brain = area(Part::Brain, width);
            assert!(stats.min.x >= 0.0 && brain.max.x <= width && brain.max.y <= 840.0);
            assert!(brain.min.x >= stats.max.x || brain.min.y >= stats.max.y);
```

</details>

**Useful syntax:** `impl` groups methods; `&self` reads, `&mut self` changes; `Option` can be absent; `Result` can fail; `?` propagates failure. `#[serde(...)]` controls the serialized contract.

</details>

<details><summary><code>python/world/telemetry.py</code></summary>

**Purpose:** Writes uniquely identified run manifests, append-only diagnostics, three rolling complete autosaves and decline/extinction captures. Restore segments remain distinguishable.

[Open the file](C:/Thought/Projects/World/python/world/telemetry.py)

**Imports / uses and why**

```python
import hashlib
import json
from pathlib import Path
import time
import uuid
from .ecosystem import write_json
        from . import _native
```

- `ecosystem`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `hashlib`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `json`: Reads and writes the versioned configuration and state protocols.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `time`: Measures wall time for pacing and achieved-speed reporting, separately from biological model time.
- `uuid`: Gives each observer session a distinct identity so stale commands are rejected.
- `world`: Imports this project's native world, controller coordination, or experiment helpers.

**Class: `RunRecorder`**

Owns the diagnostic files and checkpoint schedule for one invocation. Its unique run ID separates repeated invocations, and a segment identifies each loaded continuation.

[Source lines 10–16](C:/Thought/Projects/World/python/world/telemetry.py:10)

```python
class RunRecorder:
    def __init__(self, runtime, output, sample_ticks=120, checkpoint_ticks=6000):
        if type(sample_ticks) is not int or sample_ticks < 1:
            raise ValueError("sample_ticks must be a positive integer")
        if type(checkpoint_ticks) is not int or checkpoint_ticks < 1:
            raise ValueError("checkpoint_ticks must be a positive integer")
        self.prefix = Path(output).with_suffix("")
```

<details><summary><code>__init__()</code></summary>

Initializes this object and its owned state so later methods share one consistent set of parameters.

[Source lines 11–17](C:/Thought/Projects/World/python/world/telemetry.py:11)

```python
    def __init__(self, runtime, output, sample_ticks=120, checkpoint_ticks=6000):
        if type(sample_ticks) is not int or sample_ticks < 1:
            raise ValueError("sample_ticks must be a positive integer")
        if type(checkpoint_ticks) is not int or checkpoint_ticks < 1:
            raise ValueError("checkpoint_ticks must be a positive integer")
        self.prefix = Path(output).with_suffix("")
        self.prefix.parent.mkdir(parents=True, exist_ok=True)
```

</details>

<details><summary><code>path()</code></summary>

Handles path for `RunRecorder`.

[Source lines 32–33](C:/Thought/Projects/World/python/world/telemetry.py:32)

```python
    def path(self, suffix):
        return self.prefix.with_name(self.prefix.name + suffix)
```

</details>

<details><summary><code>start()</code></summary>

Captures the actual rules, configuration and source fingerprints, and writes an initial complete autosave before further biological decisions.

[Source lines 35–41](C:/Thought/Projects/World/python/world/telemetry.py:35)

```python
    def start(self, runtime):
        from . import _native

        state = runtime.snapshot()
        sources = [Path(_native.__file__), *Path(__file__).parent.rglob("*.py")]
        manifest = {
            "schema_version": 1,
```

</details>

<details><summary><code>sample()</code></summary>

Appends a compact audited diagnostic receipt at its sampling interval, detects cohort decline/extinction, and rotates three full checkpoints without reseeding the population.

[Source lines 64–70](C:/Thought/Projects/World/python/world/telemetry.py:64)

```python
    def sample(self, runtime, force=False):
        if runtime is not self.previous_runtime:
            self.segment += 1
            self.previous_runtime = runtime
            self.last_sample = runtime.tick - self.sample_ticks
            self.last_checkpoint = runtime.tick
            self.peaks.clear()
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

<details><summary><code>tests/test_run_diagnostics.py</code></summary>

**Purpose:** Verifies bounded intake, paid contact growth, injury/development separation, protected grazing, read-only diagnostics and complete automatic continuation captures.

[Open the file](C:/Thought/Projects/World/tests/test_run_diagnostics.py)

**Imports / uses and why**

```python
import json
import math
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from world._native import Habitat
from world.control import RunControl
from world.ecosystem import EcosystemRuntime
from world.experiments.adaptation import fund
from world.experiments.learning import feeding_task
from world.presets import circle_preset
from world.telemetry import RunRecorder
```

- `json`: Reads and writes the versioned configuration and state protocols.
- `math`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `pathlib`: Path objects address configuration, checkpoint, and output files without manual separators.
- `tempfile`: Supports the file's declared operations; project-local imports share existing data types and rules.
- `unittest`: Checks behavior and rejection paths using Python's standard test framework.
- `world`: Imports this project's native world, controller coordination, or experiment helpers.

**Class: `RunDiagnosticsTests`**

Handles RunDiagnosticsTests at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 26–32](C:/Thought/Projects/World/tests/test_run_diagnostics.py:26)

```python
class RunDiagnosticsTests(unittest.TestCase):
    def test_load_preserves_valid_neural_inspection(self):
        runtime = EcosystemRuntime.create(*fixture())
        with TemporaryDirectory() as directory:
            checkpoint = Path(directory) / "checkpoint.json"
            runtime.save(checkpoint)
            control = RunControl(runtime, checkpoint)
```

<details><summary><code>test_load_preserves_valid_neural_inspection()</code></summary>

Verifies load preserves valid neural inspection, including the observable result or rejected input.

[Source lines 27–33](C:/Thought/Projects/World/tests/test_run_diagnostics.py:27)

```python
    def test_load_preserves_valid_neural_inspection(self):
        runtime = EcosystemRuntime.create(*fixture())
        with TemporaryDirectory() as directory:
            checkpoint = Path(directory) / "checkpoint.json"
            runtime.save(checkpoint)
            control = RunControl(runtime, checkpoint)
            control.selected = 2
```

</details>

<details><summary><code>test_wounds_are_repaired_without_inflating_adult_development()</code></summary>

Verifies wounds are repaired without inflating adult development, including the observable result or rejected input.

[Source lines 50–56](C:/Thought/Projects/World/tests/test_run_diagnostics.py:50)

```python
    def test_wounds_are_repaired_without_inflating_adult_development(self):
        rules, config = feeding_task(RULES)
        config["evolution"].update(online_learning=False, repair_nutrient_per_time=0.01)
        rules["max_ingestion_nutrient_per_time"] = 3.0
        fund(rules, config)
        world = Habitat(json.dumps(rules), json.dumps(config))
        initial = config["producers"]["tissue_nutrient"]
```

</details>

<details><summary><code>test_contact_growth_moves_only_within_the_paid_expansion()</code></summary>

Verifies contact growth moves only within the paid expansion, including the observable result or rejected input.

[Source lines 88–94](C:/Thought/Projects/World/tests/test_run_diagnostics.py:88)

```python
    def test_contact_growth_moves_only_within_the_paid_expansion(self):
        rules, config = feeding_task(RULES)
        config["evolution"]["online_learning"] = False
        consumer = config["populations"][0]
        consumer.update(tissue_nutrient=2.5, reserve_nutrient=2.5, reserve_energy=12.0)
        density = 12.0 * math.pi
        old_radius = math.sqrt(2.5 / density)
```

</details>

<details><summary><code>test_grazing_leaves_declared_tissue_but_does_not_grant_immortality()</code></summary>

Verifies grazing leaves declared tissue but does not grant immortality, including the observable result or rejected input.

[Source lines 129–135](C:/Thought/Projects/World/tests/test_run_diagnostics.py:129)

```python
    def test_grazing_leaves_declared_tissue_but_does_not_grant_immortality(self):
        for light, expected_alive in [(8.0, True), (0.0, False)]:
            rules, config = feeding_task(RULES)
            config["evolution"].update(
                online_learning=False, repair_nutrient_per_time=1e-10
            )
            config["ecology"]["grazing_residual_fraction"] = 0.15
```

</details>

<details><summary><code>test_full_consumers_do_not_destroy_food_they_cannot_store()</code></summary>

Verifies full consumers do not destroy food they cannot store, including the observable result or rejected input.

[Source lines 178–184](C:/Thought/Projects/World/tests/test_run_diagnostics.py:178)

```python
    def test_full_consumers_do_not_destroy_food_they_cannot_store(self):
        amounts = []
        for full in [True, False]:
            rules, config = feeding_task(RULES)
            rules["sensor_energy_per_time"] = 1e-12
            config["evolution"].update(
                online_learning=False,
```

</details>

<details><summary><code>test_telemetry_is_read_only_and_accounts_for_all_nutrients()</code></summary>

Verifies telemetry is read only and accounts for all nutrients, including the observable result or rejected input.

[Source lines 227–233](C:/Thought/Projects/World/tests/test_run_diagnostics.py:227)

```python
    def test_telemetry_is_read_only_and_accounts_for_all_nutrients(self):
        runtime = EcosystemRuntime.create(*fixture())
        runtime.advance(3)
        before = runtime.snapshot()
        row = runtime.telemetry()
        total = sum(
            r["reserve_nutrient"] + r["tissue_nutrient"] for r in row["roles"].values()
```

</details>

<details><summary><code>test_birth_rejections_are_partitioned_by_role()</code></summary>

Verifies birth rejections are partitioned by role, including the observable result or rejected input.

[Source lines 248–254](C:/Thought/Projects/World/tests/test_run_diagnostics.py:248)

```python
    def test_birth_rejections_are_partitioned_by_role(self):
        rules, config = fixture()
        rules["minimum_reproduction_age"] = 0.1
        config["ecology"].update(reproduction=True, max_population=2)
        runtime = EcosystemRuntime.create(rules, config)
        runtime.advance(2)
        row = runtime.telemetry()
```

</details>

<details><summary><code>test_recorder_bounds_autosaves_and_preserves_exact_continuation()</code></summary>

Verifies recorder bounds autosaves and preserves exact continuation, including the observable result or rejected input.

[Source lines 267–273](C:/Thought/Projects/World/tests/test_run_diagnostics.py:267)

```python
    def test_recorder_bounds_autosaves_and_preserves_exact_continuation(self):
        runtime = EcosystemRuntime.create(*fixture())
        with TemporaryDirectory() as directory:
            output = Path(directory) / "run.json"
            recorder = RunRecorder(runtime, output, sample_ticks=6, checkpoint_ticks=6)
            recorder.sample(runtime)
            self.assertEqual(
```

</details>

<details><summary><code>test_recorder_captures_extinction_without_reseeding()</code></summary>

Verifies recorder captures extinction without reseeding, including the observable result or rejected input.

[Source lines 301–307](C:/Thought/Projects/World/tests/test_run_diagnostics.py:301)

```python
    def test_recorder_captures_extinction_without_reseeding(self):
        rules, config = fixture()
        config["populations"][0].update(
            reserve_energy=0.03, positions=[{"x": 35.0, "y": 19.0}]
        )
        config["producers"]["positions"] = [{"x": 2.0, "y": 2.0}]
        fund(rules, config)
```

</details>

**Functions and nested helpers**

<details><summary><code>fixture()</code></summary>

Handles fixture at this module boundary. Read the linked body for its input checks and state changes.

[Source lines 19–23](C:/Thought/Projects/World/tests/test_run_diagnostics.py:19)

```python
def fixture():
    rules, config = circle_preset(RULES, consumers=1, producers=1)
    config["ecology"]["reproduction"] = False
    fund(rules, config)
    return rules, config
```

</details>

**Useful syntax:** `self` is this object; `@dataclass` builds record methods; `with` scopes cleanup; `raise` rejects invalid operations. Tensor indices map organism and gene IDs to packed storage.

</details>

## Generated output

Reports/logs under `artifacts/setup/` record builds, audits, and tests. Lifecycle JSON records inventories/events; the bridge checkpoint stores fixture/controller state; habitat checkpoints store the complete diagnostic world. Viewer captures are BMP render outputs. They contain results, not another simulation implementation, so their contents are not copied into this guide.

`artifacts/setup/retired-dependency-scaffold/` is archived setup code, outside the active workspace. Its manifest/lock tested dependencies; its library was a placeholder; `platform_probe.rs` printed a success message after compiling/linking.

`.venv/`, the isolated wheel environment, `target/`, and `__pycache__/` contain installed dependencies or generated caches. Wheels, native extensions, executables, object files, and debugging symbols are build products.
