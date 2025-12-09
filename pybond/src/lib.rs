#![allow(clippy::too_many_arguments)]

use pyo3::prelude::*;

#[cfg(feature = "batch")]
mod batch_eval;
mod bond;
mod calendar;
mod future;
#[cfg(feature = "persist")]
mod persist;
#[cfg(feature = "pnl")]
mod pnl;
mod tf_evaluator;
mod utils;

use bond::PyBond;
#[allow(unused_imports)]
use bond_ffi::*;
use future::PyFuture;
use tf_evaluator::PyTfEvaluator;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[pyfunction]
pub fn get_version() -> &'static str {
    VERSION
}

#[pymodule]
fn pybond(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(feature = "download")]
    m.add_function(wrap_pyfunction!(bond::download_bond, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    #[cfg(feature = "persist")]
    m.add_function(wrap_pyfunction!(persist::update_info_from_wind_sql_df, m)?)?;
    m.add_class::<calendar::Ib>()?;
    m.add_class::<calendar::Sse>()?;
    m.add_class::<PyBond>()?;
    m.add_class::<PyFuture>()?;
    m.add_class::<PyTfEvaluator>()?;
    Ok(())
}
