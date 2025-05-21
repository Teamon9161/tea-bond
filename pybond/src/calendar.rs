use super::utils::extract_date;
use chrono::NaiveDate;
use pyo3::prelude::*;
use tea_bond::export::calendar::{china, Calendar};

#[pyclass]
pub struct Ib;

#[pyclass]
pub struct Sse;

#[pymethods]
impl Ib {
    #[staticmethod]
    fn is_business_day(date: &Bound<'_, PyAny>) -> PyResult<bool> {
        let date = extract_date(date)?;
        Ok(china::IB.is_business_day(date))
    }

    #[staticmethod]
    #[pyo3(signature = (date, offset=0))]
    fn find_workday(date: &Bound<'_, PyAny>, offset: i32) -> PyResult<NaiveDate> {
        let date = extract_date(date)?;
        Ok(china::IB.find_workday(date, offset))
    }
}

#[pymethods]
impl Sse {
    #[staticmethod]
    fn is_business_day(date: &Bound<'_, PyAny>) -> PyResult<bool> {
        let date = extract_date(date)?;
        Ok(china::SSE.is_business_day(date))
    }

    #[staticmethod]
    #[pyo3(signature = (date, offset=0))]
    fn find_workday(date: &Bound<'_, PyAny>, offset: i32) -> PyResult<NaiveDate> {
        let date = extract_date(date)?;
        Ok(china::SSE.find_workday(date, offset))
    }
}
