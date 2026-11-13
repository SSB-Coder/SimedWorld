use crate::{habitat::Habitat, resources::Result};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

impl Habitat {
    pub fn telemetry_json(&self) -> Result<String> {
        serde_json::to_string(&self.telemetry()?)
            .map_err(|e| crate::resources::AccountingError::Invalid(e.to_string()))
    }

    pub(crate) fn telemetry(&self) -> Result<Value> {
        let mut roles = BTreeMap::new();
        let mut producer_cells = BTreeSet::new();
        for (name, producer) in [("producer", true), ("consumer", false)] {
            let members: Vec<_> = self
                .organisms()
                .values()
                .filter(|o| {
                    o.alive
                        && if producer {
                            o.genome.photosynthetic()
                        } else {
                            o.genome.can_ingest() && !o.genome.photosynthetic()
                        }
                })
                .collect();
            let mut energy = 0.0;
            let mut nutrient = 0.0;
            let mut tissue = 0.0;
            let mut mature = 0;
            let mut neural = 0;
            for o in &members {
                energy += o.reserves.energy;
                nutrient += o.reserves.nutrient;
                tissue += o.tissue.nutrient;
                mature += usize::from(o.mature(self.rules()));
                neural += usize::from(o.brain().is_some());
                if producer {
                    producer_cells.insert(self.grid().cell_index(self.positions()[&o.id])?);
                }
            }
            let d = self.diagnostics();
            roles.insert(
                name,
                json!({"count":members.len(),"mature":mature,"neural":neural,
                "reserve_energy":energy,"reserve_nutrient":nutrient,"tissue_nutrient":tissue,
                "births":d.and_then(|d|d.births_by_role.get(name)),
                "last_birth_tick":d.and_then(|d|d.last_birth_tick.get(name)),
                "birth_blocks":d.and_then(|d|d.birth_blocks_by_role.get(name)),
                "deaths":d.and_then(|d|d.deaths_by_role.get(name))}),
            );
        }
        let cells = self.grid().cells();
        let mut guilds = BTreeMap::new();
        for organism in self.organisms().values().filter(|o| o.alive) {
            *guilds
                .entry(crate::food_web::guild(organism))
                .or_insert(0u64) += 1;
        }
        let mut guild_health = serde_json::Map::new();
        for name in [
            "producer",
            "grazer",
            "predator",
            "omnivore",
            "scavenger",
            "recycler",
        ] {
            let members: Vec<_> = self
                .organisms()
                .values()
                .filter(|o| crate::food_web::guild(o) == name)
                .collect();
            let living: Vec<_> = members.iter().copied().filter(|o| o.alive).collect();
            let mut failures = BTreeMap::<String, u64>::new();
            let mut deaths = BTreeMap::<String, u64>::new();
            let mut upkeep = 0.0;
            for organism in &members {
                if let Some(cause) = organism.death_cause {
                    *deaths
                        .entry(format!("{cause:?}").to_lowercase())
                        .or_default() += 1;
                }
                if !organism.alive {
                    continue;
                }
                upkeep += self.config.evolution.as_ref().map_or(
                    organism.tissue.nutrient * self.rules.maintenance_per_tissue_nutrient_per_time,
                    |config| crate::metabolism::upkeep(organism, &self.rules, config, 1.0).total(),
                );
                if let Some(behavior) = self
                    .diagnostics()
                    .and_then(|d| d.behavior.get(&organism.id))
                {
                    for (reason, count) in &behavior.failures {
                        *failures.entry(reason.clone()).or_default() += count;
                    }
                }
            }
            let food = &self.ecology.interactions.food_web;
            guild_health.insert(name.into(), json!({
                "alive":living.len(), "born":members.iter().filter(|o|o.parent_id.is_some()).count(), "deaths":deaths,
                "reserve_energy":living.iter().map(|o|o.reserves.energy).sum::<f64>(),
                "tissue_nutrient":living.iter().map(|o|o.tissue.nutrient).sum::<f64>(),
                "minimum_upkeep_energy_per_time":upkeep,
                "cumulative_ingested_nutrient":members.iter().map(|o|food.intake.get(&o.id).copied().unwrap_or(0.0)).sum::<f64>(),
                "cumulative_retained_food_energy":members.iter().map(|o|food.retained_energy.get(&o.id).copied().unwrap_or(0.0)).sum::<f64>(),
                "cumulative_assimilated_energy":members.iter().map(|o|food.assimilated_energy.get(&o.id).copied().unwrap_or(0.0)).sum::<f64>(),
                "cumulative_digestion_energy":members.iter().map(|o|food.digestion_energy.get(&o.id).copied().unwrap_or(0.0)).sum::<f64>(),
                "last_window_failures":failures,
            }));
        }
        let local_mineral: f64 = producer_cells
            .iter()
            .map(|i| cells[*i].mineral_nutrient())
            .sum();
        let depleted = producer_cells
            .iter()
            .filter(|i| cells[**i].mineral_nutrient() < 0.01)
            .count();
        let minerals: f64 = cells.iter().map(|c| c.mineral_nutrient()).sum();
        let detritus: f64 = cells.iter().map(|c| c.detritus().nutrient).sum();
        let corpse_nutrient: f64 = self.grid.corpses().map(|c| c.remaining.nutrient).sum();
        let corpse_energy: f64 = self.grid.corpses().map(|c| c.remaining.energy).sum();
        let filament: f64 = cells
            .iter()
            .flat_map(|c| c.filaments.values())
            .map(|r| r.nutrient)
            .sum();
        Ok(
            json!({"schema_version":1,"tick":self.tick(),"model_time":self.tick() as f64*self.rules().physics_dt,
            "model_id":self.rules().model_id,"seed":self.config.seed,"roles":roles,"guilds":guilds,
            "food_web":self.ecology.interactions.food_web,
            "guild_health":guild_health,
            "social":if self.config.evolution.is_some() {crate::social::metrics(self)} else {serde_json::Value::Null},
            "interactions":{"filaments":self.ecology.interactions.filaments.len(),"signals":self.ecology.interactions.signals.len()},
            "resources":{"mineral_nutrient":minerals,"detritus_nutrient":detritus+filament+corpse_nutrient,
                "corpse_count":self.grid.corpses().count(),"corpse_nutrient":corpse_nutrient,"corpse_energy":corpse_energy,
                "free_detritus_nutrient":detritus,"filament_nutrient":filament,
                "producer_cell_count":producer_cells.len(),"producer_cell_mineral":local_mineral,
                "depleted_producer_cells":depleted},"ecology":self.metrics(),
            "limits":{"max_population":self.config.ecology.max_population,
                "population_ceiling_reached":self.organisms().values().filter(|o|o.alive).count()>=self.config.ecology.max_population},
            "grazing_residual_fraction":self.config.ecology.grazing_residual_fraction,
            "last_window":self.last_update,"costs":self.diagnostics().map(|d|&d.costs),
            "growth_blocks":self.diagnostics().map(|d|&d.growth_blocks),
            "blocked_moves":self.diagnostics().map(|d|d.blocked_moves)}),
        )
    }
}
