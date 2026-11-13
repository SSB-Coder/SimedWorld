"""Versioned, dimensionless first-world rules using only the standard library."""

from dataclasses import dataclass, fields
import json
import math
from pathlib import Path


def finite_nonnegative(value: float, name: str, *, positive: bool = False) -> None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{name} must be a finite number")
    if not math.isfinite(value) or value < 0 or (positive and value == 0):
        raise ValueError(f"{name} must be finite and {'positive' if positive else 'nonnegative'}")


@dataclass(frozen=True)
class WorldRules:
    schema_version: int
    model_id: str
    world_width: float
    world_height: float
    grid_width: int
    grid_height: int
    physics_dt: float
    controller_interval_ticks: int
    ecology_interval_ticks: int
    tissue_energy_per_nutrient: float
    construction_heat_per_nutrient: float
    maintenance_per_tissue_nutrient_per_time: float
    minimum_viable_tissue_nutrient: float
    maturity_tissue_nutrient: float
    minimum_reproduction_age: float
    reserve_nutrient_capacity_per_tissue_nutrient: float
    reserve_energy_capacity_per_tissue_nutrient: float
    nutrient_assimilation_fraction: float
    energy_assimilation_fraction: float
    digestion_energy_per_ingested_nutrient: float
    max_ingestion_nutrient_per_time: float
    max_absorption_nutrient_per_time: float
    max_growth_nutrient_per_time: float
    max_decomposition_nutrient_per_time: float
    max_decomposition_energy_per_time: float
    photosynthesis_efficiency: float
    max_absorbed_light_per_time: float
    offspring_tissue_nutrient: float
    offspring_reserve_nutrient: float
    offspring_reserve_energy: float
    reproduction_cooldown: float
    sensor_energy_per_time: float
    movement_efficiency: float
    absolute_tolerance: float
    relative_tolerance: float

    def __post_init__(self) -> None:
        if type(self.schema_version) is not int or self.schema_version != 1:
            raise ValueError("Only world-rules schema_version 1 is supported")
        if not isinstance(self.model_id, str) or not self.model_id.strip():
            raise ValueError("model_id must be a nonempty string")
        for field in fields(self):
            if field.name not in {"schema_version", "model_id"}:
                finite_nonnegative(getattr(self, field.name), field.name, positive=True)
        for name in ("grid_width", "grid_height", "controller_interval_ticks", "ecology_interval_ticks"):
            if type(getattr(self, name)) is not int:
                raise ValueError(f"{name} must be an integer")
        for name in ("nutrient_assimilation_fraction", "energy_assimilation_fraction",
                     "photosynthesis_efficiency", "movement_efficiency"):
            if getattr(self, name) > 1:
                raise ValueError(f"{name} cannot exceed 1")
        if self.maturity_tissue_nutrient < self.minimum_viable_tissue_nutrient:
            raise ValueError("Mature tissue threshold cannot be below viable tissue threshold")
        if self.offspring_tissue_nutrient < self.minimum_viable_tissue_nutrient:
            raise ValueError("Configured initial offspring tissue must be viable")
        if self.offspring_reserve_nutrient > self.offspring_tissue_nutrient * self.reserve_nutrient_capacity_per_tissue_nutrient:
            raise ValueError("Offspring nutrient investment exceeds its capacity")
        if self.offspring_reserve_energy > self.offspring_tissue_nutrient * self.reserve_energy_capacity_per_tissue_nutrient:
            raise ValueError("Offspring energy investment exceeds its capacity")


def load_rules(path: str | Path | None = None) -> WorldRules:
    source = Path(path) if path is not None else Path(__file__).resolve().parents[1] / "configs/world_rules_v0.json"
    payload = json.loads(source.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("Rules must be a JSON object")
    return WorldRules(**payload)
