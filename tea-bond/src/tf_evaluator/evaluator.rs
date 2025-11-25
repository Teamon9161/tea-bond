use crate::{
    bond::BondYtm,
    day_counter::{ACTUAL, DayCountRule},
    future::{FuturePrice, calc_cf},
    utils::month_delta,
};
use anyhow::{Result, ensure};
use chrono::NaiveDate;

#[derive(Clone, smart_default::SmartDefault)]
pub struct TfEvaluator {
    // 传入的字段
    pub date: NaiveDate,
    pub future: FuturePrice,
    pub bond: BondYtm,
    #[default(f64::NAN)]
    pub capital_rate: f64,
    pub reinvest_rate: Option<f64>, // 再投资收益率

    // 需要计算的字段
    pub accrued_interest: Option<f64>,            // 应计利息
    pub deliver_accrued_interest: Option<f64>,    // 国债期货交割应计利息
    pub cf: Option<f64>,                          // 转换因子
    pub dirty_price: Option<f64>,                 // 债券全价
    pub clean_price: Option<f64>,                 // 债券净价
    pub future_dirty_price: Option<f64>,          // 期货全价
    pub deliver_cost: Option<f64>,                // 交割成本
    pub basis_spread: Option<f64>,                // 基差
    pub f_b_spread: Option<f64>,                  // 期现价差
    pub carry: Option<f64>,                       // 持有收益
    pub net_basis_spread: Option<f64>,            // 净基差
    pub duration: Option<f64>,                    // 修正久期
    pub irr: Option<f64>,                         // 内部收益率
    pub deliver_date: Option<NaiveDate>,          // 期货配对缴款日
    pub cp_dates: Option<(NaiveDate, NaiveDate)>, // 前一付息日和下一付息日
    pub deliver_cp_dates: Option<(NaiveDate, NaiveDate)>, // 交割日的前一付息日和下一付息日
    pub remain_cp_num: Option<i32>,               // 债券剩余付息次数
    pub remain_days_to_deliver: Option<i32>,      // 到交割的剩余天数
    pub remain_cp_to_deliver: Option<f64>,        // 到交割的期间付息
    pub remain_cp_to_deliver_wm: Option<f64>,     // 加权平均到交割的期间付息
    pub future_ytm: Option<f64>,                  // 推断的期货收益率
}

impl TfEvaluator {
    #[inline]
    pub fn new<B: TryInto<BondYtm>>(
        date: NaiveDate,
        future: impl Into<FuturePrice>,
        bond: B,
        capital_rate: f64,
    ) -> Self
    where
        B::Error: std::fmt::Debug,
    {
        Self {
            date,
            future: future.into(),
            bond: bond.try_into().unwrap(),
            capital_rate,
            ..Default::default()
        }
    }

    #[inline]
    pub fn new_with_reinvest_rate<B: TryInto<BondYtm>>(
        date: NaiveDate,
        future: impl Into<FuturePrice>,
        bond: B,
        capital_rate: f64,
        reinvest_rate: f64,
    ) -> Self
    where
        B::Error: std::fmt::Debug,
    {
        Self {
            date,
            future: future.into(),
            bond: bond.try_into().unwrap(),
            capital_rate,
            reinvest_rate: Some(reinvest_rate),
            ..Default::default()
        }
    }

    #[inline]
    /// 判断债券是否是期货的可交割券
    pub fn is_deliverable(&self) -> Result<bool> {
        self.future.is_deliverable(
            self.bond.carry_date,
            self.bond.maturity_date,
            self.deliver_date,
        )
    }

    #[inline]
    /// 计算期货配对缴款日
    pub fn with_deliver_date(mut self) -> Result<Self> {
        if self.deliver_date.is_none() {
            self.deliver_date = Some(self.future.deliver_date()?);
        }
        Ok(self)
    }

    /// 计算前一付息日和下一付息日
    #[inline]
    pub fn with_nearest_cp_dates(mut self) -> Result<Self> {
        if self.cp_dates.is_none() {
            let (pre_cp_date, next_cp_date) = self.bond.get_nearest_cp_date(self.date)?;
            self.cp_dates = Some((pre_cp_date, next_cp_date));
        }
        Ok(self)
    }

    /// 计算交割日的前一付息日和下一付息日
    #[inline]
    pub fn with_deliver_cp_dates(self) -> Result<Self> {
        if self.deliver_cp_dates.is_none() {
            let mut out = self.with_deliver_date()?;
            let (pre_cp_date, next_cp_date) =
                out.bond.get_nearest_cp_date(out.deliver_date.unwrap())?;
            out.deliver_cp_dates = Some((pre_cp_date, next_cp_date));
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算到交割的剩余天数
    #[inline]
    pub fn with_remain_days_to_deliver(self) -> Result<Self> {
        if self.remain_days_to_deliver.is_none() {
            let mut out = self.with_deliver_date()?;
            out.remain_days_to_deliver =
                Some(ACTUAL.count_days(out.date, out.deliver_date.unwrap()) as i32);
            Ok(out)
        } else {
            Ok(self)
        }
    }
    /// 计算剩余付息次数
    #[inline]
    pub fn with_remain_cp_num(self) -> Result<Self> {
        if self.remain_cp_num.is_none() {
            let mut out = self.with_nearest_cp_dates()?;
            out.remain_cp_num = Some(
                out.bond
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
            let mut out = self.with_nearest_cp_dates()?;
            out.accrued_interest = Some(
                out.bond
                    .calc_accrued_interest(out.date, Some(out.cp_dates.unwrap()))?,
            );
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算债券全价
    #[inline]
    pub fn with_dirty_price(self) -> Result<Self> {
        if self.dirty_price.is_none() {
            let mut out = self.with_remain_cp_num()?;
            out.dirty_price = Some(out.bond.calc_dirty_price_with_ytm(
                out.bond.ytm(),
                out.date,
                out.cp_dates,
                out.remain_cp_num,
            )?);
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算债券净价
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
            let mut out = self.with_remain_cp_num()?;
            out.duration = Some(out.bond.calc_duration(
                out.bond.ytm(),
                out.date,
                out.cp_dates,
                out.remain_cp_num,
            )?);
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算转换因子
    pub fn with_cf(self) -> Result<Self> {
        if self.cf.is_none() {
            let mut out = self.with_deliver_cp_dates()?;
            let deliver_date = out.deliver_date.unwrap(); // 交割日
            let (_deliver_pre_cp_date, deliver_next_cp_date) =
                out.bond.get_nearest_cp_date(deliver_date)?;
            let remain_cp_num_after_deliver = out
                .bond
                .remain_cp_num(deliver_date, Some(deliver_next_cp_date))?;
            let month_num_from_dlv2next_cp = month_delta(deliver_date, deliver_next_cp_date);
            out.cf = Some(calc_cf(
                remain_cp_num_after_deliver,
                out.bond.cp_rate,
                out.bond.inst_freq,
                month_num_from_dlv2next_cp,
                None,
            ));
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算基差
    ///
    /// 基差=现券净价-期货价格*转换因子
    #[inline]
    pub fn with_basis_spread(self) -> Result<Self> {
        if self.basis_spread.is_none() {
            let mut out = self.with_cf()?.with_clean_price()?;
            out.basis_spread = Some(out.clean_price.unwrap() - out.future.price * out.cf.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算国债期货交割应计利息
    ///
    /// 国债期货交割应计利息=区间付息* (国债期货交割缴款日 - 国债期货交割前一付息日) / (国债期货交割下一付息日 - 国债期货交割前一付息日)
    ///
    /// 按中金所发布公式, 计算结果四舍五入至小数点后7位
    pub fn with_deliver_accrued_interest(self) -> Result<Self> {
        if self.deliver_accrued_interest.is_none() {
            let mut out = self.with_deliver_cp_dates()?;
            let coupon = out.bond.get_coupon();
            let deliver_date = out.future.deliver_date()?; // 交割日
            let (deliver_pre_cp_date, deliver_next_cp_date) = out.deliver_cp_dates.unwrap();
            let deliver_accrued_interest = coupon
                * ACTUAL.count_days(deliver_pre_cp_date, deliver_date) as f64
                / ACTUAL.count_days(deliver_pre_cp_date, deliver_next_cp_date) as f64;
            out.deliver_accrued_interest = Some((deliver_accrued_interest * 1e7).round() / 1e7);
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算期货全价（发票价格)
    #[inline]
    pub fn with_future_dirty_price(self) -> Result<Self> {
        if self.future_dirty_price.is_none() {
            let mut out = self.with_cf()?.with_deliver_accrued_interest()?;
            out.future_dirty_price =
                Some(out.future.price * out.cf.unwrap() + out.deliver_accrued_interest.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算期间付息
    pub fn with_remain_cp_to_deliver(self) -> Result<Self> {
        if self.remain_cp_to_deliver.is_none() {
            let mut out = self.with_deliver_date()?.with_nearest_cp_dates()?;
            let deliver_date = out.deliver_date.unwrap();
            // 计算期间付息次数
            let n = out.bond.remain_cp_num_until(
                out.date,
                deliver_date,
                Some(out.cp_dates.unwrap().1),
            )?;
            if n != 0 {
                let coupon = out.bond.get_coupon();
                let remain_cp_dates = out.bond.remain_cp_dates_until(
                    out.date,
                    deliver_date,
                    Some(out.cp_dates.unwrap().1),
                )?;
                ensure!(remain_cp_dates.len() == n as usize, "implement error");
                out.remain_cp_to_deliver = Some(coupon * n as f64);
                // 加权平均期间付息,按每个付息日到结算日的年化剩余天数加权的实际付息
                out.remain_cp_to_deliver_wm = Some(
                    remain_cp_dates.into_iter().fold(0., |acc, d| {
                        acc + ACTUAL.count_days(d, deliver_date) as f64 / 365.
                    }) * coupon,
                );
            } else {
                out.remain_cp_to_deliver = Some(0.);
                out.remain_cp_to_deliver_wm = Some(0.);
            }
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算交割成本
    ///
    /// 交割成本=债券全价-期间付息
    pub fn with_deliver_cost(self) -> Result<Self> {
        if self.deliver_cost.is_none() {
            let mut out = self.with_dirty_price()?.with_remain_cp_to_deliver()?;
            out.deliver_cost = Some(out.dirty_price.unwrap() - out.remain_cp_to_deliver.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算期现价差
    ///
    /// 期现价差=期货全价(发票价格)-交割成本
    pub fn with_f_b_spread(self) -> Result<Self> {
        if self.f_b_spread.is_none() {
            let mut out = self.with_future_dirty_price()?.with_deliver_cost()?;
            out.f_b_spread = Some(out.future_dirty_price.unwrap() - out.deliver_cost.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算持有收益
    ///
    /// 持有收益 = (交割日应计-交易日应计 + 期间付息) + 资金成本率*(加权平均期间付息-债券全价*剩余天数/365)
    pub fn with_carry(self) -> Result<Self> {
        if self.carry.is_none() {
            let mut out = self
                .with_accrued_interest()?
                .with_dirty_price()?
                .with_deliver_accrued_interest()?
                .with_remain_days_to_deliver()?
                .with_remain_cp_to_deliver()?;
            let remain_days_to_deliver = out.remain_days_to_deliver.unwrap() as f64;
            let left_hand_side = out.deliver_accrued_interest.unwrap()
                - out.accrued_interest.unwrap()
                + out.remain_cp_to_deliver.unwrap();
            let right_hand_side = out.capital_rate
                * (out.remain_cp_to_deliver_wm.unwrap()
                    - out.dirty_price.unwrap() * remain_days_to_deliver / 365.);
            out.carry = Some(left_hand_side + right_hand_side);
            Ok(out)
        } else {
            Ok(self)
        }
    }
    /// 计算净基差
    ///
    /// 净基差=基差-持有收益
    pub fn with_net_basis_spread(self) -> Result<Self> {
        if self.net_basis_spread.is_none() {
            let mut out = self.with_basis_spread()?.with_carry()?;
            out.net_basis_spread = Some(out.basis_spread.unwrap() - out.carry.unwrap());
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算内部收益率IRR
    pub fn with_irr(self) -> Result<Self> {
        if self.irr.is_none() {
            let mut out = self
                .with_dirty_price()?
                .with_future_dirty_price()?
                .with_remain_days_to_deliver()?
                .with_remain_cp_to_deliver()?;
            if let Some(reinvest_rate) = out.reinvest_rate {
                // 如果定义了利息再投资利率则需要将使用加权平均期间付息乘以该再投资利率
                let irr = (out.future_dirty_price.unwrap()
                    + out.remain_cp_to_deliver.unwrap()
                    + out.remain_cp_to_deliver_wm.unwrap() * reinvest_rate)
                    / out.dirty_price.unwrap()
                    - 1.;
                let irr = irr * 365. / out.remain_days_to_deliver.unwrap() as f64;
                out.irr = Some(irr);
            } else {
                // QB: irr=(发票价格+期间付息-现券全价)/(现券全价*剩余天数/365-加权平均期间付息)
                out.irr = Some(
                    (out.future_dirty_price.unwrap() + out.remain_cp_to_deliver.unwrap()
                        - out.dirty_price.unwrap())
                        / (out.dirty_price.unwrap() * out.remain_days_to_deliver.unwrap() as f64
                            / 365.
                            - out.remain_cp_to_deliver_wm.unwrap()),
                );
            }
            Ok(out)
        } else {
            Ok(self)
        }
    }

    /// 计算期货隐含收益率
    pub fn with_future_ytm(self) -> Result<Self> {
        if self.future_ytm.is_none() {
            let mut out = self.with_cf()?.with_deliver_date()?;
            let deliver_date = out.deliver_date.unwrap();
            let accrued_interest = out.bond.calc_accrued_interest(deliver_date, None)?;
            let tmp_dirty_price = out.future.price * out.cf.unwrap() + accrued_interest;
            out.future_ytm =
                Some(
                    out.bond
                        .calc_ytm_with_price(tmp_dirty_price, deliver_date, None, None)?,
                );
            Ok(out)
        } else {
            Ok(self)
        }
    }

    #[inline]
    pub fn calc_all(self) -> Result<Self> {
        self.with_remain_cp_num()?
            .with_clean_price()?
            .with_duration()?
            .with_f_b_spread()?
            .with_net_basis_spread()?
            .with_irr()?
            .with_future_ytm()
    }
}
