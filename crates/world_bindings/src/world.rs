use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBool, PyDict, PyInt},
};
use world_core::world::FixtureWorld;

use crate::arrays;

#[pyclass(name = "FixtureWorld", module = "world._native")]
pub struct PyFixtureWorld {
    inner: FixtureWorld,
}

#[pymethods]
impl PyFixtureWorld {
    #[new]
    fn new(config_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: FixtureWorld::from_config_json(config_json).map_err(value_error)?,
        })
    }

    #[getter]
    fn tick(&self) -> u64 {
        self.inner.tick()
    }

    fn observe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        arrays::observation_dict(py, self.inner.observe())
    }

    #[pyo3(signature = (*, schema_version, tick, ids, topology_versions, values))]
    fn apply_actions(
        &mut self,
        schema_version: Bound<'_, PyAny>,
        tick: Bound<'_, PyAny>,
        ids: PyReadonlyArray1<'_, u64>,
        topology_versions: PyReadonlyArray1<'_, u64>,
        values: PyReadonlyArray2<'_, f32>,
    ) -> PyResult<()> {
        let schema_version = u32::try_from(unsigned_integer(&schema_version, "schema_version")?)
            .map_err(|_| PyValueError::new_err("schema_version exceeds unsigned 32-bit range"))?;
        let tick = unsigned_integer(&tick, "tick")?;
        self.inner
            .apply_actions(arrays::action_batch(
                schema_version,
                tick,
                ids,
                topology_versions,
                values,
            )?)
            .map_err(value_error)
    }

    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(value_error)
    }

    #[staticmethod]
    fn from_snapshot_json(text: &str) -> PyResult<Self> {
        Ok(Self {
            inner: FixtureWorld::from_snapshot_json(text).map_err(value_error)?,
        })
    }
}

fn value_error(error: world_core::contracts::ContractError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn unsigned_integer(value: &Bound<'_, PyAny>, name: &str) -> PyResult<u64> {
    if !value.is_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err(format!("{name} must be an integer")));
    }
    value.extract()
}
