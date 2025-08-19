use crate::utils::{extract_date, get_bond, get_future};
use chrono::NaiveDate;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tea_bond::*;

#[pyclass(name = "TfEvaluator", subclass)]
#[derive(Clone)]
pub struct PyTfEvaluator(pub TfEvaluator);

impl From<TfEvaluator> for PyTfEvaluator {
    #[inline]
    fn from(value: TfEvaluator) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PyTfEvaluator {
    #[new]
    #[pyo3(signature = (future, bond, date=None, future_price=f64::NAN, bond_ytm=f64::NAN, capital_rate=f64::NAN, reinvest_rate=None))]
    pub fn new(
        future: &Bound<'_, PyAny>,
        bond: &Bound<'_, PyAny>,
        date: Option<&Bound<'_, PyAny>>,
        future_price: f64,
        bond_ytm: f64,
        capital_rate: f64,
        reinvest_rate: Option<f64>,
    ) -> PyResult<Self> {
        let date = if let Some(date) = date {
            extract_date(date)?
        } else {
            Default::default()
        };
        let future = get_future(future)?;
        let bond = get_bond(bond)?;
        let future = FuturePrice {
            future: future.0,
            price: future_price,
        };
        let bond = BondYtm::new(bond.0, bond_ytm);
        Ok(Self(TfEvaluator {
            date,
            future,
            bond,
            capital_rate,
            reinvest_rate,
            ..Default::default()
        }))
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }

    #[getter]
    fn get_ptr(&self) -> usize {
        &self.0 as *const TfEvaluator as usize
    }

    #[staticmethod]
    pub fn from_ptr(ptr: usize) -> PyResult<Self> {
        // Convert the pointer back to a reference
        unsafe {
            let evaluator_ref = &*(ptr as *const TfEvaluator);
            Ok(Self(evaluator_ref.clone()))
        }
    }

    #[getter]
    /// 判断债券是否是期货的可交割券
    fn deliverable(&self) -> PyResult<bool> {
        self.0
            .is_deliverable()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[getter]
    fn date(&self) -> NaiveDate {
        self.0.date
    }

    #[getter]
    /// 获取债券代码
    fn bond_code(&self) -> String {
        self.0.bond.bond.code().into()
    }

    #[getter]
    /// 获取债券收益率
    fn bond_ytm(&self) -> f64 {
        self.0.bond.ytm()
    }

    #[getter]
    /// 获取期货代码
    fn future(&self) -> String {
        self.0.future.future.code.clone().into()
    }

    #[getter]
    /// 获取期货价格
    fn future_price(&self) -> f64 {
        self.0.future.price
    }

    /// 计算期货配对缴款日
    fn with_deliver_date(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_deliver_date()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算前一付息日和下一付息日
    fn with_nearest_cp_dates(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_nearest_cp_dates()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算交割日的前一付息日和下一付息日
    fn with_deliver_cp_dates(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_deliver_cp_dates()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算到交割的剩余天数
    fn with_remain_days_to_deliver(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_remain_days_to_deliver()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算剩余付息次数
    fn with_remain_cp_num(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_remain_cp_num()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算应计利息
    fn with_accrued_interest(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_accrued_interest()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算债券全价
    fn with_dirty_price(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_dirty_price()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算债券净价
    fn with_clean_price(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_clean_price()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算修正久期
    fn with_duration(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_duration()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算转换因子
    fn with_cf(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_cf()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算基差
    fn with_basis_spread(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_basis_spread()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算国债期货交割应计利息
    ///
    /// 国债期货交割应计利息=区间付息* (国债期货交割缴款日 - 国债期货交割前一付息日) / (国债期货交割下一付息日 - 国债期货交割前一付息日)
    ///
    /// 按中金所发布公式, 计算结果四舍五入至小数点后7位
    fn with_deliver_accrued_interest(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_deliver_accrued_interest()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算期货全价（发票价格)
    fn with_future_dirty_price(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_future_dirty_price()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算期间付息
    fn with_remain_cp_to_deliver(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_remain_cp_to_deliver()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算交割成本
    ///
    /// 交割成本=债券全价-期间付息
    fn with_deliver_cost(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_deliver_cost()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算期现价差
    fn with_f_b_spread(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_f_b_spread()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算持有收益
    ///
    /// 持有收益 = (交割日应计-交易日应计 + 期间付息) + 资金成本率*(加权平均期间付息-债券全价*剩余天数/365)
    fn with_carry(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_carry()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算净基差
    ///
    /// 净基差=基差-持有收益
    fn with_net_basis_spread(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_net_basis_spread()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算内部收益率IRR
    fn with_irr(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_irr()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算期货隐含收益率
    fn with_future_ytm(&self) -> PyResult<Self> {
        self.0
            .clone()
            .with_future_ytm()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    /// 计算所有指标
    fn calc_all(&self) -> PyResult<Self> {
        self.0
            .clone()
            .calc_all()
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(Self)
    }

    #[pyo3(signature = (future_price=None, bond_ytm=None, date=None, future=None, bond=None, capital_rate=None, reinvest_rate=None))]
    /// 根据新的日期、债券和期货信息更新评估器
    ///
    /// 此函数会根据输入的新信息更新评估器的各个字段，
    /// 并根据变化情况决定是否保留原有的计算结果。
    fn update(
        &self,
        future_price: Option<f64>,
        bond_ytm: Option<f64>,
        date: Option<&Bound<'_, PyAny>>,
        future: Option<&Bound<'_, PyAny>>,
        bond: Option<&Bound<'_, PyAny>>,
        capital_rate: Option<f64>,
        reinvest_rate: Option<f64>,
    ) -> PyResult<Self> {
        let date = if let Some(date) = date {
            extract_date(date)?
        } else {
            self.0.date
        };
        let future = if let Some(future) = future {
            get_future(future)?.0
        } else {
            self.0.future.future.clone()
        };
        let bond = if let Some(bond) = bond {
            get_bond(bond)?.0
        } else {
            self.0.bond.bond.clone()
        };
        let capital_rate = if let Some(capital_rate) = capital_rate {
            capital_rate
        } else {
            self.0.capital_rate
        };
        let future_price = future_price.unwrap_or(self.0.future.price);
        let bond_ytm = bond_ytm.unwrap_or(self.0.bond.ytm());
        Ok(Self(self.0.clone().update_with_new_info(
            date,
            (future, future_price),
            (bond, bond_ytm),
            capital_rate,
            reinvest_rate,
        )))
    }

    #[getter]
    /// 应计利息
    fn accrued_interest(&mut self) -> PyResult<f64> {
        if let Some(accrued_interest) = self.0.accrued_interest {
            Ok(accrued_interest)
        } else {
            self.0 = self.with_accrued_interest()?.0;
            Ok(self.0.accrued_interest.unwrap())
        }
    }

    #[getter]
    /// 国债期货交割应计利息
    fn deliver_accrued_interest(&mut self) -> PyResult<f64> {
        if let Some(deliver_accrued_interest) = self.0.deliver_accrued_interest {
            Ok(deliver_accrued_interest)
        } else {
            self.0 = self.with_deliver_accrued_interest()?.0;
            Ok(self.0.deliver_accrued_interest.unwrap())
        }
    }

    #[getter]
    /// 转换因子
    fn cf(&mut self) -> PyResult<f64> {
        if let Some(cf) = self.0.cf {
            Ok(cf)
        } else {
            self.0 = self.with_cf()?.0;
            Ok(self.0.cf.unwrap())
        }
    }

    #[getter]
    /// 债券全价
    fn dirty_price(&mut self) -> PyResult<f64> {
        if let Some(dirty_price) = self.0.dirty_price {
            Ok(dirty_price)
        } else {
            self.0 = self.with_dirty_price()?.0;
            Ok(self.0.dirty_price.unwrap())
        }
    }

    #[getter]
    /// 债券净价
    fn clean_price(&mut self) -> PyResult<f64> {
        if let Some(clean_price) = self.0.clean_price {
            Ok(clean_price)
        } else {
            self.0 = self.with_clean_price()?.0;
            Ok(self.0.clean_price.unwrap())
        }
    }

    #[getter]
    /// 期货全价
    fn future_dirty_price(&mut self) -> PyResult<f64> {
        if let Some(future_dirty_price) = self.0.future_dirty_price {
            Ok(future_dirty_price)
        } else {
            self.0 = self.with_future_dirty_price()?.0;
            Ok(self.0.future_dirty_price.unwrap())
        }
    }

    #[getter]
    /// 交割成本
    fn deliver_cost(&mut self) -> PyResult<f64> {
        if let Some(deliver_cost) = self.0.deliver_cost {
            Ok(deliver_cost)
        } else {
            self.0 = self.with_deliver_cost()?.0;
            Ok(self.0.deliver_cost.unwrap())
        }
    }

    #[getter]
    /// 基差
    fn basis_spread(&mut self) -> PyResult<f64> {
        if let Some(basis_spread) = self.0.basis_spread {
            Ok(basis_spread)
        } else {
            self.0 = self.with_basis_spread()?.0;
            Ok(self.0.basis_spread.unwrap())
        }
    }

    #[getter]
    /// 期现价差
    fn f_b_spread(&mut self) -> PyResult<f64> {
        if let Some(f_b_spread) = self.0.f_b_spread {
            Ok(f_b_spread)
        } else {
            self.0 = self.with_f_b_spread()?.0;
            Ok(self.0.f_b_spread.unwrap())
        }
    }

    #[getter]
    /// 持有收益
    fn carry(&mut self) -> PyResult<f64> {
        if let Some(carry) = self.0.carry {
            Ok(carry)
        } else {
            self.0 = self.with_carry()?.0;
            Ok(self.0.carry.unwrap())
        }
    }

    #[getter]
    /// 净基差
    fn net_basis_spread(&mut self) -> PyResult<f64> {
        if let Some(net_basis_spread) = self.0.net_basis_spread {
            Ok(net_basis_spread)
        } else {
            self.0 = self.with_net_basis_spread()?.0;
            Ok(self.0.net_basis_spread.unwrap())
        }
    }

    #[getter]
    /// 修正久期
    fn duration(&mut self) -> PyResult<f64> {
        if let Some(duration) = self.0.duration {
            Ok(duration)
        } else {
            self.0 = self.with_duration()?.0;
            Ok(self.0.duration.unwrap())
        }
    }

    #[getter]
    /// 内部收益率
    fn irr(&mut self) -> PyResult<f64> {
        if let Some(irr) = self.0.irr {
            Ok(irr)
        } else {
            self.0 = self.with_irr()?.0;
            Ok(self.0.irr.unwrap())
        }
    }

    #[getter]
    /// 期货配对缴款日
    fn deliver_date(&mut self) -> PyResult<NaiveDate> {
        if let Some(deliver_date) = self.0.deliver_date {
            Ok(deliver_date)
        } else {
            self.0 = self.with_deliver_date()?.0;
            Ok(self.0.deliver_date.unwrap())
        }
    }

    #[getter]
    /// 前一付息日和下一付息日
    fn cp_dates(&mut self) -> PyResult<(NaiveDate, NaiveDate)> {
        if let Some(cp_dates) = self.0.cp_dates {
            Ok(cp_dates)
        } else {
            self.0 = self.with_nearest_cp_dates()?.0;
            Ok(self.0.cp_dates.unwrap())
        }
    }

    #[getter]
    /// 期货交割日的前一付息日和下一付息日
    fn deliver_cp_dates(&mut self) -> PyResult<(NaiveDate, NaiveDate)> {
        if let Some(deliver_cp_dates) = self.0.deliver_cp_dates {
            Ok(deliver_cp_dates)
        } else {
            self.0 = self.with_deliver_cp_dates()?.0;
            Ok(self.0.deliver_cp_dates.unwrap())
        }
    }

    #[getter]
    /// 债券剩余付息次数
    fn remain_cp_num(&mut self) -> PyResult<i32> {
        if let Some(remain_cp_num) = self.0.remain_cp_num {
            Ok(remain_cp_num)
        } else {
            self.0 = self.with_remain_cp_num()?.0;
            Ok(self.0.remain_cp_num.unwrap())
        }
    }

    #[getter]
    /// 到交割的期间付息
    fn remain_cp_to_deliver(&mut self) -> PyResult<f64> {
        if let Some(remain_cp_to_deliver) = self.0.remain_cp_to_deliver {
            Ok(remain_cp_to_deliver)
        } else {
            self.0 = self.with_remain_cp_to_deliver()?.0;
            Ok(self.0.remain_cp_to_deliver.unwrap())
        }
    }

    #[getter]
    /// 距离交割日的天数
    fn remain_days_to_deliver(&mut self) -> PyResult<i32> {
        if let Some(remain_days_to_deliver) = self.0.remain_days_to_deliver {
            Ok(remain_days_to_deliver)
        } else {
            self.0 = self.with_remain_days_to_deliver()?.0;
            Ok(self.0.remain_days_to_deliver.unwrap())
        }
    }

    #[getter]
    /// 加权平均到交割的期间付息
    fn remain_cp_to_deliver_wm(&mut self) -> PyResult<f64> {
        if let Some(remain_cp_to_deliver_wm) = self.0.remain_cp_to_deliver_wm {
            Ok(remain_cp_to_deliver_wm)
        } else {
            self.0 = self.with_remain_cp_to_deliver()?.0;
            Ok(self.0.remain_cp_to_deliver_wm.unwrap())
        }
    }

    #[getter]
    /// 期货隐含收益率
    fn future_ytm(&mut self) -> PyResult<f64> {
        if let Some(future_ytm) = self.0.future_ytm {
            Ok(future_ytm)
        } else {
            self.0 = self.with_future_ytm()?.0;
            Ok(self.0.future_ytm.unwrap())
        }
    }

    /// Sets the conversion factor
    #[setter]
    fn set_cf(&mut self, cf: f64) -> PyResult<()> {
        self.0.cf = Some(cf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_irr() {
        TfEvaluator::new(
            NaiveDate::from_ymd_opt(2024, 11, 19).unwrap(),
            ("T2412", 106.675),
            (240013, 0.02085),
            0.018,
        )
        .with_irr()
        .unwrap();
    }
}
