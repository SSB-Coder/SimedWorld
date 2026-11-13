use crate::AppModel;
use bevy::{
    camera::visibility::RenderLayers,
    gizmos::config::{GizmoConfigGroup, GizmoConfigStore},
    prelude::*,
};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(crate) struct WorldOverlay;

pub(crate) fn setup(mut store: ResMut<GizmoConfigStore>) {
    store.config_mut::<WorldOverlay>().0.render_layers = RenderLayers::layer(1);
}

fn position(value: &serde_json::Value) -> Vec2 {
    Vec2::new(
        value["x"].as_f64().unwrap_or(0.0) as f32,
        value["y"].as_f64().unwrap_or(0.0) as f32,
    )
}

pub(crate) fn draw(app: Res<AppModel>, mut gizmos: Gizmos<WorldOverlay>) {
    let Some(frame) = &app.0.display else {
        return;
    };
    if let Some(terrain) = frame.ecosystem["terrain"].as_array() {
        for patch in terrain {
            let color = match patch["kind"].as_str().unwrap_or("") {
                "rock" => Color::srgb(0.55, 0.57, 0.58),
                "water" => Color::srgb(0.15, 0.42, 0.68),
                "vegetation" => Color::srgb(0.16, 0.40, 0.24),
                _ => Color::srgb(0.27, 0.28, 0.42),
            };
            gizmos.circle_2d(
                position(&patch["center"]),
                patch["radius"].as_f64().unwrap_or(0.0) as f32,
                color,
            );
        }
    }
    if let Some(traps) = frame.ecosystem["filaments"].as_array() {
        for trap in traps {
            let color = if trap["captured"].is_null() {
                Color::srgb(0.83, 0.78, 0.51)
            } else {
                Color::srgb(0.95, 0.38, 0.22)
            };
            gizmos.line_2d(position(&trap["anchor"]), position(&trap["end"]), color);
        }
    }
    if app.0.show_history
        && let Some(selected) = frame
            .organisms
            .iter()
            .find(|o| Some(world_core::ids::OrganismId(o.id)) == app.0.selected)
    {
        for (field, color) in [
            ("target", Color::srgb(0.95, 0.55, 0.15)),
            ("attacker", Color::srgb(0.95, 0.2, 0.25)),
        ] {
            if let Some(target) = selected.profile["behavior"][field]
                .as_u64()
                .and_then(|id| frame.organisms.iter().find(|o| o.id == id && o.alive))
            {
                gizmos.line_2d(
                    Vec2::new(selected.position.x as f32, selected.position.y as f32),
                    Vec2::new(target.position.x as f32, target.position.y as f32),
                    color,
                );
            }
        }
    }
    if let Some(signals) = frame.ecosystem["signals"].as_array() {
        for signal in signals.iter().filter(|s| {
            app.0
                .selected
                .is_some_and(|id| s["owner"].as_u64() == Some(id.0))
        }) {
            gizmos.circle_2d(
                position(&signal["position"]),
                signal["range"].as_f64().unwrap_or(0.0) as f32,
                Color::srgba(0.6, 0.4, 0.95, 0.4),
            );
        }
    }
}
