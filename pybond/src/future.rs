use std::{ops::Deref, sync::Arc};

use crate::utils::extract_date;
use chrono::NaiveDate;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tea_bond::Future;

#[pyclass(name = "Future")]
#[derive(Clone)]
pub struct PyFuture(pub Arc<Future>);

impl From<Future> for PyFuture {
    #[inline]
    fn from(future: Future) -> Self {
        Self(Arc::new(future))
    }
}

impl Deref for PyFuture {
    type Target = Future;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pymethods]
impl PyFuture {
    #[new]
    pub fn new(code: &str) -> Self {
        Self(Arc::new(Future::new(code)))
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }

    /// 判断是否是可交割券
    ///
    /// delivery_date: 可以传入已计算过的期货配对缴款日避免重复计算
    #[pyo3(signature = (carry_date, maturity_date, delivery_date=None))]
    fn is_deliverable(
        &self,
        carry_date: &Bound<'_, PyAny>,
        maturity_date: &Bound<'_, PyAny>,
        delivery_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let carry_date = extract_date(carry_date)?;
        let maturity_date = extract_date(maturity_date)?;
        let delivery_date = delivery_date.map(extract_date).transpose()?;
        self.0
            .is_deliverable(carry_date, maturity_date, delivery_date)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// 计算期货合约的最后交易日
    ///
    /// 计算国债期货的最后交易日=合约到期月份的第二个星期五
    /// 根据合约代码, 依据中金所的国债期货合约最后交易日的说, 返回该合约的最后交易日
    /// 获取年月部分
    fn last_trading_date(&self) -> PyResult<NaiveDate> {
        self.0
            .last_trading_date()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// 获取期货合约的配对缴款日
    ///
    /// 交割日为3天,其中第2天为缴款日,即最后交易日的第2个交易日,最后交易日一定为周五,所以缴款日一定是一个周二
    fn deliver_date(&self) -> PyResult<NaiveDate> {
        self.0
            .deliver_date()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// 获取期货合约的类型
    fn future_type(&self) -> PyResult<String> {
        self.0
            .future_type()
            .map(|ft| format!("{ft:?}"))
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
