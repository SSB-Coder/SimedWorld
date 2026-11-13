//! Python-independent native core.

pub mod agents;
pub mod anatomy;
pub mod audit;
mod checkpoint;
pub use checkpoint::write_atomic;
mod circle_world;
pub mod clock;
pub mod config;
pub mod contracts;
pub mod corpses;
mod development;
pub mod diagnostics;
pub mod display;
pub mod ecology;
pub mod environment;
pub mod events;
mod evolution;
pub mod evolution_config;
pub mod food_web;
mod foraging;
pub mod habitat;
pub mod habitat_config;
pub mod ids;
pub mod inherited;
pub mod interactions;
pub mod lifecycle;
pub mod metabolism;
pub mod organism;
pub mod physiology;
pub mod resources;
pub mod rules;
pub mod scheduler;
mod social;
mod spatial;
mod telemetry;
pub mod terrain;
pub mod transactions;
pub mod world;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checked addition for verifying native calls.
pub fn checked_add(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}
