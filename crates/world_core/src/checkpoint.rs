use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    VERSION,
    ecology::{EcologyReport, EcologyState},
    environment::{EnvironmentGrid, Position},
    habitat::{Habitat, WindowReport},
    habitat_config::HabitatConfig,
    ids::OrganismId,
    resources::{AccountingError, Result},
    rules::WorldRules,
    scheduler::Scheduler,
    transactions::{PopulationState, ResourceLedger},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    id: OrganismId,
    position: Position,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Motion {
    id: OrganismId,
    heading: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema_version: u32,
    kind: String,
    engine_version: String,
    rules: WorldRules,
    config: HabitatConfig,
    grid: EnvironmentGrid,
    positions: Vec<Placement>,
    motion: Vec<Motion>,
    ledger: PopulationState,
    clock: Scheduler,
    next_id: u64,
    totals: EcologyReport,
    incident_light: f64,
    escaped_light: f64,
    last_update: Option<WindowReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diagnostics: Option<crate::diagnostics::Diagnostics>,
    #[serde(default)]
    interactions: crate::interactions::InteractionState,
}

impl Habitat {
    pub fn checkpoint_json(&self) -> Result<String> {
        self.audit()?;
        let checkpoint = Checkpoint {
            schema_version: 1,
            kind: "world.habitat".into(),
            engine_version: VERSION.into(),
            rules: self.rules.clone(),
            config: self.config.clone(),
            grid: self.grid.clone(),
            positions: self
                .positions
                .iter()
                .map(|(id, position)| Placement {
                    id: *id,
                    position: *position,
                })
                .collect(),
            motion: self
                .ecology
                .headings
                .iter()
                .map(|(id, heading)| Motion {
                    id: *id,
                    heading: *heading,
                })
                .collect(),
            ledger: self.ledger.population_state(),
            clock: self.scheduler,
            next_id: self.ecology.next_id,
            totals: self.ecology.totals.clone(),
            incident_light: self.incident_light,
            escaped_light: self.escaped_light,
            last_update: self.last_update.clone(),
            diagnostics: self.diagnostics.clone(),
            interactions: self.ecology.interactions.clone(),
        };
        serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| AccountingError::Invalid(e.to_string()))
    }

    pub fn from_checkpoint_json(text: &str) -> Result<Self> {
        let checkpoint: Checkpoint = serde_json::from_str(text)
            .map_err(|e| AccountingError::Invalid(format!("invalid habitat checkpoint: {e}")))?;
        if checkpoint.schema_version != 1
            || checkpoint.kind != "world.habitat"
            || checkpoint.engine_version != VERSION
        {
            return Err(AccountingError::Invalid(
                "unsupported habitat checkpoint format or engine version".into(),
            ));
        }
        checkpoint.rules.validate()?;
        checkpoint.config.validate(&checkpoint.rules)?;
        checkpoint.clock.validate(&checkpoint.rules)?;
        checkpoint.grid.validate(&checkpoint.rules)?;
        if !checkpoint.rules.close(
            checkpoint.ledger.initial_nutrient,
            checkpoint.config.initial_nutrient,
        ) || !checkpoint.rules.close(
            checkpoint.ledger.initial_energy,
            checkpoint.config.initial_energy,
        ) {
            return Err(AccountingError::Invariant(
                "checkpoint starting budgets disagree".into(),
            ));
        }
        let mut positions = BTreeMap::new();
        for record in checkpoint.positions {
            if positions.insert(record.id, record.position).is_some() {
                return Err(AccountingError::Invalid(
                    "duplicate checkpoint position ID".into(),
                ));
            }
        }
        let mut headings = BTreeMap::new();
        for record in checkpoint.motion {
            if headings.insert(record.id, record.heading).is_some() {
                return Err(AccountingError::Invalid(
                    "duplicate checkpoint motion ID".into(),
                ));
            }
        }
        let founders = checkpoint.config.producers.count
            + checkpoint.config.consumers.as_ref().map_or(0, |p| p.count)
            + checkpoint
                .config
                .populations
                .iter()
                .map(|p| p.count)
                .sum::<usize>();
        let records = checkpoint.ledger.organisms.len();
        let dead = checkpoint
            .ledger
            .organisms
            .iter()
            .filter(|o| !o.alive)
            .count();
        if (records as u64).checked_sub(founders as u64) != Some(checkpoint.totals.births)
            || checkpoint.totals.deaths != dead as u64
        {
            return Err(AccountingError::Invariant(
                "checkpoint population counters disagree".into(),
            ));
        }
        if checkpoint.clock.last_ecology_tick() == 0 {
            if checkpoint.last_update.is_some()
                || checkpoint.totals.births != 0
                || checkpoint.totals.deaths != 0
                || checkpoint.incident_light != 0.0
                || checkpoint.escaped_light != 0.0
            {
                return Err(AccountingError::Invariant(
                    "initial checkpoint has processed history".into(),
                ));
            }
        } else {
            checkpoint
                .last_update
                .as_ref()
                .ok_or_else(|| {
                    AccountingError::Invariant("checkpoint window receipt is missing".into())
                })?
                .validate(checkpoint.clock, &checkpoint.rules)?;
        }
        let grid_template = EnvironmentGrid::new(
            &checkpoint.rules,
            checkpoint.config.mineral_nutrient_per_cell,
            checkpoint.config.light_energy_per_cell_per_time,
            checkpoint.config.light_gradient,
            &checkpoint.config.cell_overrides,
        )?;
        for (saved, expected) in checkpoint.grid.cells.iter().zip(grid_template.cells) {
            if saved.light_energy_per_time != expected.light_energy_per_time {
                return Err(AccountingError::Invariant(
                    "checkpoint light field disagrees with configuration".into(),
                ));
            }
        }
        let ledger =
            ResourceLedger::from_population_state(checkpoint.rules.clone(), checkpoint.ledger)?;
        let habitat = Self {
            audit_cache: std::sync::OnceLock::new(),
            rules: checkpoint.rules,
            config: checkpoint.config,
            grid: checkpoint.grid,
            positions,
            ledger,
            scheduler: checkpoint.clock,
            incident_light: checkpoint.incident_light,
            escaped_light: checkpoint.escaped_light,
            last_update: checkpoint.last_update,
            diagnostics: checkpoint.diagnostics,
            ecology: EcologyState {
                interactions: checkpoint.interactions,
                headings,
                next_id: checkpoint.next_id,
                totals: checkpoint.totals,
            },
        };
        habitat.audit()?;
        Ok(habitat)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        write_atomic(path.as_ref(), self.checkpoint_json()?.as_bytes())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|e| AccountingError::Invalid(e.to_string()))?;
        Self::from_checkpoint_json(&text)
    }
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| AccountingError::Invalid(e.to_string()))?;
    let filename = path
        .file_name()
        .ok_or_else(|| AccountingError::Invalid("checkpoint filename is missing".into()))?;
    let mut selected = None;
    for attempt in 0..1000 {
        let temporary = parent.join(format!(
            ".{}.{}.{}.tmp",
            filename.to_string_lossy(),
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => {
                selected = Some((Temporary(temporary), file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(AccountingError::Invalid(error.to_string())),
        }
    }
    let (temporary, mut file) = selected.ok_or_else(|| {
        AccountingError::Invalid("no available checkpoint temporary filename".into())
    })?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| AccountingError::Invalid(e.to_string()))?;
    drop(file);
    for attempt in 0..40 {
        match fs::rename(&temporary.0, path) {
            Ok(()) => return Ok(()),
            Err(error)
                if cfg!(windows)
                    && error.kind() == std::io::ErrorKind::PermissionDenied
                    && attempt < 39 =>
            {
                std::thread::sleep(std::time::Duration::from_millis(5))
            }
            Err(error) => return Err(AccountingError::Invalid(error.to_string())),
        }
    }
    Ok(())
}

struct Temporary(PathBuf);

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
