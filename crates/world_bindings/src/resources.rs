use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use world_core::{resources::AccountingError, transactions::ResourceLedger};

#[pyclass(name = "ResourceLedger", module = "world._native")]
pub struct PyResourceLedger {
    inner: ResourceLedger,
}

#[pymethods]
impl PyResourceLedger {
    #[new]
    fn new(rules_json: &str, initial_json: &str) -> PyResult<Self> {
        Ok(Self {
            inner: ResourceLedger::from_json(rules_json, initial_json).map_err(error)?,
        })
    }

    fn apply_json(&mut self, command_json: &str) -> PyResult<()> {
        self.inner.apply_json(command_json).map_err(error)
    }

    fn audit_json(&self) -> PyResult<String> {
        self.inner.audit_json().map_err(error)
    }
    fn summary_json(&self, label: &str) -> PyResult<String> {
        self.inner.summary_json(label).map_err(error)
    }
    fn snapshot_json(&self) -> PyResult<String> {
        self.inner.snapshot_json().map_err(error)
    }
}

fn error(value: AccountingError) -> PyErr {
    match value {
        AccountingError::Invalid(message) => PyValueError::new_err(message),
        AccountingError::Invariant(message) => PyRuntimeError::new_err(message),
    }
}
