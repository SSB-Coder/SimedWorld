use crate::model::Action;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use world_core::{
    habitat::Habitat,
    resources::{AccountingError, Result},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u32,
    kind: String,
    session: String,
    sequence: u64,
    paused: bool,
    speed: u32,
    #[serde(default)]
    actual_speed: Option<f64>,
    status: String,
    last_command: Option<String>,
    state: serde_json::Value,
}

pub struct Observed {
    pub habitat: Option<Habitat>,
    pub display: Option<world_core::display::DisplayFrame>,
    pub paused: bool,
    pub speed: u32,
    pub actual_speed: Option<f64>,
    pub status: String,
}

pub struct Observer {
    pub stream: PathBuf,
    pub commands: PathBuf,
    pub session: String,
    sequence: u64,
    command: u64,
    last_poll: Instant,
    pub memory: BTreeMap<(u64, u64), f64>,
    state: serde_json::Value,
    pub acknowledged: Option<String>,
    pub save_request: Option<String>,
}

impl Observer {
    pub fn open(stream: PathBuf, commands: PathBuf) -> Result<(Self, Observed)> {
        let mut client = Self {
            stream,
            commands,
            session: String::new(),
            sequence: 0,
            command: 0,
            last_poll: Instant::now() - Duration::from_secs(1),
            memory: BTreeMap::new(),
            state: serde_json::Value::Null,
            acknowledged: None,
            save_request: None,
        };
        let observed = client.read()?.ok_or_else(|| {
            AccountingError::Invalid("observer stream has no published state".into())
        })?;
        Ok((client, observed))
    }

    pub fn poll(&mut self) -> Result<Option<Observed>> {
        if self.last_poll.elapsed() < Duration::from_millis(70) {
            return Ok(None);
        }
        self.last_poll = Instant::now();
        self.read()
    }

    fn read(&mut self) -> Result<Option<Observed>> {
        let text = match fs::read_to_string(&self.stream) {
            Ok(text) => text,
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    && !self.session.is_empty() =>
            {
                return Ok(None);
            }
            Err(error) => return Err(AccountingError::Invalid(error.to_string())),
        };
        let envelope: Envelope =
            serde_json::from_str(&text).map_err(|e| AccountingError::Invalid(e.to_string()))?;
        if envelope.schema_version != 1
            || !["world.observer", "world.display"].contains(&envelope.kind.as_str())
            || envelope.session.is_empty()
            || ![1, 2, 4, 8, 16].contains(&envelope.speed)
            || envelope
                .actual_speed
                .is_some_and(|s| !s.is_finite() || s < 0.0)
        {
            return Err(AccountingError::Invalid(
                "unsupported observer stream".into(),
            ));
        }
        if envelope.session == self.session && envelope.sequence <= self.sequence {
            return Ok(None);
        }
        if envelope.kind == "world.display" {
            let display: world_core::display::DisplayFrame = serde_json::from_value(envelope.state)
                .map_err(|e| AccountingError::Invalid(e.to_string()))?;
            display.validate()?;
            self.session = envelope.session;
            self.sequence = envelope.sequence;
            self.acknowledged = envelope.last_command;
            self.state = serde_json::Value::Null;
            return Ok(Some(Observed {
                habitat: None,
                display: Some(display),
                paused: envelope.paused,
                speed: envelope.speed,
                actual_speed: envelope.actual_speed,
                status: envelope.status,
            }));
        }
        let state = &envelope.state;
        if state["schema_version"] != 1
            || state["kind"] != "world.ecosystem"
            || state["engine_version"] != world_core::VERSION
        {
            return Err(AccountingError::Invalid(
                "unsupported coordinated world state".into(),
            ));
        }
        let habitat = Habitat::from_checkpoint_json(&state["world"].to_string())?;
        let neural = &state["neural"];
        if neural["tick"].as_u64() != Some(habitat.tick())
            || neural["kind"] != "sparse_leaky_coo_v1"
        {
            return Err(AccountingError::Invalid(
                "world and neural display ticks disagree".into(),
            ));
        }
        let keys: Vec<(u64, u64)> = serde_json::from_value(neural["keys"].clone())
            .map_err(|e| AccountingError::Invalid(e.to_string()))?;
        let values: Vec<f64> = serde_json::from_value(neural["memory"].clone())
            .map_err(|e| AccountingError::Invalid(e.to_string()))?;
        let expected: Vec<_> = habitat
            .organisms()
            .values()
            .filter(|o| o.alive)
            .flat_map(|o| {
                o.genome
                    .brain
                    .iter()
                    .flat_map(move |b| b.nodes.iter().map(move |n| (o.id.0, n.id)))
            })
            .collect();
        if keys != expected
            || keys.len() != values.len()
            || values.iter().any(|v| !v.is_finite() || v.abs() > 1.0)
        {
            return Err(AccountingError::Invalid(
                "invalid neural display state".into(),
            ));
        }
        let memory: BTreeMap<_, _> = keys.into_iter().zip(values).collect();
        self.session = envelope.session;
        self.sequence = envelope.sequence;
        self.memory = memory;
        self.state = envelope.state;
        self.acknowledged = envelope.last_command.clone();
        let status = if envelope.last_command.is_some() {
            envelope.status
        } else {
            format!("{} / Python owns simulation", envelope.status)
        };
        Ok(Some(Observed {
            habitat: Some(habitat),
            display: None,
            paused: envelope.paused,
            speed: envelope.speed,
            actual_speed: envelope.actual_speed,
            status,
        }))
    }

    pub fn save_published(&self, path: &std::path::Path) -> Result<()> {
        if self.state.is_null() {
            return Err(AccountingError::Invalid(
                "display frames require an owner save command".into(),
            ));
        }
        world_core::write_atomic(path, self.state.to_string().as_bytes())
    }

    pub fn send(&mut self, action: &Action, path: &std::path::Path) -> Result<()> {
        let kind = match action {
            Action::TogglePause => "pause",
            Action::Advance => "advance",
            Action::Faster => "faster",
            Action::Slower => "slower",
            Action::Save => "save",
            Action::Load => "load",
            Action::Export => "export",
            Action::Select(_) => "inspect",
            _ => return Ok(()),
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AccountingError::Invalid(e.to_string()))?
            .as_nanos();
        self.command = self
            .command
            .checked_add(1)
            .ok_or_else(|| AccountingError::Invalid("observer command counter overflow".into()))?;
        let id = format!("{timestamp:040}-{}-{}", std::process::id(), self.command);
        let command_path = if let Action::Select(selected) = action {
            selected.map_or("0".into(), |id| id.0.to_string())
        } else {
            path.to_string_lossy().into_owned()
        };
        let payload = serde_json::json!({"schema_version":1,"session":self.session,"id":id,"kind":kind,"path":command_path});
        world_core::write_atomic(
            &self.commands.join(format!("cmd-{id}.json")),
            payload.to_string().as_bytes(),
        )?;
        if matches!(action, Action::Save) {
            self.save_request = Some(id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_commands_and_bad_frames_cannot_mutate_the_published_world() {
        let world = Habitat::from_json(
            include_str!("../../../configs/evolution_rules.json"),
            include_str!("../../../configs/evolution_habitat.json"),
        )
        .unwrap();
        let frame = world.agent_frame().unwrap();
        let keys: Vec<_> = frame
            .agents
            .iter()
            .flat_map(|a| a.brain.nodes.iter().map(move |n| (a.id.0, n.id)))
            .collect();
        let state = serde_json::json!({"schema_version":1,"kind":"world.ecosystem","engine_version":world_core::VERSION,
            "world":serde_json::from_str::<serde_json::Value>(&world.checkpoint_json().unwrap()).unwrap(),
            "neural":{"schema_version":1,"kind":"sparse_leaky_coo_v1","tick":0,"keys":keys,"memory":vec![0.0;keys.len()]}});
        let root = std::env::temp_dir().join(format!("world-observer-{}", std::process::id()));
        let stream = root.join("stream.json");
        let commands = root.join("commands");
        let saved = root.join("saved.json");
        let mut envelope = serde_json::json!({"schema_version":1,"kind":"world.observer","session":"test-session","sequence":1,"paused":false,"speed":1,"status":"test","last_command":null,"state":state});
        world_core::write_atomic(&stream, envelope.to_string().as_bytes()).unwrap();
        let (mut observer, observed) = Observer::open(stream.clone(), commands.clone()).unwrap();
        assert_eq!(
            observed.habitat.unwrap().checkpoint_json().unwrap(),
            world.checkpoint_json().unwrap()
        );
        observer.send(&Action::Advance, &saved).unwrap();
        let files: Vec<_> = fs::read_dir(&commands)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(files.len(), 1);
        let command: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&files[0]).unwrap()).unwrap();
        assert_eq!(command["kind"], "advance");
        assert_eq!(command["session"], "test-session");
        envelope["sequence"] = serde_json::json!(0);
        world_core::write_atomic(&stream, envelope.to_string().as_bytes()).unwrap();
        assert!(observer.read().unwrap().is_none());
        envelope["sequence"] = serde_json::json!(2);
        envelope["state"]["world"]["ledger"]["pools"]["heat"] = serde_json::json!(999999);
        world_core::write_atomic(&stream, envelope.to_string().as_bytes()).unwrap();
        assert!(observer.read().is_err());
        observer.save_published(&saved).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&saved).unwrap())
                .unwrap(),
            state
        );
        for file in files {
            fs::remove_file(file).unwrap();
        }
        fs::remove_file(stream).unwrap();
        fs::remove_file(saved).unwrap();
        fs::remove_dir(commands).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
