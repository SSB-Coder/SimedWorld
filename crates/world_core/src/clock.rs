use crate::contracts::{ContractError, Result};

#[derive(Clone, Copy, Debug)]
pub struct Clock(pub u64);

impl Clock {
    pub fn next(self) -> Result<Self> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| ContractError("tick overflow".into()))
    }
}
