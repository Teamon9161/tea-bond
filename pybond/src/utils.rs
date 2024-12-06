use std::borrow::Cow;
use std::sync::Arc;

use crate::bond::PyBond;
use crate::future::PyFuture;
use chrono::NaiveDate;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use tea_bond::{Bond, Future};

/// Extract a NaiveDate from a Python object. Accepts either a Python date object or a date string.
pub fn extract_date(dt: &Bound<'_, PyAny>) -> PyResult<NaiveDate> {
    if let Ok(dt) = dt.extract() {
        Ok(dt)
    } else if let Ok(str_dt) = dt.extract::<Cow<'_, str>>() {
        let dt = str_dt.parse().map_err(|_| {
            PyValueError::new_err(format!("Can not parse {} to a Date object", str_dt))
        })?;
        Ok(dt)
    } else if dt.is_none() {
        Ok(Default::default())
    } else {
        Err(PyValueError::new_err(format!(
            "Can not parse {} to a Date object",
            dt
        )))
    }
}

/// Extract a tuple of two NaiveDates from a Python object. Accepts either a tuple or list containing two dates.
pub fn extract_date2(dts: &Bound<'_, PyAny>) -> PyResult<(NaiveDate, NaiveDate)> {
    if let Ok(dts) = dts.downcast::<PyTuple>() {
        Ok((
            extract_date(&dts.get_item(0)?)?,
            extract_date(&dts.get_item(1)?)?,
        ))
    } else if let Ok(dts) = dts.downcast::<PyList>() {
        Ok((
            extract_date(&dts.get_item(0)?)?,
            extract_date(&dts.get_item(1)?)?,
        ))
    } else {
        Err(PyValueError::new_err("Expect a tuple or list of two dates"))
    }
}

/// Extract a PyFuture from a Python object. Accepts either a PyFuture object or a future code string.
pub fn get_future(future: &Bound<'_, PyAny>) -> PyResult<PyFuture> {
    if let Ok(future) = future.extract::<PyFuture>() {
        Ok(future)
    } else if let Ok(future_str) = future.extract::<Cow<'_, str>>() {
        let future_rs = Future::new(future_str);
        Ok(PyFuture(Arc::new(future_rs)))
    } else {
        Err(PyValueError::new_err(
            "Expected a Future object or a Future code string (e.g. T2503)",
        ))
    }
}

/// Helper function to create a PyBond from a bond code string
fn get_bond_from_code(bond_code: &str) -> PyResult<PyBond> {
    let bond_rs = Bond::read_json(bond_code, None).map_err(|e| {
        PyValueError::new_err(format!(
            "Can not read bond {} from default path, {}",
            bond_code, e
        ))
    })?;
    Ok(PyBond(Arc::new(bond_rs)))
}

/// Extract a PyBond from a Python object. Accepts a PyBond object, bond code string, or bond code integer.
pub fn get_bond(bond: &Bound<'_, PyAny>) -> PyResult<PyBond> {
    if let Ok(bond) = bond.extract::<PyBond>() {
        Ok(bond)
    } else if let Ok(bond_str) = bond.extract::<Cow<'_, str>>() {
        get_bond_from_code(bond_str.as_ref())
    } else if let Ok(bond_code) = bond.extract::<i32>() {
        get_bond_from_code(&bond_code.to_string())
    } else {
        Err(PyValueError::new_err(
            "Expect a Bond object or a Bond code string",
        ))
    }
}
