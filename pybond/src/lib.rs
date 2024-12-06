#![allow(clippy::too_many_arguments)]

use pyo3::prelude::*;

#[cfg(feature = "numpy")]
mod batch_eval;
mod bond;
mod future;
mod tf_evaluator;
mod utils;

use bond::PyBond;
use future::PyFuture;
use tf_evaluator::PyTfEvaluator;

#[pymodule]
fn pybond(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBond>()?;
    m.add_class::<PyFuture>()?;
    m.add_class::<PyTfEvaluator>()?;
    Ok(())
}
