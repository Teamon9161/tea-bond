use std::fmt::Debug;

use crate::{BondYtm, FuturePrice, TfEvaluator};
// use anyhow::Error;
use chrono::NaiveDate;

impl TfEvaluator {
    /// 根据新的日期、债券和期货信息更新评估器
    ///
    /// 此函数会根据输入的新信息更新评估器的各个字段，
    /// 并根据变化情况决定是否保留原有的计算结果。
    #[inline]
    pub fn update_with_new_info<B: TryInto<BondYtm>>(
        self,
        date: NaiveDate,
        future: impl Into<FuturePrice>,
        bond: B,
        capital_rate: f64,
        reinvest_rate: Option<f64>,
    ) -> Self
    where
        B::Error: Debug,
    {
        self.update_with_new_info_impl(
            date,
            future.into(),
            bond.try_into().unwrap(),
            capital_rate,
            reinvest_rate,
        )
    }

    /// 根据新的日期、债券和期货信息更新评估器
    ///
    /// 此函数会根据输入的新信息更新评估器的各个字段，
    /// 并根据变化情况决定是否保留原有的计算结果。
    fn update_with_new_info_impl(
        self,
        date: NaiveDate,
        future: FuturePrice,
        bond: BondYtm,
        capital_rate: f64,
        reinvest_rate: Option<f64>,
    ) -> Self {
        // 检查期货、债券和日期是否发生变化
        let future_chg = future.future != self.future.future;
        let bond_chg = bond.bond != self.bond.bond;
        let date_chg = date != self.date;

        // 如果期货不变，则配对缴款日不变
        let deliver_date = if !future_chg { self.deliver_date } else { None };

        // 如果债券和日期不变，则前一付息日和下一付息日不变
        let cp_dates = if !bond_chg && !date_chg {
            self.cp_dates
        } else {
            None
        };

        // 如果债券、期货不变，则交割日的前一付息日和下一付息日不变
        let deliver_cp_dates = if !bond_chg && !future_chg {
            self.deliver_cp_dates
        } else {
            None
        };

        // 如果债券和日期不变，则剩余付息次数不变
        let remain_cp_num = if !bond_chg && !date_chg {
            self.remain_cp_num
        } else {
            None
        };

        // 如果期货和日期不变，则到交割的剩余天数不变
        let remain_days_to_deliver = if !future_chg && !date_chg {
            self.remain_days_to_deliver
        } else {
            None
        };

        // 如果债券、期货和日期不变，则到交割的期间付息和加权期间付息不变
        let (remain_cp_to_deliver, remain_cp_to_deliver_wm) =
            if !bond_chg && !future_chg && !date_chg {
                (self.remain_cp_to_deliver, self.remain_cp_to_deliver_wm)
            } else {
                (None, None)
            };

        // 如果债券和日期不变，则应计利息不变
        let accrued_interest = if !bond_chg && !date_chg {
            self.accrued_interest
        } else {
            None
        };

        // 如果债券、日期和债券ytm不变，则全价不变
        let dirty_price = if self.bond == bond && !date_chg {
            self.dirty_price
        } else {
            None
        };

        // 如果债券、日期、债券ytm不变，则净价不变
        let clean_price = if self.bond == bond && !date_chg {
            self.clean_price
        } else {
            None
        };

        // 如果债券、日期和债券ytm不变，则久期不变
        let duration = if self.bond == bond && !date_chg {
            self.duration
        } else {
            None
        };

        // 如果债券和期货不变，则转换因子不变
        let cf = if !bond_chg && !future_chg {
            self.cf
        } else {
            None
        };

        // 如果债券和期货不变，则交割的应计利息不变
        let deliver_accrued_interest = if !bond_chg && !future_chg {
            self.deliver_accrued_interest
        } else {
            None
        };

        // 如果债券和期货不变且期货的价格不变，则期货全价不变
        let future_dirty_price = if !bond_chg && !future_chg && future.price == self.future.price {
            self.future_dirty_price
        } else {
            None
        };

        // 返回更新后的评估器
        TfEvaluator {
            date,
            future,
            bond,
            capital_rate,
            reinvest_rate,

            accrued_interest,
            deliver_accrued_interest,
            cf,
            dirty_price,
            clean_price,
            future_dirty_price,
            deliver_date,
            cp_dates,
            deliver_cp_dates,
            remain_cp_num,
            remain_days_to_deliver,
            remain_cp_to_deliver,
            remain_cp_to_deliver_wm,
            duration,
            ..Default::default()
        }
    }
}
