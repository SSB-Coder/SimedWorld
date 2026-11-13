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

#[derive(Component)]
pub struct WorldCamera;
#[derive(Component)]
pub struct UiElement;
#[derive(Component)]
pub struct CellGlyph;
#[derive(Component)]
pub struct BodyGlyph;
#[derive(Component)]
pub struct Panel;
#[derive(Component)]
pub struct Heading;
#[derive(Component)]
pub struct Subtitle;
#[derive(Component)]
pub struct Legend;
#[derive(Component)]
pub struct Background;

pub(crate) struct Glyph {
    pub(crate) entity: Entity,
    pub(crate) material: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct SceneCache {
    pub(crate) revision: Option<(u64, bool, bool, bool)>,
    pub(crate) cells: Vec<Entity>,
    pub(crate) bodies: BTreeMap<(OrganismId, u64), Glyph>,
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) injury_mesh: Handle<Mesh>,
    pub(crate) ring: Entity,
    pub(crate) bounds: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct ViewArea {
    pub ui_width: f32,
    pub world_height: f32,
    pub aspect: f32,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 840.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(0),
    ));
    commands.spawn((
        Camera2d,
        WorldCamera,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.025, 0.055, 0.065)),
            ..default()
        },
        RenderLayers::layer(1),
    ));
    commands.spawn((
        UiElement,
        Background,
        Sprite::from_color(Color::srgb(0.055, 0.075, 0.10), Vec2::new(320.0, 680.0)),
        Transform::from_xyz(440.0, -15.0, -2.0),
        RenderLayers::layer(0),
    ));
    for (marker, value, size) in [
        (0, "WORLD / evolving life", 28.0),
        (
            1,
            "Green capture | gold intake | cyan sensing | blue propulsion | red injury",
            14.0,
        ),
        (2, "Loading habitat...", 14.0),
        (
            3,
            "SPACE pause | N decision/tick | +/- speed | F5 save | F9 load | L field\nWASD/arrows/right-drag pan | wheel zoom | HOME fit | F follow | G fields | B brain | F11 fullscreen",
            13.0,
        ),
    ] {
        let mut entity = commands.spawn((
            UiElement,
            Text2d::new(value),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.91, 0.94)),
            Anchor::TOP_LEFT,
            Transform::from_xyz(-600.0, 380.0, 5.0),
            RenderLayers::layer(0),
        ));
        match marker {
            0 => {
                entity.insert(Heading);
            }
            1 => {
                entity.insert(Subtitle);
            }
            2 => {
                entity.insert((Panel, TextLayout::no_wrap()));
            }
            _ => {
                entity.insert(Legend);
            }
        }
    }
    let mesh = meshes.add(Circle::new(1.0));
    let ring = commands
        .spawn((
            Mesh2d(meshes.add(Annulus::new(0.96, 1.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.86, 0.35))),
            Transform::default(),
            Visibility::Hidden,
            RenderLayers::layer(1),
        ))
        .id();
    commands.insert_resource(SceneCache {
        revision: None,
        cells: Vec::new(),
        bodies: BTreeMap::new(),
        mesh,
        injury_mesh: meshes.add(Annulus::new(0.94, 1.0)),
        ring,
        bounds: Vec::new(),
    });
    commands.insert_resource(ViewArea::default());
}

pub fn caption(app: Res<AppModel>, mut texts: Query<&mut Text2d, With<Subtitle>>) {
    let value = if app.0.display.is_some() {
        "Green producers | blue consumers | white dot facing | red injury | funded body size"
    } else {
        "Green capture | gold intake | cyan sensing | blue propulsion | red injury"
    };
    for mut text in &mut texts {
        if text.0 != value {
            text.0 = value.into();
        }
    }
}

type UiQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
        Option<&'static Heading>,
        Option<&'static Subtitle>,
        Option<&'static Panel>,
        Option<&'static Legend>,
        Option<&'static Background>,
    ),
    (With<UiElement>, Without<WorldCamera>),
>;
pub(crate) type CellSprites<'w, 's> = Query<
    'w,
    's,
    (&'static mut Sprite, &'static mut Visibility),
    (With<CellGlyph>, Without<BodyGlyph>),
>;
pub(crate) type BodyTransforms<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<BodyGlyph>, Without<CellGlyph>),
>;

pub fn layout(
    window: Single<&Window>,
    mut app: ResMut<AppModel>,
    mut area: ResMut<ViewArea>,
    mut camera: Single<(&mut Camera, &mut Projection, &mut Transform), With<WorldCamera>>,
    mut ui: UiQuery<'_, '_>,
) {
    let model = &mut app.0;
    if model.follow
        && let Some(id) = model.selected
        && let Some(position) = model.position(id)
    {
        model.center = position;
    }
    let ui_width = window.width() / window.height().max(1.0) * 840.0;
    let canvas_width = (ui_width - 380.0).max(100.0);
    let position = UVec2::new(
        (20.0 / ui_width * window.physical_width() as f32) as u32,
        (110.0 / 840.0 * window.physical_height() as f32) as u32,
    );
    let size = UVec2::new(
        (canvas_width / ui_width * window.physical_width() as f32).max(1.0) as u32,
        (660.0 / 840.0 * window.physical_height() as f32).max(1.0) as u32,
    );
    camera.0.viewport = if model.display.is_some() {
        None
    } else {
        Some(Viewport {
            physical_position: position,
            physical_size: size,
            ..default()
        })
    };
    let (width, height, _, _) = model.dimensions();
    let aspect = if model.display.is_some() {
        window.width() / window.height().max(1.0)
    } else {
        size.x as f32 / size.y.max(1) as f32
    };
    let world_height = (height as f32).max(width as f32 / aspect) / model.zoom;
    if let Projection::Orthographic(projection) = &mut *camera.1 {
        projection.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: world_height,
        };
    }
    camera.2.translation = Vec3::new(model.center.x as f32, model.center.y as f32, 0.0);
    area.ui_width = ui_width;
    area.world_height = world_height;
    area.aspect = aspect;
    for (mut transform, mut visibility, heading, subtitle, panel, legend, background) in &mut ui {
        *visibility = if model.display.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if heading.is_some() {
            transform.translation = Vec3::new(-ui_width * 0.5 + 20.0, 395.0, 5.0);
        }
        if subtitle.is_some() {
            transform.translation = Vec3::new(-ui_width * 0.5 + 20.0, 350.0, 5.0);
        }
        if panel.is_some() {
            transform.translation = Vec3::new(ui_width * 0.5 - 330.0, 310.0, 5.0);
        }
        if legend.is_some() {
            transform.translation = Vec3::new(-ui_width * 0.5 + 20.0, -375.0, 5.0);
        }
        if background.is_some() {
            transform.translation = Vec3::new(ui_width * 0.5 - 180.0, -15.0, -2.0);
        }
    }
}

fn colour(kind: ModuleKind, producer: bool, predator: bool, integrity: f64) -> Color {
    let (r, g, b) = match kind {
        ModuleKind::Core => {
            if producer {
                (0.25, 0.72, 0.43)
            } else if predator {
                (1.0, 0.42, 0.24)
            } else {
                (0.30, 0.78, 0.95)
            }
        }
        ModuleKind::Photosynthetic => (0.15, 0.94, 0.43),
        ModuleKind::Mouth => (0.98, 0.74, 0.28),
        ModuleKind::Sensor => (0.35, 0.95, 0.96),
        ModuleKind::Propulsion => (0.30, 0.45, 0.95),
        ModuleKind::Armor => (0.64, 0.69, 0.75),
        ModuleKind::Storage => (0.77, 0.42, 0.90),
    };
    let health = integrity as f32;
    Color::srgb(r * health + 0.8 * (1.0 - health), g * health, b * health)
}

pub fn sync(
    mut commands: Commands,
    mut app: ResMut<AppModel>,
    mut cache: ResMut<SceneCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cells: CellSprites<'_, '_>,
    mut bodies: BodyTransforms<'_, '_>,
) {
    if app.0.display.is_some() {
        crate::circle_scene::sync(
            &mut commands,
            &mut app,
            &mut cache,
            &mut materials,
            &mut cells,
            &mut bodies,
        );
        return;
    }
    if app.0.scene_dirty {
        for entity in cache.bounds.drain(..) {
            commands.entity(entity).despawn();
        }
        for entity in cache.cells.drain(..) {
            commands.entity(entity).despawn();
        }
        for (_, glyph) in std::mem::take(&mut cache.bodies) {
            commands.entity(glyph.entity).despawn();
            materials.remove(glyph.material.id());
        }
        let grid = app.0.habitat.grid();
        let (width, height, columns, rows) = grid.dimensions();
        let size = Vec2::new(width as f32 / columns as f32, height as f32 / rows as f32);
        for (center, size) in [
            (
                Vec2::new(width as f32 * 0.5, 0.0),
                Vec2::new(width as f32, 0.035),
            ),
            (
                Vec2::new(width as f32 * 0.5, height as f32),
                Vec2::new(width as f32, 0.035),
            ),
            (
                Vec2::new(0.0, height as f32 * 0.5),
                Vec2::new(0.035, height as f32),
            ),
            (
                Vec2::new(width as f32, height as f32 * 0.5),
                Vec2::new(0.035, height as f32),
            ),
        ] {
            let entity = commands
                .spawn((
                    Sprite::from_color(Color::srgb(0.14, 0.34, 0.38), size),
                    Transform::from_xyz(center.x, center.y, 0.0),
                    RenderLayers::layer(1),
                ))
                .id();
            cache.bounds.push(entity);
        }
        for index in 0..grid.cells().len() {
            let entity = commands
                .spawn((
                    CellGlyph,
                    Sprite::from_color(Color::BLACK, size),
                    Transform::from_xyz(
                        (index % columns) as f32 * size.x + size.x * 0.5,
                        (index / columns) as f32 * size.y + size.y * 0.5,
                        -1.0,
                    ),
                    RenderLayers::layer(1),
                ))
                .id();
            cache.cells.push(entity);
        }
        app.0.scene_dirty = false;
    }
    for (index, entity) in cache.cells.iter().enumerate() {
        let cell = &app.0.habitat.grid().cells()[index];
        if let Ok((mut sprite, mut visible)) = cells.get_mut(*entity) {
            *visible = if app.0.show_fields {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            let level = if app.0.show_detritus {
                cell.detritus().nutrient / 1.2
            } else if app.0.show_light {
                (cell.light_available() / 0.8).clamp(0.0, 1.0)
            } else {
                (cell.mineral_nutrient() / 6.0).clamp(0.0, 1.0)
            } as f32;
            let waste = (cell.detritus().nutrient / 4.0).clamp(0.0, 1.0) as f32;
            sprite.color = if app.0.show_detritus {
                Color::srgb(
                    0.06 + level * 0.65,
                    0.04 + level * 0.24,
                    0.03 + level * 0.12,
                )
            } else if app.0.show_light {
                Color::srgb(
                    0.08 + level * 0.25,
                    0.11 + level * 0.20,
                    0.15 + level * 0.04,
                )
            } else {
                Color::srgb(
                    0.04 + waste * 0.27,
                    0.08 + level * 0.18,
                    0.10 + level * 0.12,
                )
            };
        }
    }
    let mut live = BTreeSet::new();
    let world = &app.0.habitat;
    for (id, o) in world.organisms().iter().filter(|(_, o)| o.alive) {
        let position = world.positions()[id];
        let heading = world.heading(*id).unwrap_or(0.0);
        if let Ok(shapes) = agents::circles(o, position, heading) {
            for shape in shapes {
                let key = (*id, shape.id);
                live.insert(key);
                let radius = if o.body.is_none() {
                    (o.tissue.nutrient.sqrt() as f32 * 0.07).clamp(0.08, 0.5)
                } else {
                    shape.radius as f32
                };
                let transform =
                    Transform::from_xyz(shape.center.x as f32, shape.center.y as f32, 2.0)
                        .with_scale(Vec3::splat(radius.max(0.001)));
                let predator = o
                    .genome
                    .body
                    .as_ref()
                    .is_some_and(|b| b.diet != world_genetics::body::Diet::Grazer);
                let color = colour(
                    shape.kind,
                    o.genome.photosynthetic(),
                    predator,
                    shape.integrity,
                );
                let visible = if radius > 0.001 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if let Some(glyph) = cache.bodies.get(&key) {
                    if let Ok((mut body, mut visibility)) = bodies.get_mut(glyph.entity) {
                        *body = transform;
                        *visibility = visible;
                    }
                    if let Some(mut material) = materials.get_mut(&glyph.material) {
                        material.color = color;
                    }
                } else {
                    let material = materials.add(color);
                    let entity = commands
                        .spawn((
                            BodyGlyph,
                            Mesh2d(cache.mesh.clone()),
                            MeshMaterial2d(material.clone()),
                            transform,
                            visible,
                            RenderLayers::layer(1),
                        ))
                        .id();
                    cache.bodies.insert(key, Glyph { entity, material });
                }
            }
        }
    }
    let removed: Vec<_> = cache
        .bodies
        .keys()
        .filter(|key| !live.contains(key))
        .copied()
        .collect();
    for key in removed {
        if let Some(glyph) = cache.bodies.remove(&key) {
            commands.entity(glyph.entity).despawn();
            materials.remove(glyph.material.id());
        }
    }
    if let Some(id) = app.0.selected
        && world.organisms().get(&id).is_some_and(|o| o.alive)
    {
        let o = &world.organisms()[&id];
        let position = world.positions()[&id];
        let radius = agents::circles(o, position, world.heading(id).unwrap_or(0.0))
            .unwrap_or_default()
            .iter()
            .map(|s| (s.center.x - position.x).hypot(s.center.y - position.y) + s.radius)
            .fold(0.2_f64, f64::max) as f32;
        commands.entity(cache.ring).insert((
            Transform::from_xyz(position.x as f32, position.y as f32, 1.0)
                .with_scale(Vec3::splat(radius + 0.04)),
            Visibility::Inherited,
        ));
    } else {
        commands.entity(cache.ring).insert(Visibility::Hidden);
    }
}

fn rectangle(gizmos: &mut Gizmos, low: Vec2, high: Vec2, color: Color) {
    for (a, b) in [
        (low, Vec2::new(high.x, low.y)),
        (Vec2::new(high.x, low.y), high),
        (high, Vec2::new(low.x, high.y)),
        (Vec2::new(low.x, high.y), low),
    ] {
        gizmos.line_2d(a, b, color);
    }
}

pub fn panel(
    app: Res<AppModel>,
    area: Res<ViewArea>,
    mut panels: Query<(&mut Text2d, &mut TextBounds), With<Panel>>,
    mut gizmos: Gizmos,
) {
    if app.0.display.is_some() {
        return;
    }
    let model = &app.0;
    let world = &model.habitat;
    let metrics = world.metrics();
    let summary = &model.summary;
    let status_chars: Vec<_> = model.status.chars().take(60).collect();
    let status = status_chars
        .chunks(30)
        .map(|line| line.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    let speed = if let Some(actual) = model.actual_speed {
        format!(
            "{} | target {}x\nActual {:.2}x | zoom {:.1}x",
            if model.paused { "PAUSED" } else { "RUNNING" },
            model.speed,
            actual,
            model.zoom
        )
    } else {
        format!(
            "{} | {}x | zoom {:.1}x",
            if model.paused { "PAUSED" } else { "RUNNING" },
            model.speed,
            model.zoom
        )
    };
    let mut content = format!(
        "{}\nTime {:.2} T / tick {}\n\nProducers {} / Consumers {}\nBirths {} / Deaths {}\nFeeding events {}\nNutrient {:.3} N\nCaptured light {:.2} E\nRecycled {:.2} N\n\n{}\n\n",
        speed,
        world.tick() as f64 * world.rules().physics_dt,
        world.tick(),
        summary["producer_count"],
        summary["consumer_count"],
        metrics.births,
        metrics.deaths,
        metrics.feeding_events,
        summary["total_nutrient"].as_f64().unwrap_or(0.0),
        summary["absorbed_light"].as_f64().unwrap_or(0.0),
        metrics.decomposed_nutrient,
        status
    );
    content = format!(
        "{}\n{}",
        if model.observer.is_some() {
            "NEURAL ECOSYSTEM"
        } else {
            "SCRIPTED DIAGNOSTIC"
        },
        content
    );
    if let Some(id) = model.selected
        && let Some(o) = world.organisms().get(&id)
    {
        content += &format!(
            "ID {} / {} / age {:.2} T\nTissue {:.3} N\nReserves {:.3} N / {:.3} E\nParent {}\n",
            id.0,
            if o.alive { "alive" } else { "dead" },
            o.age,
            o.tissue.nutrient,
            o.reserves.nutrient,
            o.reserves.energy,
            o.parent_id.map_or("founder".into(), |p| p.0.to_string())
        );
        if let Some(body) = &o.body {
            content += &format!(
                "Modules {} / injury {:.3} N\n",
                body.modules.len(),
                body.missing()
            );
            if !model.show_neural
                && let Some(genome) = &o.genome.body
            {
                content += &format!("Diet {:?}\n", genome.diet);
                for gene in genome.modules.iter().take(4) {
                    content += &format!(
                        "{:?}: {:.0}% integrity\n",
                        gene.kind,
                        body.integrity(gene.id) * 100.0
                    );
                }
            }
        }
        if let Some(brain) = &o.genome.brain {
            content += &format!(
                "Neurons {} / Synapses {}\nTopology {} / B: graph\n",
                brain.nodes.len(),
                brain.connections.len(),
                brain.version
            );
        } else {
            content += "No neural controller\n";
        }
        if let Some(diagnostics) = world.diagnostics()
            && let Some(event) = diagnostics
                .events
                .iter()
                .rev()
                .find(|e| e.kind == "damage" && e.target == Some(id))
        {
            content += &format!("Last hit: ID {} / {:.3} N\n", event.actor.0, event.amount);
        }
        if model.show_neural
            && let Some(brain) = &o.genome.brain
        {
            content += "Graph: up to 48 nodes/type\n512 edges; colour = activity\n";
            let left = area.ui_width * 0.5 - 320.0;
            let mut locations = BTreeMap::new();
            for (kind, x) in [
                (NodeKind::Input, left),
                (NodeKind::Hidden, left + 130.0),
                (NodeKind::Output, left + 270.0),
            ] {
                let nodes: Vec<_> = brain
                    .nodes
                    .iter()
                    .filter(|n| n.kind == kind)
                    .take(48)
                    .collect();
                for (index, n) in nodes.iter().enumerate() {
                    locations.insert(
                        n.id,
                        Vec2::new(x, -120.0 - index as f32 * 85.0 / nodes.len().max(1) as f32),
                    );
                }
            }
            for edge in brain.connections.iter().take(512) {
                if let (Some(a), Some(b)) =
                    (locations.get(&edge.source), locations.get(&edge.target))
                {
                    gizmos.line_2d(
                        *a,
                        *b,
                        if edge.weight >= 0.0 {
                            Color::srgba(0.2, 0.55, 0.65, 0.25)
                        } else {
                            Color::srgba(0.75, 0.35, 0.25, 0.25)
                        },
                    );
                }
            }
            for (id_node, p) in locations {
                let activation = model
                    .observer
                    .as_ref()
                    .and_then(|s| s.memory.get(&(id.0, id_node)))
                    .copied()
                    .unwrap_or(0.0);
                gizmos.circle_2d(
                    p,
                    2.0 + activation.abs() as f32 * 2.0,
                    if activation >= 0.0 {
                        Color::srgb(0.95, 0.68, 0.3)
                    } else {
                        Color::srgb(0.3, 0.8, 0.95)
                    },
                );
            }
        }
    } else {
        content += "Click an organism to inspect.\nF follows / B opens brain.\n\nGreen: photosynthetic tissue\nBlue: grazer core\nOrange: predator core\nRed: injured tissue\nG toggles resource cells.\n";
    }
    for (mut text, mut bounds) in &mut panels {
        if text.0 != content {
            text.0 = content.clone();
        }
        if bounds.width != Some(300.0) || bounds.height.is_some() {
            *bounds = TextBounds {
                width: Some(300.0),
                height: None,
            };
        }
    }
    let (width, height, _, _) = world.grid().dimensions();
    let low = Vec2::new(area.ui_width * 0.5 - 320.0, -330.0);
    let size = Vec2::new(280.0, 110.0);
    rectangle(&mut gizmos, low, low + size, Color::srgb(0.25, 0.42, 0.5));
    for (id, o) in world.organisms().iter().filter(|(_, o)| o.alive) {
        let p = world.positions()[id];
        let point = low + Vec2::new(p.x as f32 / width as f32, p.y as f32 / height as f32) * size;
        gizmos.circle_2d(
            point,
            1.3,
            if o.genome.photosynthetic() {
                Color::srgb(0.25, 0.9, 0.5)
            } else {
                Color::srgb(1.0, 0.55, 0.25)
            },
        );
    }
    let center = low
        + Vec2::new(
            model.center.x as f32 / width as f32,
            model.center.y as f32 / height as f32,
        ) * size;
    let half = Vec2::new(
        area.world_height * area.aspect / width as f32,
        area.world_height / height as f32,
    ) * size
        * 0.5;
    rectangle(
        &mut gizmos,
        (center - half).max(low),
        (center + half).min(low + size),
        Color::srgb(0.95, 0.85, 0.4),
    );
}
