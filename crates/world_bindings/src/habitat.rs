use numpy::{PyReadonlyArray1, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use world_core::{habitat::Habitat, resources::AccountingError};

#[pyclass(name = "Habitat", module = "world._native")]
pub struct PyHabitat {
    inner: Habitat,
}

#[pymethods]
impl PyHabitat {
    #[new]
    fn new(rules_json: &str, config_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::from_json(rules_json, config_json).map_err(error)?,
        })
    }

    #[getter]
    fn tick(&self) -> u64 {
        self.inner.tick()
    }

    #[getter]
    fn decision_duration(&self) -> f64 {
        let rules = self.inner.rules();
        rules.physics_dt * rules.ecology_interval_ticks as f64
    }

    #[getter]
    fn elapsed_time(&self) -> f64 {
        self.inner.tick() as f64 * self.inner.rules().physics_dt
    }

    #[getter]
    fn next_event(&self) -> u64 {
        self.inner.diagnostics().map_or(0, |d| d.next_event)
    }

    fn advance(&mut self) -> PyResult<()> {
        self.inner.advance().map_err(error)
    }

    fn fork(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn agent_frame_json(&self) -> PyResult<String> {
        self.inner.agent_frame_json().map_err(error)
    }

    fn events_since_json(&self, sequence: u64) -> PyResult<String> {
        self.inner.events_since_json(sequence).map_err(error)
    }

    fn agent_frame_delta_json(&self, known: &str) -> PyResult<String> {
        self.inner.agent_frame_delta_json(known).map_err(error)
    }

    #[pyo3(signature = (selected=None))]
    fn display_json(&self, selected: Option<u64>) -> PyResult<String> {
        self.inner.display_json(selected).map_err(error)
    }

    fn advance_controlled(&mut self, actions: &str) -> PyResult<()> {
        self.inner.advance_controlled(actions).map_err(error)
    }

    fn propose_controlled(&self, actions: &str) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.propose_controlled(actions).map_err(error)?,
        })
    }

    fn propose_controlled_packed(
        &self,
        actions: &str,
        node_keys: PyReadonlyArray2<'_, u64>,
        biases: PyReadonlyArray1<'_, f64>,
        edge_keys: PyReadonlyArray2<'_, u64>,
        weights: PyReadonlyArray1<'_, f64>,
    ) -> PyResult<Self> {
        if node_keys.shape()[1] != 2 || edge_keys.shape()[1] != 2 {
            return Err(PyValueError::new_err(
                "parameter keys must have two columns",
            ));
        }
        let contiguous = || PyValueError::new_err("parameter arrays must be C-contiguous");
        if !node_keys.is_c_contiguous()
            || !biases.is_c_contiguous()
            || !edge_keys.is_c_contiguous()
            || !weights.is_c_contiguous()
        {
            return Err(contiguous());
        }
        let parameters = world_core::agents::PackedParameters {
            node_keys: node_keys.as_slice().map_err(|_| contiguous())?,
            biases: biases.as_slice().map_err(|_| contiguous())?,
            edge_keys: edge_keys.as_slice().map_err(|_| contiguous())?,
            weights: weights.as_slice().map_err(|_| contiguous())?,
        };
        Ok(Self {
            inner: self
                .inner
                .propose_controlled_packed(actions, parameters)
                .map_err(error)?,
        })
    }

    fn audit_json(&self) -> PyResult<String> {
        self.inner.audit_json().map_err(error)
    }

    fn summary_json(&self) -> PyResult<String> {
        self.inner.summary_json().map_err(error)
    }

    fn summary_compact_json(&self) -> PyResult<String> {
        self.inner.summary_compact_json().map_err(error)
    }

    fn telemetry_json(&self) -> PyResult<String> {
        self.inner.telemetry_json().map_err(error)
    }

    fn subject_json(&self, id: u64) -> PyResult<String> {
        self.inner.subject_json(id).map_err(error)
    }

    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(error)
    }

    fn checkpoint_json(&self) -> PyResult<String> {
        self.inner.checkpoint_json().map_err(error)
    }

    #[staticmethod]
    fn from_checkpoint_json(text: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::from_checkpoint_json(text).map_err(error)?,
        })
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.inner.save(path).map_err(error)
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Habitat::load(path).map_err(error)?,
        })
    }
}

fn error(value: AccountingError) -> PyErr {
    match value {
        AccountingError::Invalid(message) => PyValueError::new_err(message),
        AccountingError::Invariant(message) => PyRuntimeError::new_err(message),
    }
}
