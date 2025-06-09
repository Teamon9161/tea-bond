use std::{ops::Deref, path::PathBuf};

use crate::utils::{extract_date, extract_date2};
use chrono::NaiveDate;
use pyo3::prelude::*;
use tea_bond::*;

#[pyclass(name = "Bond", subclass)]
#[derive(PartialEq, Eq, Clone)]
pub struct PyBond(pub CachedBond);

impl From<Bond> for PyBond {
    #[inline]
    fn from(bond: Bond) -> Self {
        Self(bond.into())
    }
}

impl Deref for PyBond {
    type Target = Bond;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "download")]
#[pyfunction]
pub fn download_bond(code: &str) -> PyResult<PyBond> {
    let rt = tea_bond::export::tokio::runtime::Runtime::new()?;
    let bond = rt.block_on(async { Bond::download(code).await })?;
    Ok(bond.into())
}

#[pymethods]
impl PyBond {
    /// Create a new Bond instance
    ///
    /// Args:
    ///     code (str): Bond code
    ///     path (str, optional): Path to directory containing bond data. Defaults to ""
    ///
    /// Returns:
    ///     Bond: New Bond instance
    ///
    /// Raises:
    ///     ValueError: If bond data cannot be read or parsed
    #[new]
    #[pyo3(signature = (code, path))]
    fn new(code: &str, path: Option<PathBuf>) -> PyResult<Self> {
        crate::utils::get_bond_from_code(code, path.as_deref())
    }

    /// Save the bond data to a file.
    ///
    /// Args:
    ///     path (Optional[PathBuf]): The path where the bond data will be saved.
    ///                               If not provided, the default path will be used.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     IOError: If the bond data cannot be saved to the specified path.
    #[pyo3(signature = (path=None))]
    fn save(&self, path: Option<PathBuf>) -> PyResult<()> {
        let path = Bond::get_save_path(self.bond_code(), path.as_deref());
        self.0.save(path).map_err(Into::into)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    /// 债券代码, 不包含交易所后缀
    #[getter]
    pub fn code(&self) -> &str {
        self.0.code()
    }

    /// 债券代码, 包含交易所后缀
    #[getter]
    pub fn full_code(&self) -> &str {
        &self.0.bond_code
    }

    /// 债券市场
    #[getter]
    pub fn market(&self) -> String {
        format!("{:?}", self.0.mkt)
    }

    /// 债券简称
    #[getter]
    pub fn abbr(&self) -> &str {
        &self.0.abbr
    }

    /// 债券名称 (alias for abbr)
    #[getter]
    pub fn name(&self) -> &str {
        self.abbr()
    }

    /// 债券面值
    #[getter]
    pub fn par_value(&self) -> f64 {
        self.0.par_value
    }

    /// 息票品种
    #[getter]
    pub fn coupon_type(&self) -> String {
        format!("{:?}", self.0.cp_type)
    }

    /// 息票利率类型
    #[getter]
    pub fn interest_type(&self) -> String {
        format!("{:?}", self.0.interest_type)
    }

    /// 票面利率, 浮动付息债券仅表示发行时票面利率
    #[getter]
    pub fn coupon_rate(&self) -> f64 {
        self.0.cp_rate_1st
    }

    /// 基准利率, 浮动付息债券适用
    #[getter]
    pub fn base_rate(&self) -> Option<f64> {
        self.0.base_rate
    }

    /// 固定利差, 浮动付息债券适用
    #[getter]
    pub fn rate_spread(&self) -> Option<f64> {
        self.0.rate_spread
    }

    /// 年付息次数
    #[getter]
    pub fn inst_freq(&self) -> i32 {
        self.0.inst_freq
    }

    /// 起息日
    #[getter]
    pub fn carry_date(&self) -> NaiveDate {
        self.0.carry_date
    }

    /// 到期日
    #[getter]
    pub fn maturity_date(&self) -> NaiveDate {
        self.0.maturity_date
    }

    /// 计息基准
    #[getter]
    pub fn day_count(&self) -> String {
        format!("{:?}", self.0.day_count)
    }

    /// 是否为零息债券
    pub fn is_zero_coupon(&self) -> bool {
        self.0.is_zero_coupon()
    }

    /// 剩余年数
    #[pyo3(signature = (date=None))]
    pub fn remain_year(&self, date: Option<&Bound<'_, PyAny>>) -> PyResult<f64> {
        let date = date.map(extract_date).transpose()?;
        Ok(self.0.remain_year(date.unwrap_or(self.maturity_date())))
    }

    /// 发行年数
    #[getter]
    pub fn issue_year(&self) -> i32 {
        self.0.issue_year()
    }

    /// 获取区间付息（单个付息周期的利息金额）
    ///
    /// 区间付息 = 票面利率 * 面值 / 年付息次数
    #[getter]
    pub fn get_coupon(&self) -> f64 {
        self.0.get_coupon()
    }

    /// 最后一个计息年度的天数
    #[getter]
    pub fn last_cp_year_days(&self) -> PyResult<i64> {
        Ok(self.0.get_last_cp_year_days()?)
    }

    /// 获取上一付息日和下一付息日
    pub fn nearest_cp_date(&self, date: &Bound<'_, PyAny>) -> PyResult<(NaiveDate, NaiveDate)> {
        let date = extract_date(date)?;
        Ok(self.0.get_nearest_cp_date(date)?)
    }

    /// 剩余的付息次数
    #[pyo3(signature = (date, next_cp_date=None))]
    pub fn remain_cp_num(
        &self,
        date: &Bound<'_, PyAny>,
        next_cp_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<i32> {
        let date = extract_date(date)?;
        let next_cp_date: Option<NaiveDate> = next_cp_date.map(extract_date).transpose()?;
        Ok(self.0.remain_cp_num(date, next_cp_date)?)
    }

    /// 剩余的付息次数直到指定日期
    #[pyo3(signature = (date, until_date, next_cp_date=None))]
    pub fn remain_cp_num_until(
        &self,
        date: &Bound<'_, PyAny>,
        until_date: &Bound<'_, PyAny>,
        next_cp_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<i32> {
        let date = extract_date(date)?;
        let until_date = extract_date(until_date)?;
        let next_cp_date: Option<NaiveDate> = next_cp_date.map(extract_date).transpose()?;
        Ok(self.0.remain_cp_num_until(date, until_date, next_cp_date)?)
    }

    /// 剩余的付息日期列表（不包含指定日期）
    #[pyo3(signature = (date, until_date, next_cp_date=None))]
    pub fn remain_cp_dates_until(
        &self,
        date: &Bound<'_, PyAny>,
        until_date: &Bound<'_, PyAny>,
        next_cp_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<NaiveDate>> {
        let date = extract_date(date)?;
        let until_date = extract_date(until_date)?;
        let next_cp_date: Option<NaiveDate> = next_cp_date.map(extract_date).transpose()?;
        Ok(self
            .0
            .remain_cp_dates_until(date, until_date, next_cp_date)?)
    }

    /// 计算应计利息
    ///
    /// 银行间和交易所的计算规则不同,银行间是算头不算尾,而交易所是算头又算尾
    #[pyo3(signature = (calculating_date, cp_dates=None))]
    pub fn calc_accrued_interest(
        &self,
        calculating_date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let calculating_date = extract_date(calculating_date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        Ok(self.0.calc_accrued_interest(calculating_date, cp_dates)?)
    }

    /// 通过ytm计算债券全价
    #[pyo3(signature = (ytm, date, cp_dates=None, remain_cp_num=None))]
    pub fn calc_dirty_price_with_ytm(
        &self,
        ytm: f64,
        date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
        remain_cp_num: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let date = extract_date(date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        let remain_cp_num: Option<i32> = remain_cp_num.map(|d| d.extract()).transpose()?;
        Ok(self
            .0
            .calc_dirty_price_with_ytm(ytm, date, cp_dates, remain_cp_num)?)
    }

    /// 通过ytm计算债券净价
    #[pyo3(signature = (ytm, date, cp_dates=None, remain_cp_num=None))]
    pub fn calc_clean_price_with_ytm(
        &self,
        ytm: f64,
        date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
        remain_cp_num: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let date = extract_date(date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        let remain_cp_num: Option<i32> = remain_cp_num.map(|d| d.extract()).transpose()?;
        Ok(self
            .0
            .calc_clean_price_with_ytm(ytm, date, cp_dates, remain_cp_num)?)
    }

    /// 通过债券全价计算ytm
    #[pyo3(signature = (dirty_price, date, cp_dates=None, remain_cp_num=None))]
    pub fn calc_ytm_with_price(
        &self,
        dirty_price: f64,
        date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
        remain_cp_num: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let date = extract_date(date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        let remain_cp_num: Option<i32> = remain_cp_num.map(|d| d.extract()).transpose()?;
        Ok(self
            .0
            .calc_ytm_with_price(dirty_price, date, cp_dates, remain_cp_num)?)
    }

    /// 计算麦考利久期
    #[pyo3(signature = (ytm, date, cp_dates=None, remain_cp_num=None))]
    pub fn calc_macaulay_duration(
        &self,
        ytm: f64,
        date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
        remain_cp_num: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let date = extract_date(date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        let remain_cp_num: Option<i32> = remain_cp_num.map(|d| d.extract()).transpose()?;
        Ok(self
            .0
            .calc_macaulay_duration(ytm, date, cp_dates, remain_cp_num)?)
    }

    /// 计算修正久期
    #[pyo3(signature = (ytm, date, cp_dates=None, remain_cp_num=None))]
    pub fn calc_duration(
        &self,
        ytm: f64,
        date: &Bound<'_, PyAny>,
        cp_dates: Option<&Bound<'_, PyAny>>,
        remain_cp_num: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let date = extract_date(date)?;
        let cp_dates: Option<(NaiveDate, NaiveDate)> = cp_dates.map(extract_date2).transpose()?;
        let remain_cp_num: Option<i32> = remain_cp_num.map(|d| d.extract()).transpose()?;
        Ok(self.0.calc_duration(ytm, date, cp_dates, remain_cp_num)?)
    }
}
