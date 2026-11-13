//! Python bindings for the native core.

use pyo3::prelude::*;

mod arrays;
mod habitat;
mod resources;
mod world;

#[pymodule]
mod _native {
    use super::*;
    use pyo3::exceptions::PyOverflowError;

    #[pymodule_export]
    use crate::world::PyFixtureWorld;

    #[pymodule_export]
    use crate::resources::PyResourceLedger;

    #[pymodule_export]
    use crate::habitat::PyHabitat;

    #[pyfunction]
    fn core_version() -> &'static str {
        world_core::VERSION
    }

    /// Add signed 64-bit integers, reporting overflow as a Python exception.
    #[pyfunction]
    fn checked_add(a: i64, b: i64) -> PyResult<i64> {
        world_core::checked_add(a, b)
            .ok_or_else(|| PyOverflowError::new_err("sum exceeds signed 64-bit range"))
    }

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", env!("CARGO_PKG_VERSION"))
    }
}
