mod bond;
pub mod day_counter;
mod future;
mod utils;

use anyhow::Result;
pub use bond::{Bond, CouponType, InterestType, Market};
use chrono::NaiveDate;
pub use future::{Future, FutureType};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FuturePrice {
    pub future: Arc<Future>,
    pub price: f64,
}

impl Default for FuturePrice {
    fn default() -> Self {
        FuturePrice {
            future: Arc::new(Future::default()),
            price: f64::NAN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BondYtm {
    pub bond: Arc<Bond>,
    pub ytm: f64,
}

impl Default for BondYtm {
    fn default() -> Self {
        BondYtm {
            bond: Arc::new(Bond::default()),
            ytm: f64::NAN,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TfEvaluator {
    pub date: NaiveDate,
    pub future: FuturePrice,
    pub bond: BondYtm,
    pub capital_rate: f64,
    pub reinvest_rate: f64, // 再投资收益率

    // 需要计算的字段
    pub accrued_interest: Option<f64>,            // 应计利息
    pub cf: Option<f64>,                          // 转换因子
    pub dirty_price: Option<f64>,                 // 债券全价
    pub clean_price: Option<f64>,                 // 债券净价
    pub duration: Option<f64>,                    // 修正久期
    pub irr: Option<f64>,                         // 内部收益率
    pub cp_dates: Option<(NaiveDate, NaiveDate)>, // 前一付息日和下一付息日
    pub remain_cp_times: Option<i32>,             // 剩余付息次数
}

impl TfEvaluator {
    /// 计算前一付息日和下一付息日
    #[inline]
    pub fn with_nearest_cp_date(mut self) -> Result<Self> {
        if self.cp_dates.is_none() {
            let bond = &self.bond.bond;
            let (pre_cp_date, next_cp_date) = bond.get_nearest_cp_date(self.date)?;
            self.cp_dates = Some((pre_cp_date, next_cp_date));
        }
        Ok(self)
    }

    /// 计算剩余付息次数
    #[inline]
    pub fn with_remain_cp_times(self) -> Result<Self> {
        if self.remain_cp_times.is_none() {
            let mut out = self.with_nearest_cp_date()?;
            out.remain_cp_times = Some(
                out.bond
                    .bond
                    .remain_cp_num(out.date, Some(out.cp_dates.unwrap().1))?,
            );
            Ok(out)
        } else {
            Ok(self)
        }
    }
    /// 计算应计利息
    #[inline]
    pub fn with_accrued_interest(self) -> Result<Self> {
        if self.accrued_interest.is_none() {
            let mut out = self.with_nearest_cp_date()?;
            out.accrued_interest = Some(
                out.bond
                    .bond
                    .calc_accrued_interest(out.date, Some(out.cp_dates.unwrap()))?,
            );
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算全价
    #[inline]
    pub fn with_dirty_price(self) -> Result<Self> {
        if self.dirty_price.is_none() {
            let mut out = self.with_remain_cp_times()?;
            out.dirty_price = Some(out.bond.bond.calc_dirty_price_with_ytm(
                out.bond.ytm,
                out.date,
                None,
                out.remain_cp_times,
            )?);
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算净价
    #[inline]
    pub fn with_clean_price(self) -> Result<Self> {
        if self.clean_price.is_none() {
            let mut out = self.with_dirty_price()?.with_accrued_interest()?;
            out.clean_price = Some(out.dirty_price.unwrap() - out.accrued_interest.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算久期
    #[inline]
    pub fn with_duration(self) -> Result<Self> {
        if self.duration.is_none() {
            let mut out = self.with_dirty_price()?.with_accrued_interest()?;
            out.duration = Some(out.bond.bond.calc_duration(
                out.bond.ytm,
                out.date,
                Some(out.cp_dates.unwrap().1),
            )?);
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算转换因子
    pub fn with_cf(self) -> Result<Self> {
        if self.cf.is_none() {
            let mut out = self.with_remain_cp_times()?;
            let remain_cp_times = out
                .bond
                .bond
                .remain_cp_num(out.date, Some(out.cp_dates.unwrap().1))?;
            let deliver_date = out.future.future.paydate()?; // 交割日
            let (_pre_deliver_cp_date, next_deliver_cp_date) =
                out.bond.bond.get_nearest_cp_date(deliver_date)?;
            let month_num_from_dlv2next_cp = utils::month_delta(deliver_date, next_deliver_cp_date);
            out.cf = Some(future::calc_cf(
                remain_cp_times,
                out.bond.bond.cp_rate_1st,
                out.bond.bond.inst_freq,
                month_num_from_dlv2next_cp,
                None,
            ));
            Ok(out)
        } else {
            Ok(self)
        }
    }

    pub fn calc_all(self) -> Result<Self> {
        self.with_remain_cp_times()?
            .with_dirty_price()?
            .with_clean_price()?
            .with_duration()?
            .with_cf()
    }
}
