use crate::{
    AppModel,
    scene::{BodyTransforms, CellGlyph, CellSprites, Glyph, SceneCache},
};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use std::collections::BTreeSet;
use world_core::ids::OrganismId;

pub(crate) fn sync(
    commands: &mut Commands,
    app: &mut AppModel,
    cache: &mut SceneCache,
    materials: &mut Assets<ColorMaterial>,
    cells: &mut CellSprites,
    bodies: &mut BodyTransforms,
) {
    let model = &mut app.0;
    let Some(frame) = &model.display else {
        return;
    };
    if model.scene_dirty {
        for entity in cache.cells.drain(..).chain(cache.bounds.drain(..)) {
            commands.entity(entity).despawn();
        }
        for (_, glyph) in std::mem::take(&mut cache.bodies) {
            commands.entity(glyph.entity).despawn();
            materials.remove(glyph.material.id());
        }
        let size = Vec2::new(
            frame.width as f32 / frame.columns as f32,
            frame.height as f32 / frame.rows as f32,
        );
        for i in 0..frame.cells.len() {
            cache.cells.push(
                commands
                    .spawn((
                        CellGlyph,
                        Sprite::from_color(Color::BLACK, size),
                        Transform::from_xyz(
                            (i % frame.columns) as f32 * size.x + size.x / 2.0,
                            (i / frame.columns) as f32 * size.y + size.y / 2.0,
                            -1.0,
                        ),
                        RenderLayers::layer(1),
                    ))
                    .id(),
            );
        }
        model.scene_dirty = false;
        cache.revision = None;
    }
    let revision = (
        model.revision,
        model.show_fields,
        model.show_light,
        model.show_detritus,
    );
    if cache.revision != Some(revision) {
        for (i, entity) in cache.cells.iter().enumerate() {
            if let Ok((mut sprite, mut visible)) = cells.get_mut(*entity) {
                *visible = if model.show_fields {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                let value = frame.cells[i];
                let level = (if model.show_detritus {
                    value[1] / 1.2
                } else if model.show_light {
                    value[2] / 0.8
                } else {
                    value[0] / 10.0
                })
                .clamp(0.0, 1.0) as f32;
                sprite.color = if model.show_detritus {
                    Color::srgb(
                        0.06 + level * 0.65,
                        0.04 + level * 0.24,
                        0.03 + level * 0.12,
                    )
                } else {
                    Color::srgb(
                        0.035 + value[1].min(3.0) as f32 * 0.06,
                        0.05 + level * 0.18,
                        0.08 + level * 0.12,
                    )
                };
            }
        }
        let mut live = BTreeSet::new();
        sync_terrain(commands, cache, materials, bodies, &mut live, frame);
        for corpse in &frame.corpses {
            let key = (corpse.source, u64::MAX - 2);
            live.insert(key);
            let transform =
                Transform::from_xyz(corpse.position.x as f32, corpse.position.y as f32, 1.5)
                    .with_scale(Vec3::splat(corpse.radius().max(0.025) as f32));
            let color = Color::srgb(0.48, 0.49, 0.50);
            if let Some(glyph) = cache.bodies.get(&key) {
                if let Ok((mut t, mut v)) = bodies.get_mut(glyph.entity) {
                    *t = transform;
                    *v = Visibility::Inherited;
                }
            } else {
                let material = materials.add(color);
                let entity = commands
                    .spawn((
                        crate::scene::BodyGlyph,
                        Mesh2d(cache.mesh.clone()),
                        MeshMaterial2d(material.clone()),
                        transform,
                        RenderLayers::layer(1),
                    ))
                    .id();
                cache.bodies.insert(key, Glyph { entity, material });
            }
        }
        for o in frame.organisms.iter().filter(|o| o.alive) {
            for kind in [0, 1, 2] {
                let marker = kind == 1;
                let injury = kind == 2;
                if (marker && o.neurons == 0) || (injury && o.integrity >= 0.999) {
                    continue;
                }
                let key = (
                    OrganismId(o.id),
                    if marker {
                        u64::MAX
                    } else if injury {
                        u64::MAX - 1
                    } else {
                        1
                    },
                );
                live.insert(key);
                let radius = if marker {
                    o.radius * 0.12
                } else if injury {
                    o.radius * 1.06
                } else {
                    o.radius
                };
                let offset = if marker { o.radius * 0.75 } else { 0.0 };
                let transform = Transform::from_xyz(
                    (o.position.x + o.heading.cos() * offset) as f32,
                    (o.position.y + o.heading.sin() * offset) as f32,
                    if marker || injury { 3.0 } else { 2.0 },
                )
                .with_scale(Vec3::splat(radius.max(0.001) as f32));
                let color = if marker {
                    Color::srgb(0.95, 0.97, 0.94)
                } else if injury {
                    Color::srgba(0.96, 0.32, 0.28, (0.35 + 0.65 * (1.0 - o.integrity)) as f32)
                } else if o.profile["guild"].as_str() == Some("predator") {
                    Color::srgb(0.83, 0.43, 0.25)
                } else if o.profile["guild"].as_str() == Some("scavenger") {
                    Color::srgb(0.63, 0.48, 0.75)
                } else if o.profile["guild"].as_str() == Some("recycler") {
                    Color::srgb(0.63, 0.61, 0.35)
                } else if o.producer {
                    Color::srgb(0.22, 0.68, 0.43)
                } else {
                    Color::srgb(0.24, 0.63, 0.86)
                };
                if let Some(glyph) = cache.bodies.get(&key) {
                    if let Ok((mut t, mut v)) = bodies.get_mut(glyph.entity) {
                        *t = transform;
                        *v = Visibility::Inherited;
                    }
                    if let Some(mut material) = materials.get_mut(&glyph.material) {
                        material.color = color;
                    }
                } else {
                    let material = materials.add(color);
                    let entity = commands
                        .spawn((
                            crate::scene::BodyGlyph,
                            Mesh2d(if injury {
                                cache.injury_mesh.clone()
                            } else {
                                cache.mesh.clone()
                            }),
                            MeshMaterial2d(material.clone()),
                            transform,
                            RenderLayers::layer(1),
                        ))
                        .id();
                    cache.bodies.insert(key, Glyph { entity, material });
                }
            }
        }
        let removed: Vec<_> = cache
            .bodies
            .keys()
            .filter(|k| !live.contains(k))
            .copied()
            .collect();
        for key in removed {
            if let Some(g) = cache.bodies.remove(&key) {
                commands.entity(g.entity).despawn();
                materials.remove(g.material.id());
            }
        }
        cache.revision = Some(revision);
    }
    let selection = model.selected.and_then(|id| {
        frame
            .organisms
            .iter()
            .find(|o| o.id == id.0 && o.alive)
            .map(|o| (o.position, o.radius))
            .or_else(|| {
                frame
                    .corpses
                    .iter()
                    .find(|c| c.source == id)
                    .map(|c| (c.position, c.radius().max(0.025)))
            })
    });
    if let Some((position, radius)) = selection {
        commands.entity(cache.ring).insert((
            Transform::from_xyz(position.x as f32, position.y as f32, 1.0)
                .with_scale(Vec3::splat((radius + 0.06) as f32)),
            Visibility::Inherited,
        ));
    } else {
        commands.entity(cache.ring).insert(Visibility::Hidden);
    }
}

fn sync_terrain(
    commands: &mut Commands,
    cache: &mut SceneCache,
    materials: &mut Assets<ColorMaterial>,
    bodies: &mut BodyTransforms,
    live: &mut BTreeSet<(OrganismId, u64)>,
    frame: &world_core::display::DisplayFrame,
) {
    let Some(patches) = frame.ecosystem["terrain"].as_array() else {
        return;
    };
    for (index, patch) in patches.iter().enumerate() {
        let key = (OrganismId(0), index as u64 + 1);
        live.insert(key);
        let transform = Transform::from_xyz(
            patch["center"]["x"].as_f64().unwrap_or(0.0) as f32,
            patch["center"]["y"].as_f64().unwrap_or(0.0) as f32,
            -0.5,
        )
        .with_scale(Vec3::splat(patch["radius"].as_f64().unwrap_or(0.0) as f32));
        let color = match patch["kind"].as_str().unwrap_or("") {
            "rock" => Color::srgba(0.22, 0.25, 0.25, 0.9),
            "water" => Color::srgba(0.035, 0.17, 0.29, 0.7),
            "vegetation" => Color::srgba(0.045, 0.19, 0.09, 0.5),
            _ => Color::srgba(0.04, 0.045, 0.08, 0.55),
        };
        if let Some(glyph) = cache.bodies.get(&key) {
            if let Ok((mut t, mut v)) = bodies.get_mut(glyph.entity) {
                *t = transform;
                *v = Visibility::Inherited;
            }
            if let Some(mut material) = materials.get_mut(&glyph.material) {
                material.color = color;
            }
        } else {
            let material = materials.add(color);
            let entity = commands
                .spawn((
                    crate::scene::BodyGlyph,
                    Mesh2d(cache.mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    transform,
                    RenderLayers::layer(1),
                ))
                .id();
            cache.bodies.insert(key, Glyph { entity, material });
        }
    }
}
