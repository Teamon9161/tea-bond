use std::{ops::Deref, sync::Arc};

use crate::utils::extract_date;
use chrono::NaiveDate;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::str::FromStr;
use tea_bond::{Future, FutureType};

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

    #[staticmethod]
    #[pyo3(signature = (start, end=None, future_type=None))]
    /// 获取指定时间段内有交易的期货合约
    fn trading_futures(
        start: &Bound<'_, PyAny>,
        end: Option<&Bound<'_, PyAny>>,
        future_type: Option<&str>,
    ) -> PyResult<Vec<String>> {
        let start = extract_date(start)?;
        let end = match end {
            Some(dt) => Some(extract_date(dt)?),
            None => None,
        };
        let future_type = match future_type {
            Some(s) => {
                Some(FutureType::from_str(s).map_err(|e| PyValueError::new_err(e.to_string()))?)
            }
            None => None,
        };
        Future::trading_futures(start, end, future_type)
            .map(|v| {
                v.into_iter()
                    .map(|f| {
                        let code = f.code.to_string();
                        match f.market {
                            Some(market) => format!("{code}.{market}"),
                            None => code,
                        }
                    })
                    .collect()
            })
            .map_err(|e| PyValueError::new_err(e.to_string()))
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

    /// 获取期货合约的首个交易日
    ///
    /// 对于首批上市合约,返回该品种的上市日期;
    /// 对于后续合约,返回前3季度合约最后交易日的下一个交易日
    fn first_trading_date(&self) -> PyResult<NaiveDate> {
        self.0
            .first_trading_date()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// 获取期货合约的交易区间
    ///
    /// 返回 (首个交易日, 最后交易日)
    fn trading_window(&self) -> PyResult<(NaiveDate, NaiveDate)> {
        self.0
            .trading_window()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn code(&self) -> String {
        self.0.code.to_string()
    }

    fn market(&self) -> Option<String> {
        self.0.market.as_deref().map(ToString::to_string)
    }

    /// 获取下一季月合约
    fn next_future(&self) -> PyResult<Self> {
        self.0
            .next_future()
            .map(|f| PyFuture(Arc::new(f)))
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// 获取上一季月合约
    fn prev_future(&self) -> PyResult<Self> {
        self.0
            .prev_future()
            .map(|f| PyFuture(Arc::new(f)))
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
