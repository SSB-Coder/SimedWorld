use crate::{AppModel, model::ViewerModel, scene::ViewArea};
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    sprite::Anchor,
    text::{FontSize, TextBounds},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Component)]
enum Part {
    Stats,
    Selection,
    Brain,
    BrainFooter,
    Help,
    Hint,
    History,
}

#[derive(Component)]
pub(crate) struct Overlay {
    part: Part,
    background: bool,
}

type OverlayQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Overlay,
        &'static mut Transform,
        &'static mut Visibility,
        Option<&'static mut Sprite>,
        Option<&'static mut Text2d>,
        Option<&'static mut TextBounds>,
    ),
>;

fn area(part: Part, width: f32) -> Rect {
    let narrow = width < 760.0;
    let panel_width = (width - 32.0).min(340.0);
    match part {
        Part::Stats => Rect::from_corners(Vec2::splat(16.0), Vec2::new(16.0 + panel_width, 210.0)),
        Part::Selection => {
            Rect::from_corners(Vec2::new(16.0, 660.0), Vec2::new(16.0 + panel_width, 786.0))
        }
        Part::Brain => {
            if narrow {
                Rect::from_corners(Vec2::new(16.0, 226.0), Vec2::new(width - 16.0, 786.0))
            } else {
                Rect::from_corners(
                    Vec2::new(width - 388.0, 16.0),
                    Vec2::new(width - 16.0, 664.0),
                )
            }
        }
        Part::BrainFooter => {
            let rect = area(Part::Brain, width);
            Rect::from_corners(Vec2::new(rect.min.x, rect.max.y - 54.0), rect.max)
        }
        Part::Help => {
            Rect::from_corners(Vec2::new(16.0, 550.0), Vec2::new(16.0 + panel_width, 786.0))
        }
        Part::History => {
            Rect::from_corners(Vec2::new(16.0, 226.0), Vec2::new(16.0 + panel_width, 642.0))
        }
        Part::Hint => Rect::from_corners(
            Vec2::new(16.0, 798.0),
            Vec2::new((width - 16.0).min(650.0), 828.0),
        ),
    }
}

fn visible(part: Part, model: &ViewerModel) -> bool {
    model.display.is_some()
        && match part {
            Part::Stats | Part::Hint => true,
            Part::Selection => model.selected.is_some() && !model.show_neural && !model.show_help,
            Part::Brain | Part::BrainFooter => model.show_neural,
            Part::Help => model.show_help,
            Part::History => model.show_history && !model.show_help,
        }
}

pub(crate) fn blocks_cursor(model: &ViewerModel, size: Vec2, cursor: Vec2) -> bool {
    if model.display.is_none() {
        return false;
    }
    let scale = 840.0 / size.y.max(1.0);
    let p = cursor * scale;
    [
        Part::Stats,
        Part::Selection,
        Part::Brain,
        Part::BrainFooter,
        Part::Help,
        Part::Hint,
        Part::History,
    ]
    .iter()
    .any(|part| visible(*part, model) && area(*part, size.x * scale).contains(p))
}

pub(crate) fn setup(mut commands: Commands) {
    for part in [
        Part::Stats,
        Part::Selection,
        Part::Brain,
        Part::BrainFooter,
        Part::Help,
        Part::Hint,
        Part::History,
    ] {
        commands.spawn((
            Overlay {
                part,
                background: true,
            },
            Sprite::from_color(Color::srgba(0.035, 0.055, 0.075, 0.92), Vec2::ONE),
            Transform::default(),
            Visibility::Hidden,
            RenderLayers::layer(0),
        ));
        commands.spawn((
            Overlay {
                part,
                background: false,
            },
            Text2d::new(""),
            TextFont {
                font_size: FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.84, 0.91, 0.92)),
            TextLayout::no_wrap(),
            TextBounds::default(),
            Anchor::TOP_LEFT,
            Transform::default(),
            Visibility::Hidden,
            RenderLayers::layer(0),
        ));
    }
}

fn text(part: Part, model: &ViewerModel) -> String {
    let Some(frame) = &model.display else {
        return String::new();
    };
    let selected = model
        .selected
        .and_then(|id| frame.organisms.iter().find(|o| o.id == id.0));
    if matches!(part, Part::Selection | Part::Brain)
        && let Some(corpse) = model
            .selected
            .and_then(|id| frame.corpses.iter().find(|c| c.source == id))
    {
        return format!(
            "CORPSE #{}\nRemaining energy {:.2} E\nRemaining nutrient {:.2} N\nDecay age {:.1} T\nCause: {:?}\n\nGrey bodies are edible remains.\nRecycling releases local minerals.",
            corpse.source.0,
            corpse.remaining.energy,
            corpse.remaining.nutrient,
            corpse.age,
            corpse.cause
        );
    }
    match part {
        Part::Stats => {
            let health = &frame.summary["health"];
            let birth = health["roles"]["producer"]["last_birth_tick"].as_u64();
            let history = birth.map_or("No producer births recorded".into(), |tick|format!("Last producer birth {:.1} T ago", (frame.tick.saturating_sub(tick)) as f64*frame.physics_dt));
            format!("WORLD  /  {}\nTick {}   Time {:.1} T\nSpeed {}x requested / {:.2}x actual\n\nProducers {}   Consumers {}\nBirths {}   Deaths {}\nMinerals {:.1} N   Detritus {:.1} N\n{}\n{}",
                if model.paused {"PAUSED"} else {"LIVE"},frame.tick,frame.model_time,model.speed,model.actual_speed.unwrap_or(0.0),
                frame.summary["producer_count"],frame.summary["consumer_count"],frame.summary["ecology"]["births"],frame.summary["ecology"]["deaths"],
                health["resources"]["mineral_nutrient"].as_f64().unwrap_or(0.0),health["resources"]["detritus_nutrient"].as_f64().unwrap_or(0.0),history,
                if frame.summary["consumer_count"].as_u64()==Some(0) {"Consumers extinct; run retained"} else {"G resources   L mineral/light/detritus"})
        },
        Part::Selection => selected.map_or(String::new(), |o|format!("{} #{}  /  {}\nEnergy {:.2} E   Nutrient {:.2} N\nIntegrity {:.0}%   Age {:.1} T\nBars: red integrity / cyan energy\n{}",
            o.profile["guild"].as_str().unwrap_or(if o.producer {"Producer"} else {"Consumer"}),o.id,if o.alive {"alive"} else {"dead"},o.energy,o.nutrient,o.integrity*100.0,o.age,model.status.chars().take(44).collect::<String>())),
        Part::Brain => {
            let Some(o)=selected else {return "NEURAL INSPECTOR          [B] close\n\nSelect an organism to inspect its brain.\n\nThe world continues outside this panel.".into();};
            let b=&frame.brain;
            let mut content=format!("NEURAL INSPECTOR          [B] close\n{} #{}  /  {}\nAge {:.1} T   Integrity {:.0}%\nEnergy {:.2} E   Nutrient {:.2} N\n\nNeurons {}   Connections {}\nReward {:+.3}   Movement {}\n",
                o.profile["guild"].as_str().unwrap_or(if o.producer {"Producer"} else {"Consumer"}),o.id,if o.alive {"alive"} else {"dead"},o.age,o.integrity*100.0,o.energy,o.nutrient,o.neurons,o.connections,o.reward,if o.blocked {"blocked"} else {"clear"});
            if b["id"].as_u64()==Some(o.id) {
                let inputs=b["nodes"].as_array().map_or(15,|nodes|nodes.iter().filter(|n|n["kind"]=="input").count());
                let parameters=o.connections+o.neurons.saturating_sub(inputs);
                content+=&format!("Trainable parameters {}\nUpdates {}   Change {:.3}\n",parameters,b["updates"],b["parameter_change"].as_f64().unwrap_or(0.0));
                if let Some(nodes)=b["nodes"].as_array() {
                    let output=|channel|nodes.iter().find(|n|n["channel"].as_u64()==Some(channel)).and_then(|n|n["value"].as_f64()).unwrap_or(0.0);
                    content+=&format!("\nNEURAL OUTPUTS  thrust {:.2}  turn {:+.2}\nFeed activation {:.2}\n",output(110).max(0.0),output(111),output(120).max(0.0));
                    if o.profile["adaptations"]["agency"].as_bool()==Some(true) {content+=&format!("Grow {:.2}  Repair {:.2}  Reproduce {:.2}\nAttack {:.2}  Trap {:.2}  Signal {:.2}\n",output(100).max(0.0),output(101).max(0.0),output(103).max(0.0),output(121).max(0.0),output(106).max(0.0),output(108).max(0.0));}
                    content+=&format!("\nINPUTS           MEMORY            OUTPUTS\n\nShown {} / {} nodes, {} / {} edges",nodes.len(),o.neurons,b["edges"].as_array().map_or(0,|e|e.len()),o.connections);
                }
            } else { content+="\nNo active neural controller.\n"; }
            content
        },
        Part::BrainFooter => "Hover a node for its name and activity.\nLinks: cyan positive / rust negative".into(),
        Part::Help => "CONTROLS\nSpace pause   N one decision\n+/- speed     F5 save   F9 load\nClick select  B neural inspector\nWASD / arrows / right drag pan\nWheel zoom    Home fit   F follow\nG fields      L field    F11 fullscreen\nGreen producer / Blue grazer\nOrange predator / Purple scavenger\nOlive recycler / Gold filament\nRed rim injury / White heading".into(),
        Part::History => {
            let mut content=if selected.is_some() {"ACTION TRACE                  [V]\nRequests are neural motor outputs\n\n"} else {"FOOD WEB / RECENT EVENTS      [V]\nTransfers measured in nutrient N\n\n"}.to_string();
            if selected.is_none() {
            if let Some(guilds)=frame.summary["health"]["guilds"].as_object() {
                for (name,count) in guilds {content+=&format!("{name}: {count}\n");}
                content+="\n\n";
            }
            if let Some(flows)=frame.ecosystem["food_web"]["flows"].as_object() {
                for (name,amount) in flows.iter().take(8) {content+=&format!("{name}: {:.2}\n",amount.as_f64().unwrap_or(0.0));}
            }
            }
            if let Some(o)=selected {
                content+=&format!("\nSelected #{}  Parent {:?}\n",o.id,o.parent);
                let behavior=&o.profile["behavior"];
                if let Some(death)=o.profile["death_history"].as_object() {
                    content+=&format!("Died: {}\n",death["cause"].as_str().unwrap_or("unknown"));
                    if let Some(signals)=death["contributing_signals"].as_array() {
                        for signal in signals.iter().take(2) {content+=&format!("{}\n",signal.as_str().unwrap_or("").replace('_'," "));}
                    }
                }
                if !behavior.is_null() {
                    content+=&format!("Idle {:.3} E/T  Food retained {:.3} E\nTissue fuel {:.3} E  Shortfall {:.3} E\n",behavior["upkeep_per_time"].as_f64().unwrap_or(0.0),behavior["retained_food_energy"].as_f64().unwrap_or(0.0),behavior["catabolic_energy"].as_f64().unwrap_or(0.0),behavior["maintenance_shortfall"].as_f64().unwrap_or(0.0));
                    content+=&format!("Energy {:.2} -> {:.2}\nMoved {:.3}  Fed {:.3}  Damage {:.3}\n",behavior["energy_before"].as_f64().unwrap_or(0.0),behavior["energy_after"].as_f64().unwrap_or(0.0),behavior["executed"]["110"].as_f64().unwrap_or(0.0),behavior["executed"]["120"].as_f64().unwrap_or(0.0),behavior["executed"]["121"].as_f64().unwrap_or(0.0));
                    content+=&format!("Requested move {:.2} turn {:+.2}\nRequested feed {:.2} attack {:.2}\n",behavior["requested"]["110"].as_f64().unwrap_or(0.0),behavior["requested"]["111"].as_f64().unwrap_or(0.0),behavior["requested"]["120"].as_f64().unwrap_or(0.0),behavior["requested"]["121"].as_f64().unwrap_or(0.0));
                    if o.producer {content+=&format!("Grazer-edible tissue {:.3} N\n",o.profile["edible_tissue"].as_f64().unwrap_or(0.0));}
                    let label=|value:&serde_json::Value|value.as_u64().map_or_else(||"none".to_string(),|id|format!("#{id}"));
                    content+=&format!("Target {}  Attacker {}\nInjury received {:.3}\n",label(&behavior["target"]),label(&behavior["attacker"]),behavior["damage_received"].as_f64().unwrap_or(0.0));
                    if let Some(failures)=behavior["failures"].as_object() {for (reason,_) in failures.iter().take(3) {
                        let message=match reason.as_str() {
                            "attack_no_contact"=>"No target in strike range",
                            "attack_energy"=>"Not enough energy to strike",
                            "feed_no_contact"=>"No edible contact",
                            "feed_no_detritus"=>"No local detritus",
                            "digestion_energy"=>"Not enough energy to digest",
                            "feed_capacity"=>"Food reserves are full",
                            "feed_capacity_or_energy"=>"Food reserves full or digestion energy low",
                            "controller_unfunded"=>"Requested meal did not fund controller",
                            other=>other,
                        };
                        content+=&format!("Action blocked: {message}\n");
                    }}
                }
                if let Some(events)=o.profile["events"].as_array() {
                    for event in events.iter().take(4) {content+=&format!("{}: {}\n",event["tick"],event["kind"].as_str().unwrap_or("event"));}
                }
                if let Some(history)=o.profile["energy_history"].as_array() {
                    content+="Sampled actions (move / feed)\n";
                    for sample in history.iter().rev().take(2) {
                        let b=&sample["behavior"];
                        content+=&format!("Tick {}: {:.3} / {:.3}\n",sample["tick"],b["executed"]["110"].as_f64().unwrap_or(0.0),b["executed"]["120"].as_f64().unwrap_or(0.0));
                    }
                }
            }
            content
        },
        Part::Hint => "SPACE pause  +/- speed  B brain  V web  E export  H help".into(),
    }
}

pub(crate) fn update(
    window: Single<&Window>,
    app: Res<AppModel>,
    area_state: Res<ViewArea>,
    mut overlays: OverlayQuery<'_, '_>,
    mut gizmos: Gizmos,
) {
    let width = area_state.ui_width;
    for (element, mut transform, mut visibility, sprite, value, bounds) in &mut overlays {
        let show = visible(element.part, &app.0);
        let next_visibility = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next_visibility {
            *visibility = next_visibility;
        }
        if !show {
            continue;
        }
        let rect = area(element.part, width);
        if element.background {
            let translation =
                Vec3::new(rect.center().x - width / 2.0, 420.0 - rect.center().y, 8.0);
            if transform.translation != translation {
                transform.translation = translation;
            }
            if let Some(mut sprite) = sprite
                && sprite.custom_size != Some(rect.size())
            {
                sprite.custom_size = Some(rect.size());
            }
        } else {
            let translation = Vec3::new(
                rect.min.x + 12.0 - width / 2.0,
                420.0 - rect.min.y - 10.0,
                10.0,
            );
            if transform.translation != translation {
                transform.translation = translation;
            }
            if let Some(mut value) = value {
                let content = if matches!(element.part, Part::BrainFooter) {
                    hover_description(
                        &app.0,
                        width,
                        window
                            .cursor_position()
                            .map(|p| p * 840.0 / window.height().max(1.0)),
                    )
                } else {
                    text(element.part, &app.0)
                };
                if value.0 != content {
                    value.0 = content;
                }
            }
            if let Some(mut bounds) = bounds
                && (bounds.width != Some(rect.width() - 24.0)
                    || bounds.height != Some(rect.height() - 16.0))
            {
                *bounds = TextBounds {
                    width: Some(rect.width() - 24.0),
                    height: Some(rect.height() - 16.0),
                };
            }
        }
    }
    if app.0.display.is_some() && app.0.show_neural {
        let rect = area(Part::Brain, width);
        if let Some(frame) = &app.0.display {
            draw_brain(&frame.brain, rect, width, &mut gizmos);
        }
    }
    if visible(Part::Selection, &app.0)
        && let Some(frame) = &app.0.display
        && let Some(o) = frame
            .organisms
            .iter()
            .find(|o| Some(o.id) == app.0.selected.map(|id| id.0))
    {
        let rect = area(Part::Selection, width);
        let capacity = o.profile["energy_capacity"]
            .as_f64()
            .unwrap_or(o.energy.max(1.0));
        let length = (rect.width() - 36.0) * 0.5;
        for (i, fraction, color) in [
            (0, o.integrity, Color::srgb(0.95, 0.35, 0.3)),
            (
                1,
                (o.energy / capacity.max(1e-12)).clamp(0.0, 1.0),
                Color::srgb(0.3, 0.8, 0.9),
            ),
        ] {
            let start = Vec2::new(
                rect.min.x + 12.0 + i as f32 * (length + 12.0) - width / 2.0,
                420.0 - rect.max.y + 12.0,
            );
            gizmos.line_2d(
                start,
                start + Vec2::X * length,
                Color::srgb(0.23, 0.27, 0.3),
            );
            gizmos.line_2d(start, start + Vec2::X * length * fraction as f32, color);
        }
    }
}

fn draw_brain(brain: &serde_json::Value, rect: Rect, width: f32, gizmos: &mut Gizmos) {
    let Some(nodes) = brain["nodes"].as_array() else {
        return;
    };
    let positions = brain_positions(brain, rect, width);
    if let Some(edges) = brain["edges"].as_array() {
        for edge in edges {
            if let (Some(a), Some(b)) = (
                edge["source"].as_u64().and_then(|id| positions.get(&id)),
                edge["target"].as_u64().and_then(|id| positions.get(&id)),
            ) {
                let weight = edge["weight"].as_f64().unwrap_or(0.0);
                let alpha = (0.15 + weight.abs().min(3.0) * 0.2) as f32;
                let color = if weight >= 0.0 {
                    Color::srgba(0.32, 0.68, 0.75, alpha)
                } else {
                    Color::srgba(0.84, 0.47, 0.32, alpha)
                };
                if a == b {
                    gizmos.circle_2d(*a + Vec2::new(6.0, 0.0), 6.0, color);
                } else {
                    gizmos.line_2d(*a, *b, color);
                }
            }
        }
    }
    for node in nodes {
        if let Some(p) = node["id"].as_u64().and_then(|id| positions.get(&id)) {
            let v = node["value"].as_f64().unwrap_or(0.0);
            gizmos.circle_2d(
                *p,
                2.5 + v.abs() as f32 * 2.0,
                if v >= 0.0 {
                    Color::srgb(0.98, 0.72, 0.34)
                } else {
                    Color::srgb(0.36, 0.78, 0.95)
                },
            );
        }
    }
}

fn brain_positions(brain: &serde_json::Value, rect: Rect, width: f32) -> BTreeMap<u64, Vec2> {
    let Some(nodes) = brain["nodes"].as_array() else {
        return BTreeMap::new();
    };
    let mut positions = BTreeMap::new();
    let top = 420.0 - rect.min.y - 300.0;
    let height = (rect.height() - 360.0).max(100.0);
    let left = rect.min.x + 26.0 - width / 2.0;
    let span = rect.width() - 52.0;
    for (kind, fraction) in [("input", 0.0), ("hidden", 0.5), ("output", 1.0)] {
        let group: Vec<_> = nodes.iter().filter(|n| n["kind"] == kind).collect();
        for (i, n) in group.iter().enumerate() {
            if let Some(id) = n["id"].as_u64() {
                positions.insert(
                    id,
                    Vec2::new(
                        left + fraction * span,
                        top - height * (i as f32 + 0.5) / group.len().max(1) as f32,
                    ),
                );
            }
        }
    }
    positions
}

fn hover_description(model: &ViewerModel, width: f32, cursor: Option<Vec2>) -> String {
    let default = "Hover a node for its name and activity.\nLinks: cyan positive / rust negative";
    let Some(frame) = &model.display else {
        return default.into();
    };
    let Some(cursor) = cursor else {
        return default.into();
    };
    let cursor = Vec2::new(cursor.x - width / 2.0, 420.0 - cursor.y);
    let positions = brain_positions(&frame.brain, area(Part::Brain, width), width);
    let Some((id, _)) = positions
        .iter()
        .filter(|(_, p)| p.distance(cursor) < 9.0)
        .min_by(|(_, a), (_, b)| a.distance(cursor).total_cmp(&b.distance(cursor)))
    else {
        return link_description(&frame.brain, &positions, cursor)
            .unwrap_or_else(|| default.into());
    };
    let Some(node) = frame.brain["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|n| n["id"].as_u64() == Some(*id)))
    else {
        return default.into();
    };
    let name = match node["channel"].as_u64() {
        Some(0) => "Energy reserve",
        Some(1) => "Nutrient reserve",
        Some(2) => "Development",
        Some(3) => "Injury",
        Some(4) => "Maturity",
        Some(10) => "Food bearing sine",
        Some(11) => "Food bearing cosine",
        Some(12) => "Food proximity",
        Some(13) => "Harvestable food size",
        Some(14) => "Threat bearing sine",
        Some(15) => "Threat bearing cosine",
        Some(16) => "Threat proximity",
        Some(17) => "Actual movement",
        Some(19) => "Blocked movement",
        Some(20) => "Forward wall",
        Some(5) => "Local light",
        Some(6) => "Local minerals",
        Some(21) => "Local cover",
        Some(22) => "Local detritus",
        Some(23) => "Entanglement",
        Some(24) => "Local signal strength",
        Some(25) => "Signal bearing sine",
        Some(26) => "Signal bearing cosine",
        Some(27) => "Age",
        Some(30) => "Similar-neighbour cohesion sine",
        Some(31) => "Similar-neighbour cohesion cosine",
        Some(32) => "Visible-neighbour density",
        Some(33) => "Neighbour alignment sine",
        Some(34) => "Neighbour alignment cosine",
        Some(35) => "Local crowding",
        Some(36) => "Threat bearing sine",
        Some(37) => "Threat bearing cosine",
        Some(38) => "Threat proximity",
        Some(39) => "Relative threat size",
        Some(40) => "Edible contact",
        Some(41) => "Recent intake",
        Some(42) => "Recent damage",
        Some(43) => "Left clearance",
        Some(44) => "Right clearance",
        Some(45) => "Threat approach direction",
        Some(46) => "Attacker bearing sine",
        Some(47) => "Attacker bearing cosine",
        Some(48) => "Alternative food bearing sine",
        Some(49) => "Alternative food bearing cosine",
        Some(50) => "Alternative food proximity",
        Some(51) => "Alternative harvestable size",
        Some(52) => "Energy runway for searching",
        Some(53) => "Estimated food energy after travel",
        Some(54) => "Food energy yield after digestion",
        Some(55) => "Estimated travel affordability",
        Some(100) => "Growth allocation",
        Some(101) => "Repair allocation",
        Some(103) => "Reproductive attempt",
        Some(104) => "Neural growth",
        Some(105) => "Neural pruning",
        Some(106) => "Deploy filament",
        Some(107) => "Retract filament",
        Some(108) => "Emit local signal",
        Some(109) => "Burst locomotion",
        Some(121) => "Attack",
        Some(122) => "Break filament restraint",
        Some(123) => "Concealment posture",
        Some(124) => "Prefer profitable local food",
        Some(110) => "Thrust",
        Some(111) => "Turn",
        Some(120) => "Feed",
        _ => "Recurrent memory",
    };
    format!(
        "Neuron #{}: {}\nActivity {:+.4}",
        id,
        name,
        node["value"].as_f64().unwrap_or(0.0)
    )
}

fn link_description(
    brain: &serde_json::Value,
    positions: &BTreeMap<u64, Vec2>,
    cursor: Vec2,
) -> Option<String> {
    let edges = brain["edges"].as_array()?;
    let (edge, _) = edges
        .iter()
        .filter_map(|edge| {
            let a = *positions.get(&edge["source"].as_u64()?)?;
            let b = *positions.get(&edge["target"].as_u64()?)?;
            let distance = if a == b {
                (cursor.distance(a + Vec2::new(6.0, 0.0)) - 6.0).abs()
            } else {
                let fraction = ((cursor - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
                cursor.distance(a + (b - a) * fraction)
            };
            (distance < 3.0).then_some((edge, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    Some(format!(
        "Connection {} -> {}\nWeight {:+.5}",
        edge["source"],
        edge["target"],
        edge["weight"].as_f64()?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlays_fit_and_reflow_without_overlapping_stats() {
        for width in [500.0, 760.0, 1280.0, 2000.0] {
            let stats = area(Part::Stats, width);
            let brain = area(Part::Brain, width);
            assert!(stats.min.x >= 0.0 && brain.max.x <= width && brain.max.y <= 840.0);
            assert!(brain.min.x >= stats.max.x || brain.min.y >= stats.max.y);
        }
    }
}
