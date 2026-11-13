use crate::{agents, habitat::Habitat, organism::Organism, spatial::distance};
use world_genetics::body::Diet;

pub(crate) fn organic_options(
    world: &Habitat,
    observer: &Organism,
    range: f64,
) -> Vec<(crate::environment::Position, crate::resources::Resources)> {
    let at = world.positions[&observer.id];
    let config = world.config.evolution.as_ref().expect("evolution");
    let fraction =
        crate::food_web::detritus_energy_fraction(observer, &world.rules, config.embodied_ecology);
    let viable = |food: crate::resources::Resources| {
        food.nutrient > 0.001
            && food.energy * fraction
                > food.nutrient * world.rules.digestion_energy_per_ingested_nutrient
    };
    let mut options = Vec::new();
    for index in world.grid.nearby_cells(at, range) {
        let cell = &world.grid.cells[index];
        let p = world.grid.food_position(index, at);
        if viable(cell.detritus)
            && distance(at, p) <= range
            && crate::terrain::visible(&config.terrain, at, p)
        {
            options.push((p, cell.detritus));
        }
    }
    for corpse in world.grid.corpses() {
        let p = corpse.food_position(at);
        if viable(corpse.remaining)
            && distance(at, p) <= range
            && crate::terrain::visible(&config.terrain, at, p)
        {
            options.push((p, corpse.remaining));
        }
    }
    options
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FoodChoice {
    pub signals: [f64; 4],
    pub net_energy: f64,
    pub unit_energy: f64,
    pub trip_energy: f64,
}

pub(crate) fn details(
    world: &Habitat,
    observer: &Organism,
    range: f64,
    candidates: &[crate::ids::OrganismId],
) -> [FoodChoice; 2] {
    let at = world.positions[&observer.id];
    let heading = world.ecology.headings[&observer.id];
    let config = world.config.evolution.as_ref().expect("evolution");
    let radius = agents::circles(observer, at, heading).expect("audited body")[0].radius;
    let nominal_speed = world.config.ecology.consumer_speed
        * observer
            .genome
            .body
            .as_ref()
            .and_then(|b| b.circle.as_ref())
            .map_or(1.0, |c| c.speed);
    let recent_speed = world
        .diagnostics()
        .and_then(|d| d.behavior.get(&observer.id))
        .map_or(0.0, |b| {
            b.distance / (world.rules.physics_dt * world.rules.ecology_interval_ticks as f64)
        });
    let speed = if recent_speed > 1e-6 {
        recent_speed.min(nominal_speed)
    } else {
        nominal_speed * 0.5
    };
    let upkeep = crate::metabolism::upkeep(observer, &world.rules, config, 1.0).total();
    let mut options = Vec::new();
    let mut add =
        |p: crate::environment::Position, food: f64, unit_energy: f64, contact_radius: f64| {
            let d = distance(at, p);
            let gap = (d - contact_radius).max(0.0);
            let trip_energy = gap * world.config.ecology.movement_energy_per_distance
                + gap / speed.max(1e-6) * upkeep;
            let gain = food * unit_energy;
            let angle = if d < 1e-12 {
                0.0
            } else {
                (p.y - at.y).atan2(p.x - at.x) - heading
            };
            options.push((
                d,
                if config.energy_budgeting {
                    gain - trip_energy
                } else {
                    food / (d + 0.25)
                },
                FoodChoice {
                    signals: [
                        angle.sin(),
                        angle.cos(),
                        (1.0 - d / range.max(1e-12)).max(0.0),
                        food / (observer.tissue.nutrient + food),
                    ],
                    net_energy: gain - trip_energy,
                    unit_energy,
                    trip_energy,
                },
            ));
        };
    for id in candidates {
        let other = &world.organisms()[id];
        let food = agents::available_food(
            observer,
            other,
            world.config.ecology.grazing_residual_fraction,
        );
        if food <= 1e-8 || !crate::development::detected(world, observer, other, range) {
            continue;
        }
        let p = world.positions[id];
        let other_radius =
            agents::circles(other, p, world.ecology.headings[id]).expect("audited body")[0].radius;
        add(
            p,
            food,
            world.rules.tissue_energy_per_nutrient * world.rules.energy_assimilation_fraction
                - world.rules.digestion_energy_per_ingested_nutrient,
            radius + other_radius + 0.08,
        );
    }
    if matches!(
        observer.genome.body.as_ref().expect("body").diet,
        Diet::Predator | Diet::Omnivore | Diet::Scavenger | Diet::Recycler
    ) {
        let fraction = crate::food_web::detritus_energy_fraction(
            observer,
            &world.rules,
            config.embodied_ecology,
        );
        for (p, food) in organic_options(world, observer, range) {
            add(
                p,
                food.nutrient,
                food.energy * fraction / food.nutrient.max(1e-12)
                    - world.rules.digestion_energy_per_ingested_nutrient,
                radius + 0.08,
            );
        }
    }
    options.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some(nearest) = options.first() else {
        return [FoodChoice::default(); 2];
    };
    let profitable = options
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("nonempty");
    let prefer = world
        .diagnostics()
        .and_then(|d| d.behavior.get(&observer.id))
        .and_then(|b| b.requested.get(&124))
        .copied()
        .unwrap_or(0.0)
        > 0.5;
    if prefer {
        [profitable.2, nearest.2]
    } else {
        [nearest.2, profitable.2]
    }
}

pub(crate) fn contact_energy(world: &Habitat, observer: &Organism, duration: f64) -> f64 {
    let available = observer.reserves.energy
        + crate::metabolism::fuel_available(
            observer,
            &world.rules,
            world.config.evolution.as_ref().expect("evolution"),
            duration,
        );
    if available
        >= crate::metabolism::upkeep(
            observer,
            &world.rules,
            world.config.evolution.as_ref().expect("evolution"),
            duration,
        )
        .total()
    {
        return 0.0;
    }
    let at = world.positions[&observer.id];
    let radius = agents::circles(observer, at, world.ecology.headings[&observer.id])
        .expect("audited body")[0]
        .radius;
    let limit = world.rules.max_ingestion_nutrient_per_time * duration;
    let net = world.rules.tissue_energy_per_nutrient * world.rules.energy_assimilation_fraction
        - world.rules.digestion_energy_per_ingested_nutrient;
    let mut credit = 0.0_f64;
    for other in world
        .organisms()
        .values()
        .filter(|o| agents::edible(observer, o))
    {
        let p = world.positions[&other.id];
        let r = agents::circles(other, p, world.ecology.headings[&other.id]).expect("audited body")
            [0]
        .radius;
        if distance(at, p) <= radius + r + 0.08
            && crate::terrain::visible(
                &world.config.evolution.as_ref().expect("evolution").terrain,
                at,
                p,
            )
        {
            credit = credit.max(
                agents::available_food(
                    observer,
                    other,
                    world.config.ecology.grazing_residual_fraction,
                )
                .min(limit)
                    * net.max(0.0),
            );
        }
    }
    if matches!(
        observer.genome.body.as_ref().expect("body").diet,
        Diet::Predator | Diet::Omnivore | Diet::Scavenger | Diet::Recycler
    ) {
        let fraction = crate::food_web::detritus_energy_fraction(
            observer,
            &world.rules,
            world
                .config
                .evolution
                .as_ref()
                .expect("evolution")
                .embodied_ecology,
        );
        for (_, food) in organic_options(world, observer, radius + 0.08) {
            credit = credit.max(
                food.nutrient.min(limit)
                    * (food.energy * fraction / food.nutrient.max(1e-12)
                        - world.rules.digestion_energy_per_ingested_nutrient)
                        .max(0.0),
            );
        }
    }
    credit
}
