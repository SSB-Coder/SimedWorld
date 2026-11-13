use numpy::{PyArray1, PyArrayMethods, PyReadonlyArray1, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use world_core::{
    contracts::{ACTION_WIDTH, ActionBatch, OBSERVATION_WIDTH, ObservationBatch},
    ids::OrganismId,
};

pub fn observation_dict<'py>(
    py: Python<'py>,
    batch: ObservationBatch,
) -> PyResult<Bound<'py, PyDict>> {
    let count = batch.ids.len();
    let values: Vec<f32> = batch.values.into_iter().flatten().collect();
    let result = PyDict::new(py);
    result.set_item("schema_version", batch.schema_version)?;
    result.set_item("tick", batch.tick)?;
    result.set_item(
        "ids",
        PyArray1::from_vec(py, batch.ids.into_iter().map(|id| id.0).collect()),
    )?;
    result.set_item(
        "topology_versions",
        PyArray1::from_vec(py, batch.topology_versions),
    )?;
    result.set_item(
        "values",
        PyArray1::from_vec(py, values).reshape([count, OBSERVATION_WIDTH])?,
    )?;
    Ok(result)
}

pub fn action_batch(
    schema_version: u32,
    tick: u64,
    ids: PyReadonlyArray1<'_, u64>,
    topology_versions: PyReadonlyArray1<'_, u64>,
    values: PyReadonlyArray2<'_, f32>,
) -> PyResult<ActionBatch> {
    if !ids.is_c_contiguous() || !topology_versions.is_c_contiguous() || !values.is_c_contiguous() {
        return Err(PyValueError::new_err("batch arrays must be C-contiguous"));
    }
    let ids = ids
        .as_slice()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    let versions = topology_versions
        .as_slice()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    if values.shape() != [ids.len(), ACTION_WIDTH] || versions.len() != ids.len() {
        return Err(PyValueError::new_err("action batch shapes do not agree"));
    }
    let values = values
        .as_slice()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(ActionBatch {
        schema_version,
        tick,
        ids: ids.iter().copied().map(OrganismId).collect(),
        topology_versions: versions.to_vec(),
        values: values
            .chunks_exact(ACTION_WIDTH)
            .map(|row| [row[0], row[1]])
            .collect(),
    })
}
