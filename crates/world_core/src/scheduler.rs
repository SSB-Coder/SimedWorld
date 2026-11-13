use serde::{Deserialize, Serialize};

use crate::{
    clock::Clock,
    resources::{AccountingError, Result, finite},
    rules::WorldRules,
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scheduler {
    tick: u64,
    last_ecology_tick: u64,
}

impl Scheduler {
    pub(crate) fn validate(&self, rules: &WorldRules) -> Result<()> {
        if self.last_ecology_tick
            != self.tick / rules.ecology_interval_ticks * rules.ecology_interval_ticks
        {
            return Err(AccountingError::Invariant(
                "clock window is inconsistent".into(),
            ));
        }
        finite(
            self.tick as f64 * rules.physics_dt,
            "checkpoint time",
            false,
        )
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn last_ecology_tick(&self) -> u64 {
        self.last_ecology_tick
    }

    pub fn next(&self, rules: &WorldRules) -> Result<(Self, Option<f64>)> {
        finite(rules.physics_dt, "physics duration", true)?;
        if rules.ecology_interval_ticks == 0 {
            return Err(AccountingError::Invalid(
                "ecology interval must be positive".into(),
            ));
        }
        let tick = Clock(self.tick)
            .next()
            .map_err(|error| AccountingError::Invalid(error.to_string()))?
            .0;
        finite(tick as f64 * rules.physics_dt, "elapsed model time", false)?;
        if tick.is_multiple_of(rules.ecology_interval_ticks) {
            let duration = (tick - self.last_ecology_tick) as f64 * rules.physics_dt;
            finite(duration, "ecology window", true)?;
            Ok((
                Self {
                    tick,
                    last_ecology_tick: tick,
                },
                Some(duration),
            ))
        } else {
            Ok((
                Self {
                    tick,
                    last_ecology_tick: self.last_ecology_tick,
                },
                None,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_and_nonfinite_time_do_not_change_the_clock() {
        let mut rules =
            WorldRules::from_json(include_str!("../../../configs/world_rules_v0.json")).unwrap();
        let scheduler = Scheduler {
            tick: u64::MAX,
            last_ecology_tick: 0,
        };
        assert!(scheduler.next(&rules).is_err());
        assert_eq!(scheduler.tick(), u64::MAX);
        rules.physics_dt = 1e308;
        let scheduler = Scheduler {
            tick: 1,
            last_ecology_tick: 0,
        };
        assert!(scheduler.next(&rules).is_err());
        assert_eq!(scheduler.tick(), 1);
    }
}
