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
    #[cfg(feature = "download")]
    m.add_function(wrap_pyfunction!(bond::download_bond_from_china_money, m)?)?;
    m.add_class::<PyBond>()?;
    m.add_class::<PyFuture>()?;
    m.add_class::<PyTfEvaluator>()?;
    Ok(())
}
