use std::path::PathBuf;

use world_core::{habitat::Habitat, ids::OrganismId, resources::Result};

pub enum Action {
    TogglePause,
    Advance,
    Faster,
    Slower,
    Save,
    Load,
    Export,
    ToggleLayer,
    Select(Option<OrganismId>),
}

pub struct ViewerModel {
    pub habitat: Habitat,
    pub display: Option<world_core::display::DisplayFrame>,
    pub revision: u64,
    pub paused: bool,
    pub speed: u32,
    pub actual_speed: Option<f64>,
    pub selected: Option<OrganismId>,
    pub show_light: bool,
    pub show_detritus: bool,
    pub checkpoint: PathBuf,
    pub status: String,
    pub scene_dirty: bool,
    pub observer: Option<crate::observer::Observer>,
    pub center: world_core::environment::Position,
    pub zoom: f32,
    pub follow: bool,
    pub show_fields: bool,
    pub show_neural: bool,
    pub show_help: bool,
    pub show_history: bool,
    pub previous_cursor: Option<(f32, f32)>,
    pub summary: serde_json::Value,
}

impl ViewerModel {
    pub fn new(habitat: Habitat, checkpoint: PathBuf, paused: bool) -> Self {
        let (width, height, _, _) = habitat.grid().dimensions();
        let summary =
            serde_json::from_str(&habitat.summary_json().unwrap_or_default()).unwrap_or_default();
        Self {
            habitat,
            display: None,
            revision: 0,
            paused,
            speed: 1,
            actual_speed: None,
            selected: None,
            show_light: false,
            show_detritus: false,
            checkpoint,
            status: "Click an organism to inspect it".into(),
            scene_dirty: true,
            observer: None,
            center: world_core::environment::Position {
                x: width * 0.5,
                y: height * 0.5,
            },
            zoom: 1.0,
            follow: false,
            show_fields: false,
            show_neural: false,
            show_help: false,
            show_history: false,
            previous_cursor: None,
            summary,
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<()> {
        if let Some(observer) = &mut self.observer
            && !matches!(action, Action::ToggleLayer | Action::Select(_))
        {
            observer.send(&action, &self.checkpoint)?;
            self.status = "Command sent to biological runtime".into();
            return Ok(());
        }
        match action {
            Action::TogglePause => self.paused = !self.paused,
            Action::Advance => {
                self.paused = true;
                self.habitat.advance()?;
                self.update_summary()?;
            }
            Action::Faster => self.speed = (self.speed * 2).min(16),
            Action::Slower => self.speed = (self.speed / 2).max(1),
            Action::Save => {
                self.habitat.save(&self.checkpoint)?;
                self.status = format!("Saved at tick {}", self.habitat.tick());
            }
            Action::Load => {
                let habitat = Habitat::load(&self.checkpoint)?;
                self.habitat = habitat;
                self.update_summary()?;
                self.paused = true;
                self.scene_dirty = true;
                if self
                    .selected
                    .is_some_and(|id| !self.habitat.organisms().contains_key(&id))
                {
                    self.selected = None;
                }
                self.status = format!("Loaded tick {} (paused)", self.habitat.tick());
            }
            Action::ToggleLayer => {
                if self.show_light {
                    self.show_light = false;
                    self.show_detritus = true;
                } else if self.show_detritus {
                    self.show_detritus = false;
                } else {
                    self.show_light = true;
                }
                self.show_fields = true;
            }
            Action::Export => {
                self.status = "Subject export requires the neural runtime".into();
            }
            Action::Select(id) => {
                self.selected = id;
                if let Some(observer) = &mut self.observer {
                    observer.send(&Action::Select(id), &self.checkpoint)?;
                }
            }
        }
        Ok(())
    }

    pub fn advance_frame(&mut self, stop_at: Option<u64>) -> Result<()> {
        if self.observer.is_some() {
            return Ok(());
        }
        if self.paused {
            return Ok(());
        }
        for _ in 0..self.speed {
            if stop_at.is_some_and(|tick| self.habitat.tick() >= tick) {
                break;
            }
            self.habitat.advance()?;
        }
        self.update_summary()?;
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(observer) = &mut self.observer
            && let Some(update) = observer.poll()?
        {
            let old_dimensions = self.dimensions();
            if let Some(habitat) = update.habitat {
                self.habitat = habitat;
            }
            self.display = update.display;
            self.revision = self.revision.wrapping_add(1);
            self.scene_dirty |= old_dimensions != self.dimensions();
            self.paused = update.paused;
            self.speed = update.speed;
            self.actual_speed = update.actual_speed;
            self.status = update.status;
            self.update_summary()?;
            if self.selected.is_some_and(|id| self.position(id).is_none()) {
                self.selected = None;
            }
        }
        Ok(())
    }

    pub fn fit(&mut self) {
        let (width, height, _, _) = self.dimensions();
        self.center = world_core::environment::Position {
            x: width * 0.5,
            y: height * 0.5,
        };
        self.zoom = 1.0;
        self.follow = false;
    }
    fn update_summary(&mut self) -> Result<()> {
        if let Some(display) = &self.display {
            self.summary = display.summary.clone();
            return Ok(());
        }
        self.summary = serde_json::from_str(&self.habitat.summary_json()?)
            .map_err(|e| world_core::resources::AccountingError::Invalid(e.to_string()))?;
        Ok(())
    }

    pub fn dimensions(&self) -> (f64, f64, usize, usize) {
        self.display.as_ref().map_or_else(
            || self.habitat.grid().dimensions(),
            |d| (d.width, d.height, d.columns, d.rows),
        )
    }

    pub fn tick(&self) -> u64 {
        self.display
            .as_ref()
            .map_or_else(|| self.habitat.tick(), |d| d.tick)
    }

    pub fn position(&self, id: OrganismId) -> Option<world_core::environment::Position> {
        if let Some(display) = &self.display {
            display
                .organisms
                .iter()
                .find(|o| o.id == id.0)
                .map(|o| o.position)
                .or_else(|| {
                    display
                        .corpses
                        .iter()
                        .find(|c| c.source == id)
                        .map(|c| c.position)
                })
        } else {
            self.habitat.positions().get(&id).copied()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> ViewerModel {
        let habitat = Habitat::from_json(
            include_str!("../../../configs/ecosystem_rules.json"),
            include_str!("../../../configs/ecosystem_habitat.json"),
        )
        .unwrap();
        ViewerModel::new(
            habitat,
            std::env::temp_dir().join(format!("world-viewer-controls-{}.json", std::process::id())),
            false,
        )
    }

    #[test]
    fn pause_tick_speed_and_layer_controls_preserve_native_time() {
        let mut model = model();
        model.apply(Action::TogglePause).unwrap();
        model.advance_frame(None).unwrap();
        assert_eq!(model.habitat.tick(), 0);
        model.apply(Action::Advance).unwrap();
        assert_eq!(model.habitat.tick(), 1);
        assert!(model.paused);
        model.apply(Action::Faster).unwrap();
        model.apply(Action::TogglePause).unwrap();
        model.advance_frame(None).unwrap();
        assert_eq!(model.habitat.tick(), 3);
        model.apply(Action::Slower).unwrap();
        assert_eq!(model.speed, 1);
        model.apply(Action::ToggleLayer).unwrap();
        assert!(model.show_light);
        model.apply(Action::ToggleLayer).unwrap();
        assert!(model.show_detritus && !model.show_light);
        model.apply(Action::ToggleLayer).unwrap();
        assert!(!model.show_detritus && !model.show_light);
        model.apply(Action::Select(Some(OrganismId(1)))).unwrap();
        assert_eq!(model.selected, Some(OrganismId(1)));
    }

    #[test]
    fn save_load_restores_the_world_and_failed_load_preserves_it() {
        let mut model = model();
        for _ in 0..12 {
            model.advance_frame(None).unwrap();
        }
        model.apply(Action::Save).unwrap();
        let before = model.habitat.checkpoint_json().unwrap();
        model.advance_frame(None).unwrap();
        model.apply(Action::Load).unwrap();
        assert_eq!(model.habitat.checkpoint_json().unwrap(), before);
        assert!(model.paused);
        std::fs::write(&model.checkpoint, "invalid").unwrap();
        assert!(model.apply(Action::Load).is_err());
        assert_eq!(model.habitat.checkpoint_json().unwrap(), before);
        std::fs::remove_file(model.checkpoint).unwrap();
    }

    #[test]
    fn camera_preferences_and_fit_do_not_change_biological_trajectories() {
        let mut first = model();
        let mut second = model();
        first.zoom = 32.0;
        first.center = world_core::environment::Position { x: -10.0, y: 50.0 };
        first.show_neural = true;
        for _ in 0..60 {
            first.advance_frame(None).unwrap();
            second.advance_frame(None).unwrap();
        }
        assert_eq!(
            first.habitat.checkpoint_json().unwrap(),
            second.habitat.checkpoint_json().unwrap()
        );
        first.follow = true;
        first.fit();
        assert_eq!(first.zoom, 1.0);
        assert!(!first.follow);
        let (width, height, _, _) = first.habitat.grid().dimensions();
        assert_eq!(first.center.x, width * 0.5);
        assert_eq!(first.center.y, height * 0.5);
    }
}
