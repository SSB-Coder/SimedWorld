"""Atomic resource transactions for the v0 rule contract.

This is an accounting reference, not a spatial simulator or tick scheduler.
The caller supplies one authorized integration window/contact result per process.
"""

from dataclasses import dataclass, replace
from functools import wraps
from inspect import signature
import math
from types import MappingProxyType
from typing import Mapping

from .rules import WorldRules, finite_nonnegative


class InvariantViolation(RuntimeError):
    """The complete inventory no longer matches its declared sources."""


@dataclass(frozen=True)
class Resources:
    nutrient: float = 0.0
    energy: float = 0.0

    def __post_init__(self) -> None:
        finite_nonnegative(self.nutrient, "nutrient")
        finite_nonnegative(self.energy, "energy")

    def plus(self, other: "Resources") -> "Resources":
        return Resources(self.nutrient + other.nutrient, self.energy + other.energy)

    def minus(self, other: "Resources") -> "Resources":
        if other.nutrient > self.nutrient or other.energy > self.energy:
            raise ValueError("Insufficient resources")
        return Resources(self.nutrient - other.nutrient, self.energy - other.energy)


@dataclass(frozen=True)
class Genome:
    photosynthetic: bool = False
    can_ingest: bool = False
    # Inherited parameter fixture, not an executing neural network.
    initial_weights: tuple[float, ...] = ()

    def __post_init__(self) -> None:
        if type(self.photosynthetic) is not bool or type(self.can_ingest) is not bool:
            raise ValueError("Capabilities must be booleans")
        if not isinstance(self.initial_weights, tuple):
            raise ValueError("Inherited weights must be an immutable tuple")
        for weight in self.initial_weights:
            if isinstance(weight, bool) or not isinstance(weight, (float, int)) or not math.isfinite(weight):
                raise ValueError("Inherited parameters must be finite")


@dataclass(frozen=True)
class Organism:
    organism_id: str
    genome: Genome
    tissue: Resources
    reserves: Resources
    age: float = 0.0
    alive: bool = True
    parent_id: str | None = None
    next_reproduction_age: float = 0.0
    death_cause: str | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.organism_id, str) or not self.organism_id:
            raise ValueError("Organism ID must be a nonempty string")
        if not isinstance(self.genome, Genome) or not isinstance(self.tissue, Resources) or not isinstance(self.reserves, Resources):
            raise ValueError("Organism requires typed genome, tissue and reserve records")
        if type(self.alive) is not bool:
            raise ValueError("alive must be boolean")
        if self.parent_id is not None and (not isinstance(self.parent_id, str) or not self.parent_id):
            raise ValueError("parent_id must be a nonempty string or None")
        if self.death_cause is not None and self.death_cause not in {"starvation", "consumption", "damage"}:
            raise ValueError("Unsupported death cause")
        finite_nonnegative(self.age, "age")
        finite_nonnegative(self.next_reproduction_age, "next_reproduction_age")

    def mature(self, rules: WorldRules) -> bool:
        return (self.alive and self.age >= rules.minimum_reproduction_age
                and self.tissue.nutrient >= rules.maturity_tissue_nutrient)


def transaction(method):
    """Commit once, or restore every resource/organism/event field on failure."""
    @wraps(method)
    def checked(self, *args, **kwargs):
        self.audit()
        before = self._checkpoint()
        try:
            result = method(self, *args, **kwargs)
            self.audit()
            arguments = dict(signature(method).bind(self, *args, **kwargs).arguments)
            arguments.pop("self")
            self._events.append({"sequence": len(self._events) + 1,
                                 "operation": method.__name__, "arguments": arguments})
            return result
        except Exception:
            self._restore(before)
            raise
    return checked


class Ledger:
    def __init__(self, rules: WorldRules, *, mineral_nutrient: float,
                 organisms: list[Organism], initial_nutrient: float,
                 initial_energy: float, light_available: float = 0.0,
                 detritus: Resources = Resources()):
        self.rules = rules
        for name, value in (("mineral_nutrient", mineral_nutrient), ("initial_nutrient", initial_nutrient),
                            ("initial_energy", initial_energy), ("light_available", light_available)):
            finite_nonnegative(value, name)
        self._organisms = {organism.organism_id: organism for organism in organisms}
        if len(self._organisms) != len(organisms):
            raise ValueError("Initial organism IDs must be unique")
        self._mineral = mineral_nutrient
        self._detritus = detritus
        self._initial_nutrient = initial_nutrient
        self._initial_energy = initial_energy
        self._light_available = light_available
        self._absorbed_light = 0.0
        self._heat = 0.0
        self._mechanical = 0.0
        self._events: list[dict] = []
        self.audit()

    @property
    def organisms(self) -> Mapping[str, Organism]:
        # Snapshot readers must not observe tentative writes.
        return MappingProxyType(dict(self._organisms))

    @property
    def events(self) -> tuple[dict, ...]:
        # Keep callers from mutating committed history.
        return tuple({**event, "arguments": dict(event["arguments"])} for event in self._events)

    def _checkpoint(self):
        return (dict(self._organisms), self._mineral, self._detritus, self._light_available,
                self._absorbed_light, self._heat, self._mechanical, len(self._events))

    def _restore(self, state) -> None:
        (self._organisms, self._mineral, self._detritus, self._light_available,
         self._absorbed_light, self._heat, self._mechanical, event_count) = state
        del self._events[event_count:]

    def _living(self, organism_id: str) -> Organism:
        if organism_id not in self._organisms:
            raise ValueError(f"Unknown organism: {organism_id}")
        organism = self._organisms[organism_id]
        if not organism.alive:
            raise ValueError(f"Organism is dead: {organism_id}")
        return organism

    def _rate_check(self, amount: float, duration: float, rate: float) -> None:
        finite_nonnegative(amount, "amount", positive=True)
        finite_nonnegative(duration, "duration", positive=True)
        if amount > rate * duration:
            raise ValueError("Requested amount exceeds this integration window's process capacity")

    def _set_reserves(self, organism: Organism, reserves: Resources) -> Organism:
        n_cap = organism.tissue.nutrient * self.rules.reserve_nutrient_capacity_per_tissue_nutrient
        e_cap = organism.tissue.nutrient * self.rules.reserve_energy_capacity_per_tissue_nutrient
        # Account for overflow as mineral return or heat.
        self._mineral += max(0.0, reserves.nutrient - n_cap)
        self._heat += max(0.0, reserves.energy - e_cap)
        return replace(organism, reserves=Resources(min(reserves.nutrient, n_cap), min(reserves.energy, e_cap)))

    def _debit_energy(self, organism: Organism, amount: float) -> Organism:
        finite_nonnegative(amount, "energy", positive=True)
        return replace(organism, reserves=organism.reserves.minus(Resources(energy=amount)))

    def _die(self, organism: Organism, cause: str) -> None:
        self._detritus = self._detritus.plus(organism.tissue).plus(organism.reserves)
        self._organisms[organism.organism_id] = replace(
            organism, tissue=Resources(), reserves=Resources(), alive=False, death_cause=cause)

    def _after_tissue_loss(self, organism: Organism, cause: str) -> None:
        if organism.tissue.nutrient < self.rules.minimum_viable_tissue_nutrient:
            self._die(organism, cause)
        else:
            self._organisms[organism.organism_id] = self._set_reserves(organism, organism.reserves)

    def audit(self) -> dict[str, float]:
        n_live = math.fsum(o.tissue.nutrient + o.reserves.nutrient for o in self._organisms.values())
        e_live = math.fsum(o.tissue.energy + o.reserves.energy for o in self._organisms.values())
        nutrient = math.fsum((self._mineral, self._detritus.nutrient, n_live))
        energy = math.fsum((self._detritus.energy, e_live, self._heat, self._mechanical))
        expected_energy = self._initial_energy + self._absorbed_light
        for name, value in (("minerals", self._mineral), ("heat", self._heat),
                            ("mechanical", self._mechanical), ("absorbed light", self._absorbed_light),
                            ("available light", self._light_available)):
            finite_nonnegative(value, name)
        def close(a, b):
            return math.isclose(a, b, rel_tol=self.rules.relative_tolerance,
                                abs_tol=self.rules.absolute_tolerance)
        for organism_id, organism in self._organisms.items():
            if organism_id != organism.organism_id:
                raise InvariantViolation("Organism lookup identity mismatch")
            if not close(organism.tissue.energy, organism.tissue.nutrient * self.rules.tissue_energy_per_nutrient):
                raise InvariantViolation("Tissue embodied energy is inconsistent")
            if organism.alive:
                if organism.tissue.nutrient < self.rules.minimum_viable_tissue_nutrient or organism.death_cause is not None:
                    raise InvariantViolation("Living organism has invalid vital tissue or death state")
                if organism.reserves.nutrient > organism.tissue.nutrient * self.rules.reserve_nutrient_capacity_per_tissue_nutrient:
                    raise InvariantViolation("Nutrient reserve exceeds capacity")
                if organism.reserves.energy > organism.tissue.nutrient * self.rules.reserve_energy_capacity_per_tissue_nutrient:
                    raise InvariantViolation("Energy reserve exceeds capacity")
            elif organism.tissue != Resources() or organism.reserves != Resources() or not organism.death_cause:
                raise InvariantViolation("Dead organism retains resources or lacks a cause")
        if not close(nutrient, self._initial_nutrient) or not close(energy, expected_energy):
            raise InvariantViolation(f"Unbalanced inventory: nutrient {nutrient}/{self._initial_nutrient}; energy {energy}/{expected_energy}")
        return {"nutrient": nutrient, "energy": energy,
                "nutrient_error": nutrient - self._initial_nutrient,
                "energy_error": energy - expected_energy}

    def summary(self, label: str) -> dict:
        totals = self.audit()
        return {"label": label, "mineral_nutrient": self._mineral,
                "living_nutrient": math.fsum(o.tissue.nutrient + o.reserves.nutrient for o in self._organisms.values()),
                "living_energy": math.fsum(o.tissue.energy + o.reserves.energy for o in self._organisms.values()),
                "detritus_nutrient": self._detritus.nutrient, "detritus_energy": self._detritus.energy,
                "heat": self._heat, "mechanical_energy": self._mechanical,
                "absorbed_light": self._absorbed_light,
                "living_count": sum(o.alive for o in self._organisms.values()),
                "total_nutrient": totals["nutrient"], "accounted_energy": totals["energy"]}

    @transaction
    def photosynthesize(self, organism_id: str, light_energy: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        if not organism.genome.photosynthetic:
            raise ValueError("Organism has no light-capture capability")
        self._rate_check(light_energy, duration, self.rules.max_absorbed_light_per_time)
        if light_energy > self._light_available:
            raise ValueError("Insufficient externally available light")
        self._light_available -= light_energy
        self._absorbed_light += light_energy
        usable = light_energy * self.rules.photosynthesis_efficiency
        self._heat += light_energy - usable
        self._organisms[organism_id] = self._set_reserves(organism, organism.reserves.plus(Resources(energy=usable)))

    @transaction
    def absorb(self, organism_id: str, nutrient: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        self._rate_check(nutrient, duration, self.rules.max_absorption_nutrient_per_time)
        if not organism.genome.photosynthetic:
            raise ValueError("No passive mineral-uptake capability in this v0 configuration")
        if nutrient > self._mineral:
            raise ValueError("Insufficient available mineral nutrients")
        self._mineral -= nutrient
        self._organisms[organism_id] = self._set_reserves(organism, organism.reserves.plus(Resources(nutrient=nutrient)))

    @transaction
    def feed(self, consumer_id: str, target_id: str, tissue_nutrient: float,
             *, contact: bool, duration: float = 1.0) -> None:
        consumer, target = self._living(consumer_id), self._living(target_id)
        if consumer_id == target_id or not consumer.genome.can_ingest or contact is not True:
            raise ValueError("Feeding requires ingestion capability and authorized contact with another organism")
        self._rate_check(tissue_nutrient, duration, self.rules.max_ingestion_nutrient_per_time)
        bite = Resources(tissue_nutrient, tissue_nutrient * self.rules.tissue_energy_per_nutrient)
        remaining = target.tissue.minus(bite)
        consumer = self._debit_energy(consumer, tissue_nutrient * self.rules.digestion_energy_per_ingested_nutrient)
        self._heat += tissue_nutrient * self.rules.digestion_energy_per_ingested_nutrient
        assimilated = Resources(bite.nutrient * self.rules.nutrient_assimilation_fraction,
                                bite.energy * self.rules.energy_assimilation_fraction)
        self._detritus = self._detritus.plus(bite.minus(assimilated))
        self._organisms[consumer_id] = self._set_reserves(consumer, consumer.reserves.plus(assimilated))
        self._after_tissue_loss(replace(target, tissue=remaining), "consumption")

    @transaction
    def grow(self, organism_id: str, tissue_nutrient: float, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        self._rate_check(tissue_nutrient, duration, self.rules.max_growth_nutrient_per_time)
        embodied = tissue_nutrient * self.rules.tissue_energy_per_nutrient
        heat = tissue_nutrient * self.rules.construction_heat_per_nutrient
        reserves = organism.reserves.minus(Resources(tissue_nutrient, embodied + heat))
        self._organisms[organism_id] = replace(organism, reserves=reserves,
                                              tissue=organism.tissue.plus(Resources(tissue_nutrient, embodied)))
        self._heat += heat

    @transaction
    def maintain(self, organism_id: str, *, duration: float = 1.0) -> None:
        organism = self._living(organism_id)
        finite_nonnegative(duration, "duration", positive=True)
        required = organism.tissue.nutrient * self.rules.maintenance_per_tissue_nutrient_per_time * duration
        paid = min(required, organism.reserves.energy)
        updated = replace(organism, age=organism.age + duration,
                          reserves=organism.reserves.minus(Resources(energy=paid)))
        self._heat += paid
        if paid < required:
            self._die(updated, "starvation")
        else:
            self._organisms[organism_id] = updated

    @transaction
    def reproduce(self, parent_id: str, child_id: str, *, placement_available: bool) -> Organism:
        parent = self._living(parent_id)
        if not isinstance(child_id, str) or not child_id or child_id in self._organisms:
            raise ValueError("Child ID must be new and nonempty")
        if not parent.mature(self.rules) or parent.age < parent.next_reproduction_age or placement_available is not True:
            raise ValueError("Reproduction requires maturity, cooldown completion and authorized placement")
        tissue = Resources(self.rules.offspring_tissue_nutrient,
                           self.rules.offspring_tissue_nutrient * self.rules.tissue_energy_per_nutrient)
        reserves = Resources(self.rules.offspring_reserve_nutrient, self.rules.offspring_reserve_energy)
        heat = tissue.nutrient * self.rules.construction_heat_per_nutrient
        investment = tissue.plus(reserves).plus(Resources(energy=heat))
        remaining = parent.reserves.minus(investment)
        child = Organism(child_id, replace(parent.genome), tissue, reserves, parent_id=parent_id)
        self._organisms[parent_id] = replace(parent, reserves=remaining,
                                             next_reproduction_age=parent.age + self.rules.reproduction_cooldown)
        self._organisms[child_id] = child
        self._heat += heat
        return child

    @transaction
    def damage(self, organism_id: str, tissue_nutrient: float) -> None:
        organism = self._living(organism_id)
        finite_nonnegative(tissue_nutrient, "damaged tissue", positive=True)
        lost = Resources(tissue_nutrient, tissue_nutrient * self.rules.tissue_energy_per_nutrient)
        remaining = organism.tissue.minus(lost)
        self._detritus = self._detritus.plus(lost)
        self._after_tissue_loss(replace(organism, tissue=remaining), "damage")

    @transaction
    def decompose(self, *, fraction: float, duration: float = 1.0) -> None:
        finite_nonnegative(fraction, "fraction", positive=True)
        finite_nonnegative(duration, "duration", positive=True)
        if fraction > 1 or self._detritus == Resources():
            raise ValueError("Decomposition requires a fraction at most 1 of existing detritus")
        consumed = (self._detritus if fraction == 1 else
                    Resources(self._detritus.nutrient * fraction, self._detritus.energy * fraction))
        if (consumed.nutrient > self.rules.max_decomposition_nutrient_per_time * duration
                or consumed.energy > self.rules.max_decomposition_energy_per_time * duration):
            raise ValueError("Decomposition exceeds its nutrient or energy processing rate")
        self._detritus = self._detritus.minus(consumed)
        self._mineral += consumed.nutrient
        self._heat += consumed.energy

    @transaction
    def sense(self, organism_id: str, *, duration: float = 1.0) -> None:
        finite_nonnegative(duration, "duration", positive=True)
        organism = self._living(organism_id)
        cost = self.rules.sensor_energy_per_time * duration
        self._organisms[organism_id] = self._debit_energy(organism, cost)
        self._heat += cost

    @transaction
    def actuate(self, organism_id: str, energy: float) -> None:
        organism = self._living(organism_id)
        self._organisms[organism_id] = self._debit_energy(organism, energy)
        work = energy * self.rules.movement_efficiency
        self._mechanical += work
        self._heat += energy - work

    @transaction
    def dissipate_motion(self, energy: float) -> None:
        finite_nonnegative(energy, "mechanical dissipation", positive=True)
        if energy > self._mechanical:
            raise ValueError("Cannot dissipate unavailable mechanical energy")
        self._mechanical -= energy
        self._heat += energy
