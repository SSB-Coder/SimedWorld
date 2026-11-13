use std::{num::NonZeroU8, path::PathBuf};

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    winit::WinitSettings,
};
use world_core::habitat::Habitat;

mod circle_scene;
mod ecosystem_scene;
mod hud;
mod model;
mod observer;
mod options;
mod scene;

use model::{Action, ViewerModel};
use options::Options;

#[derive(Resource)]
pub struct AppModel(pub ViewerModel);

#[derive(Resource)]
struct Runtime {
    stop_at: Option<u64>,
    capture: Option<PathBuf>,
    frames: u64,
    requested: bool,
    failed: bool,
    save_waiting: bool,
}

fn main() {
    if std::env::args().any(|arg| arg == "--help") {
        println!(
            "World viewer\n--rules PATH --config PATH --load PATH --checkpoint PATH --paused\n--run-for-ticks N --capture PATH.bmp --observe PATH --commands DIR\nWASD/right-drag: pan | wheel: zoom | Home: fit | F: follow | G: fields | B: brain\nSpace: pause | N: one tick | +/-: speed | F5: save | F9: load | L: field | Click: inspect"
        );
        return;
    }
    let result = start();
    if let Err(error) = result {
        eprintln!("World viewer: {error}");
        std::process::exit(1);
    }
}

fn start() -> Result<(), String> {
    let options = Options::parse(std::env::args().skip(1))?;
    let mut remote = None;
    let habitat = if let Some(path) = &options.observe {
        let commands = options
            .commands
            .clone()
            .ok_or("observer command path is required")?;
        let (client, observed) =
            observer::Observer::open(path.clone(), commands).map_err(|e| e.to_string())?;
        remote = Some((
            client,
            observed.paused,
            observed.speed,
            observed.status,
            observed.actual_speed,
            observed.display,
        ));
        observed.habitat.unwrap_or(
            Habitat::from_json(
                include_str!("../../../configs/ecosystem_rules.json"),
                include_str!("../../../configs/ecosystem_habitat.json"),
            )
            .map_err(|e| e.to_string())?,
        )
    } else if let Some(path) = &options.load {
        Habitat::load(path).map_err(|e| e.to_string())?
    } else {
        let rules = match &options.rules {
            Some(path) => std::fs::read_to_string(path).map_err(|e| e.to_string())?,
            None => include_str!("../../../configs/ecosystem_rules.json").into(),
        };
        let config = match &options.config {
            Some(path) => std::fs::read_to_string(path).map_err(|e| e.to_string())?,
            None => include_str!("../../../configs/ecosystem_habitat.json").into(),
        };
        Habitat::from_json(&rules, &config).map_err(|e| e.to_string())?
    };
    let stop_at = options
        .run_for_ticks
        .map(|n| {
            remote
                .as_ref()
                .and_then(|r| r.5.as_ref().map(|d| d.tick))
                .unwrap_or_else(|| habitat.tick())
                .checked_add(n)
                .ok_or("run length overflows tick counter")
        })
        .transpose()?;
    let timestep = habitat.rules().physics_dt;
    if habitat.diagnostics().is_some() && remote.is_none() {
        return Err(
            "evolving worlds run through examples/evolution_demo.py; use --observe for the viewer"
                .into(),
        );
    }
    let mut model = ViewerModel::new(habitat, options.checkpoint, options.paused);
    model.zoom = options.zoom;
    model.selected = options.select.map(world_core::ids::OrganismId);
    model.show_neural = options.brain;
    model.show_fields = options.fields;
    model.show_history = options.history;
    model.follow = model.selected.is_some();
    if let Some((mut observer, paused, speed, status, actual_speed, display)) = remote {
        if model.selected.is_some() {
            observer
                .send(&Action::Select(model.selected), &model.checkpoint)
                .map_err(|e| e.to_string())?;
        }
        model.observer = Some(observer);
        model.paused = paused;
        model.speed = speed;
        model.status = status;
        model.actual_speed = actual_speed;
        model.display = display;
        if model.display.is_some() {
            model.fit();
            model.zoom = options.zoom;
            model.follow = model.selected.is_some();
        }
    }
    let capture_mode = options.capture.is_some();
    let outcome = App::new()
        .insert_resource(AppModel(model))
        .insert_resource(Runtime {
            stop_at,
            capture: options.capture,
            frames: 0,
            requested: false,
            failed: false,
            save_waiting: false,
        })
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.065)))
        .insert_resource(Time::<Fixed>::from_hz(1.0 / timestep))
        .insert_resource(WinitSettings::continuous())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "World - survival microcosm".into(),
                resolution: (1280, 720).into(),
                resizable: true,
                focused: !capture_mode && std::env::var_os("WORLD_VIEWER_BACKGROUND").is_none(),
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 900.0,
                    min_height: 600.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .init_gizmo_group::<ecosystem_scene::WorldOverlay>()
        .add_systems(Startup, ecosystem_scene::setup)
        .add_systems(Startup, scene::setup)
        .add_systems(Startup, hud::setup)
        .add_systems(FixedUpdate, advance)
        .add_systems(
            Update,
            (
                refresh,
                scene::caption,
                input,
                navigation,
                scene::layout,
                scene::sync,
                scene::panel,
                hud::update,
                ecosystem_scene::draw,
                finish,
            )
                .chain(),
        )
        .run();
    if matches!(outcome, AppExit::Error(_)) {
        return Err("viewer stopped after an error".into());
    }
    Ok(())
}

fn keyboard_actions(keys: &ButtonInput<KeyCode>) -> Vec<Action> {
    let mut actions = Vec::new();
    for (key, action) in [
        (KeyCode::Space, Action::TogglePause),
        (KeyCode::KeyN, Action::Advance),
        (KeyCode::Equal, Action::Faster),
        (KeyCode::NumpadAdd, Action::Faster),
        (KeyCode::Minus, Action::Slower),
        (KeyCode::NumpadSubtract, Action::Slower),
        (KeyCode::F5, Action::Save),
        (KeyCode::F9, Action::Load),
        (KeyCode::KeyE, Action::Export),
        (KeyCode::KeyL, Action::ToggleLayer),
    ] {
        if keys.just_pressed(key) {
            actions.push(action);
        }
    }
    actions
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<scene::WorldCamera>>,
    mut window: Single<&mut Window>,
    mut app: ResMut<AppModel>,
    mut fixed: ResMut<Time<Fixed>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        window.mode = if matches!(window.mode, bevy::window::WindowMode::Windowed) {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        } else {
            bevy::window::WindowMode::Windowed
        };
    }
    if keys.just_pressed(KeyCode::Home) {
        app.0.fit();
    }
    if keys.just_pressed(KeyCode::KeyF) {
        app.0.follow = !app.0.follow;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        app.0.show_fields = !app.0.show_fields;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        app.0.show_neural = !app.0.show_neural;
        app.0.show_history = false;
    }
    if keys.just_pressed(KeyCode::KeyH) {
        app.0.show_help = !app.0.show_help;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        app.0.show_history = !app.0.show_history;
        app.0.show_neural = false;
    }
    for action in keyboard_actions(&keys) {
        if let Err(error) = app.0.apply(action) {
            app.0.status = error.to_string();
            app.0.paused = true;
        }
    }
    fixed.set_timestep_seconds(app.0.habitat.rules().physics_dt);
    if mouse.just_pressed(MouseButton::Left)
        && let Some(cursor) = window.cursor_position()
        && !hud::blocks_cursor(&app.0, Vec2::new(window.width(), window.height()), cursor)
        && let Ok(point) = camera.0.viewport_to_world_2d(camera.1, cursor)
        && camera
            .0
            .logical_viewport_rect()
            .is_some_and(|r| r.contains(cursor))
    {
        let nearest = if let Some(display) = &app.0.display {
            display
                .organisms
                .iter()
                .filter(|o| o.alive)
                .map(|o| {
                    (
                        world_core::ids::OrganismId(o.id),
                        (o.position.x - point.x as f64).hypot(o.position.y - point.y as f64),
                        o.radius,
                    )
                })
                .chain(display.corpses.iter().map(|corpse| {
                    (
                        corpse.source,
                        (corpse.position.x - point.x as f64)
                            .hypot(corpse.position.y - point.y as f64),
                        corpse.radius().max(0.025),
                    )
                }))
                .filter(|(_, d, radius)| *d <= *radius + 0.04)
                .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
                .map(|(id, _, _)| id)
        } else {
            app.0
                .habitat
                .organisms()
                .iter()
                .filter(|(_, o)| o.alive)
                .map(|(id, _)| {
                    let other = app.0.habitat.positions()[id];
                    (
                        *id,
                        (other.x - point.x as f64).hypot(other.y - point.y as f64),
                    )
                })
                .filter(|(_, distance)| *distance < 0.8)
                .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
                .map(|(id, _)| id)
        };
        let _ = app.0.apply(Action::Select(nearest));
    }
}

fn refresh(mut app: ResMut<AppModel>) {
    if let Err(error) = app.0.refresh() {
        app.0.status = format!("Observer update rejected: {error}");
    }
}

fn navigation(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    area: Res<scene::ViewArea>,
    mut app: ResMut<AppModel>,
    time: Res<Time>,
) {
    let model = &mut app.0;
    let over_world = window.cursor_position().is_some_and(|p| {
        if model.display.is_some() {
            !hud::blocks_cursor(model, Vec2::new(window.width(), window.height()), p)
        } else {
            p.x < window.width() - 350.0 && p.y > 80.0 && p.y < window.height() - 65.0
        }
    });
    for event in wheel.read() {
        if over_world {
            let unit = if matches!(event.unit, bevy::input::mouse::MouseScrollUnit::Pixel) {
                0.002
            } else {
                0.14
            };
            model.zoom = (model.zoom * (event.y * unit).exp()).clamp(0.25, 64.0);
        }
    }
    let horizontal = (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as i8
        - (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as i8;
    let vertical = (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) as i8
        - (keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) as i8;
    if horizontal != 0 || vertical != 0 {
        model.follow = false;
        model.center.x +=
            horizontal as f64 * area.world_height as f64 * time.delta_secs() as f64 * 0.6;
        model.center.y +=
            vertical as f64 * area.world_height as f64 * time.delta_secs() as f64 * 0.6;
    }
    if let Some(cursor) = window.cursor_position() {
        if mouse.pressed(MouseButton::Right)
            && over_world
            && let Some(previous) = model.previous_cursor
        {
            model.follow = false;
            model.center.x -= (cursor.x - previous.0) as f64 * area.world_height as f64
                / (window.height()
                    * if model.display.is_some() {
                        1.0
                    } else {
                        660.0 / 840.0
                    }) as f64;
            model.center.y += (cursor.y - previous.1) as f64 * area.world_height as f64
                / (window.height()
                    * if model.display.is_some() {
                        1.0
                    } else {
                        660.0 / 840.0
                    }) as f64;
        }
        model.previous_cursor = Some((cursor.x, cursor.y));
    } else {
        model.previous_cursor = None;
    }
}

fn advance(mut app: ResMut<AppModel>, mut runtime: ResMut<Runtime>) {
    if let Err(error) = app.0.advance_frame(runtime.stop_at) {
        app.0.status = error.to_string();
        app.0.paused = true;
        error!("Habitat update rejected: {error}");
        if runtime.stop_at.is_some() {
            runtime.failed = true;
        }
    }
}

fn finish(
    mut commands: Commands,
    mut runtime: ResMut<Runtime>,
    mut app: ResMut<AppModel>,
    mut exit: MessageWriter<AppExit>,
) {
    runtime.frames += 1;
    if runtime.failed {
        exit.write(AppExit::Error(NonZeroU8::new(1).unwrap()));
        return;
    }
    if runtime.requested || runtime.frames < 60 || runtime.stop_at.is_none_or(|n| app.0.tick() < n)
    {
        return;
    }
    if app.0.display.is_some() {
        if !runtime.save_waiting {
            if let Err(error) = app.0.apply(Action::Save) {
                error!("Owner save failed: {error}");
                runtime.failed = true;
            }
            runtime.save_waiting = true;
            return;
        }
        if app
            .0
            .observer
            .as_ref()
            .is_some_and(|o| o.save_request.is_none() || o.save_request != o.acknowledged)
        {
            return;
        }
    }
    runtime.requested = true;
    app.0.paused = true;
    let save = if app.0.display.is_some() {
        Ok(())
    } else if let Some(observer) = &app.0.observer {
        observer.save_published(&app.0.checkpoint)
    } else {
        app.0.habitat.save(&app.0.checkpoint)
    };
    if let Err(error) = save {
        error!("Final checkpoint failed: {error}");
        exit.write(AppExit::Error(NonZeroU8::new(1).unwrap()));
        return;
    }
    if let Some(path) = &runtime.capture {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let path = path.clone();
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| e.to_string())
                    .and_then(|image| image.save(&path).map_err(|e| e.to_string()));
                match result {
                    Ok(()) => {
                        exit.write(AppExit::Success);
                    }
                    Err(error) => {
                        error!("Capture failed: {error}");
                        exit.write(AppExit::Error(NonZeroU8::new(1).unwrap()));
                    }
                }
            },
        );
    } else {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_routes_pause_tick_save_and_load() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        keys.press(KeyCode::KeyN);
        keys.press(KeyCode::F5);
        keys.press(KeyCode::F9);
        let actions = keyboard_actions(&keys);
        assert_eq!(actions.len(), 4);
        assert!(matches!(actions[0], Action::TogglePause));
        assert!(matches!(actions[1], Action::Advance));
        assert!(matches!(actions[2], Action::Save));
        assert!(matches!(actions[3], Action::Load));
        keys.clear();
        keys.press(KeyCode::NumpadAdd);
        assert!(matches!(
            keyboard_actions(&keys).as_slice(),
            [Action::Faster]
        ));
        keys.clear();
        keys.press(KeyCode::NumpadSubtract);
        assert!(matches!(
            keyboard_actions(&keys).as_slice(),
            [Action::Slower]
        ));
    }
}
