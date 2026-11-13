use std::{collections::VecDeque, path::PathBuf};

pub struct Options {
    pub rules: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub load: Option<PathBuf>,
    pub checkpoint: PathBuf,
    pub paused: bool,
    pub run_for_ticks: Option<u64>,
    pub capture: Option<PathBuf>,
    pub observe: Option<PathBuf>,
    pub commands: Option<PathBuf>,
    pub select: Option<u64>,
    pub zoom: f32,
    pub brain: bool,
    pub fields: bool,
    pub history: bool,
}

impl Options {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut args: VecDeque<_> = args.collect();
        let mut options = Self {
            rules: None,
            config: None,
            load: None,
            checkpoint: "artifacts/world-checkpoint.json".into(),
            paused: false,
            run_for_ticks: None,
            capture: None,
            observe: None,
            commands: None,
            select: None,
            zoom: 1.0,
            brain: false,
            fields: false,
            history: false,
        };
        while let Some(argument) = args.pop_front() {
            if argument == "--paused" {
                options.paused = true;
                continue;
            }
            if argument == "--brain" {
                options.brain = true;
                continue;
            }
            if argument == "--fields" {
                options.fields = true;
                continue;
            }
            if argument == "--history" {
                options.history = true;
                continue;
            }
            let value = args
                .pop_front()
                .ok_or_else(|| format!("missing value for {argument}"))?;
            match argument.as_str() {
                "--rules" => options.rules = Some(value.into()),
                "--config" => options.config = Some(value.into()),
                "--load" => options.load = Some(value.into()),
                "--checkpoint" => options.checkpoint = value.into(),
                "--observe" => options.observe = Some(value.into()),
                "--commands" => options.commands = Some(value.into()),
                "--select" => {
                    options.select = Some(
                        value
                            .parse()
                            .map_err(|_| "select must be an unsigned identity")?,
                    )
                }
                "--zoom" => {
                    options.zoom = value.parse().map_err(|_| "zoom must be a number")?;
                    if !options.zoom.is_finite() || !(0.25..=64.0).contains(&options.zoom) {
                        return Err("zoom must lie in [0.25,64]".into());
                    }
                }
                "--run-for-ticks" => {
                    options.run_for_ticks = Some(
                        value
                            .parse()
                            .map_err(|_| "run-for-ticks must be an unsigned integer")?,
                    )
                }
                "--capture" => {
                    let path = PathBuf::from(value);
                    if path.extension().and_then(|x| x.to_str()) != Some("bmp") {
                        return Err("capture output must use .bmp".into());
                    }
                    options.capture = Some(path);
                }
                _ => return Err(format!("unknown argument: {argument}")),
            }
        }
        if options.capture.is_some() && options.run_for_ticks.is_none() {
            return Err("capture requires --run-for-ticks".into());
        }
        if options.observe.is_some() != options.commands.is_some() {
            return Err("observe and commands must be supplied together".into());
        }
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_reject_missing_unknown_and_invalid_values() {
        for args in [
            vec!["--rules"],
            vec!["--unknown", "x"],
            vec!["--run-for-ticks", "-1"],
            vec!["--capture", "x.png"],
        ] {
            assert!(Options::parse(args.into_iter().map(str::to_string)).is_err());
        }
        assert!(
            Options::parse(
                ["--run-for-ticks", "0", "--capture", "test.bmp"]
                    .into_iter()
                    .map(str::to_string)
            )
            .is_ok()
        );
    }
}
