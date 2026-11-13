use crate::{
    agents, environment::Position, habitat::Habitat, organism::Organism, spatial::distance,
};

pub fn observations(
    world: &Habitat,
    observer: &Organism,
    range: f64,
    candidates: &[crate::ids::OrganismId],
) -> [f64; 18] {
    let mut v = [0.0; 18];
    let at = world.positions[&observer.id];
    let heading = world.ecology.headings[&observer.id];
    let pigment = crate::food_web::adaptations(observer).map_or(0.5, |a| a.pigment);
    let patches = &world.config.evolution.as_ref().expect("evolution").terrain;
    let radius = agents::circles(observer, at, heading).expect("audited body")[0].radius;
    if observer.genome.body.as_ref().is_some_and(|b| {
        matches!(
            b.diet,
            world_genetics::body::Diet::Predator
                | world_genetics::body::Diet::Omnivore
                | world_genetics::body::Diet::Scavenger
                | world_genetics::body::Diet::Recycler
        )
    }) && !crate::foraging::organic_options(world, observer, radius + 0.08).is_empty()
    {
        v[10] = 1.0;
    }
    let (mut count, mut weight) = (0.0_f64, 0.0_f64);
    let (mut x, mut y, mut sin, mut cos) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let mut unfamiliar = None;
    for other in candidates
        .iter()
        .map(|id| &world.organisms()[id])
        .filter(|o| o.alive && o.id != observer.id)
    {
        let p = world.positions[&other.id];
        let d = distance(at, p);
        if d > range || !crate::development::detected(world, observer, other, range) {
            continue;
        }
        let other_radius = agents::circles(other, p, world.ecology.headings[&other.id])
            .expect("audited body")[0]
            .radius;
        if agents::available_food(
            observer,
            other,
            world.config.ecology.grazing_residual_fraction,
        ) > 1e-8
            && d <= radius + other_radius + 0.08
        {
            v[10] = 1.0;
        }
        if other.genome.photosynthetic() {
            continue;
        }
        count += 1.0;
        if d < radius + other_radius + 0.3 {
            v[5] += 1.0;
        }
        let similarity = (1.0
            - (pigment - crate::food_web::adaptations(other).map_or(0.5, |a| a.pigment)).abs()
                * 4.0)
            .max(0.0);
        let w = similarity * (1.0 - d / range).max(0.0);
        x += (p.x - at.x) * w;
        y += (p.y - at.y) * w;
        sin += (world.ecology.headings[&other.id] - heading).sin() * w;
        cos += (world.ecology.headings[&other.id] - heading).cos() * w;
        weight += w;
        if agents::threatens(world, observer, other)
            && unfamiliar.is_none_or(|(_, near, _)| d < near)
        {
            unfamiliar = Some((other, d, p));
        }
    }
    if weight > 1e-12 {
        let angle = y.atan2(x) - heading;
        let strength = x.hypot(y) / (weight * range);
        v[0] = angle.sin() * strength;
        v[1] = angle.cos() * strength;
        v[3] = sin / weight;
        v[4] = cos / weight;
    }
    v[2] = count / (count + 4.0);
    v[5] /= v[5] + 1.0;
    if let Some((other, d, p)) = unfamiliar {
        let angle = (p.y - at.y).atan2(p.x - at.x) - heading;
        v[6] = angle.sin();
        v[7] = angle.cos();
        v[8] = 1.0 - d / range;
        v[9] = (other.tissue.nutrient - observer.tissue.nutrient)
            / (other.tissue.nutrient + observer.tissue.nutrient);
        let h = world.ecology.headings[&other.id];
        v[15] = ((h.cos() * (at.x - p.x) + h.sin() * (at.y - p.y)) / d.max(1e-12)).clamp(-1.0, 1.0);
    }
    if let Some(b) = world
        .diagnostics
        .as_ref()
        .and_then(|d| d.behavior.get(&observer.id))
    {
        v[11] = b.ingested / (1.0 + b.ingested);
        v[12] = b.damage_received / (0.1 + b.damage_received);
        if let Some(attacker) = b.attacker.and_then(|id| world.organisms().get(&id)) {
            let p = world.positions[&attacker.id];
            if attacker.alive
                && world.tick().saturating_sub(b.attacker_tick) <= 120
                && distance(at, p) <= range
                && crate::terrain::visible(patches, at, p)
            {
                let angle = (p.y - at.y).atan2(p.x - at.x) - heading;
                v[16] = angle.sin();
                v[17] = angle.cos();
            }
        }
    }
    for (slot, offset) in [(13, 0.65_f64), (14, -0.65_f64)] {
        let angle = heading + offset;
        for step in 1..=12 {
            let f = step as f64 / 12.0;
            let p = Position {
                x: at.x + angle.cos() * range * f,
                y: at.y + angle.sin() * range * f,
            };
            if p.x < radius
                || p.y < radius
                || p.x > world.rules.world_width - radius
                || p.y > world.rules.world_height - radius
                || crate::development::rock_collision(patches, p, radius)
            {
                break;
            }
            v[slot] = f;
        }
        let dx = angle.cos() * range;
        let dy = angle.sin() * range;
        let length = range * range;
        for id in candidates.iter().filter(|id| **id != observer.id) {
            let other = &world.organisms()[id];
            if !other.alive {
                continue;
            }
            let p = world.positions[id];
            let r = radius
                + agents::circles(other, p, world.ecology.headings[id]).expect("audited body")[0]
                    .radius;
            let x = at.x - p.x;
            let y = at.y - p.y;
            let b = x * dx + y * dy;
            let c = x * x + y * y - r * r;
            let discriminant = b * b - length * c;
            if b < 0.0 && discriminant >= 0.0 {
                v[slot] = v[slot].min(((-b - discriminant.sqrt()) / length).clamp(0.0, 1.0));
            }
        }
    }
    v.map(|value| value.clamp(-1.0, 1.0))
}

pub fn metrics(world: &Habitat) -> serde_json::Value {
    let members: Vec<_> = world
        .organisms()
        .values()
        .filter(|o| o.alive && !o.genome.photosynthetic())
        .collect();
    let index =
        crate::spatial::SpatialIndex::new(&world.grid, &world.positions, world.organisms().iter())
            .expect("audited world");
    let mut parents: std::collections::BTreeMap<_, _> =
        members.iter().map(|o| (o.id, o.id)).collect();
    fn root(
        parents: &std::collections::BTreeMap<crate::ids::OrganismId, crate::ids::OrganismId>,
        mut id: crate::ids::OrganismId,
    ) -> crate::ids::OrganismId {
        while parents[&id] != id {
            id = parents[&id];
        }
        id
    }
    let (mut pairs, mut alignment, mut distances) = (0_u64, 0.0, 0.0);
    for o in &members {
        let at = world.positions[&o.id];
        for other in index
            .within(&world.grid, &world.positions, at, 1.5)
            .expect("local query")
        {
            if other <= o.id || !parents.contains_key(&other) {
                continue;
            }
            let p = world.positions[&other];
            if !crate::terrain::visible(
                &world.config.evolution.as_ref().expect("evolution").terrain,
                at,
                p,
            ) {
                continue;
            }
            let similar = (crate::food_web::adaptations(o).map_or(0.5, |a| a.pigment)
                - crate::food_web::adaptations(&world.organisms()[&other])
                    .map_or(0.5, |a| a.pigment))
            .abs()
                < 0.125;
            if !similar {
                continue;
            }
            pairs += 1;
            alignment += (world.ecology.headings[&o.id] - world.ecology.headings[&other]).cos();
            distances += distance(at, p);
            let a = root(&parents, o.id);
            let b = root(&parents, other);
            parents.insert(a.max(b), a.min(b));
        }
    }
    let mut sizes = std::collections::BTreeMap::new();
    for id in parents.keys() {
        *sizes.entry(root(&parents, *id)).or_insert(0usize) += 1;
    }
    serde_json::json!({"local_pairs":pairs,"alignment":alignment/(pairs.max(1) as f64),"mean_pair_distance":distances/(pairs.max(1) as f64),"groups":sizes.values().filter(|v|**v>=3).count(),"grouped_organisms":sizes.values().filter(|v|**v>=3).sum::<usize>(),"definition":"visible pigment-similar neighbours within 1.5 world units; groups have at least three members"})
}
