mod bond_ytm;
mod cached_bond;
#[cfg(feature = "download")]
mod download;
mod enums;
mod impl_convert;
mod impl_traits;
mod io;
pub use bond_ytm::BondYtm;
pub use cached_bond::{CachedBond, free_bond_dict};
pub use enums::{BondDayCount, CouponType, InterestType, Market};

use crate::SmallStr;
use crate::day_counter::{ACTUAL, DayCountRule};
use anyhow::{Result, bail, ensure};
use chrono::{Datelike, Months, NaiveDate};
use impl_traits::{deserialize_date, serialize_date};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bond {
    pub bond_code: SmallStr,         // 债券代码
    pub mkt: Market,                 // 市场
    pub abbr: SmallStr,              // 债券简称
    pub par_value: f64,              // 债券面值
    pub cp_type: CouponType,         // 息票品种
    pub interest_type: InterestType, // 息票利率类型
    #[serde(alias = "cp_rate_1st")]
    pub cp_rate: f64, // 票面利率, 浮动付息债券仅表示发行时票面利率
    pub base_rate: Option<f64>,      // 基准利率, 浮动付息债券适用
    pub rate_spread: Option<f64>,    // 固定利差, 浮动付息债券适用
    pub inst_freq: i32,              // 年付息次数
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub carry_date: NaiveDate, // 起息日
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub maturity_date: NaiveDate, // 到期日
    pub day_count: BondDayCount,     // 计息基准, 如A/365F
}

impl Bond {
    pub(crate) fn check_ytm(&self, ytm: f64) -> f64 {
        if ytm > 1.0 {
            eprintln!(
                "Warning: Bond: {}, YTM exceeds 100%, dividing by 100 by default.",
                self.bond_code()
            );
            ytm * 0.01
        } else {
            ytm
        }
    }

    #[inline]
    /// 债券代码，不包含交易所后缀
    pub fn code(&self) -> &str {
        if let Some(code) = self.bond_code.split_once('.') {
            code.0
        } else {
            &self.bond_code
        }
    }

    #[inline]
    /// 债券代码，包含交易所后缀
    pub fn bond_code(&self) -> &str {
        &self.bond_code
    }

    #[inline]
    /// 是否为零息债券
    pub fn is_zero_coupon(&self) -> bool {
        self.cp_type == CouponType::ZeroCoupon
    }

    #[inline]
    pub fn remain_year(&self, date: NaiveDate) -> f64 {
        let year_diff = self.maturity_date.year() - date.year();
        let month_diff = self.maturity_date.month() as i32 - date.month() as i32;
        let date_diff = self.maturity_date.day() as i32 - date.day() as i32;
        year_diff as f64 + month_diff as f64 / 12.0 + date_diff as f64 / 365.0
    }

    #[inline]
    pub fn issue_year(&self) -> i32 {
        self.maturity_date.year() - self.carry_date.year()
    }

    // #[inline]
    // /// 确保不是零息债券
    // fn ensure_not_zero_coupon(&self) -> Result<()> {
    //     if self.is_zero_coupon() {
    //         bail!("Zero Coupon bond does not have coupon dates");
    //     }
    //     Ok(())
    // }

    #[inline]
    /// 获取区间付息（单个付息周期的利息金额）
    ///
    /// 区间付息 = 票面利率 * 面值 / 年付息次数
    pub fn get_coupon(&self) -> f64 {
        self.cp_rate * self.par_value / self.inst_freq as f64
    }

    /// 获得付息间隔
    #[inline]
    pub fn get_cp_offset(&self) -> Result<Months> {
        match self.inst_freq {
            0 => Ok(Months::new(0)),
            1 => Ok(Months::new(12)),
            2 => Ok(Months::new(6)),
            _ => bail!("Invalid inst_freq: {}", self.inst_freq),
        }
    }

    /// 最后一个计息年度的天数
    pub fn get_last_cp_year_days(&self) -> Result<i64> {
        let offset = self.get_cp_offset()?;
        let mut cp_date = self.maturity_date - offset;
        while cp_date.year() == self.maturity_date.year() {
            cp_date = cp_date - self.get_cp_offset()?;
        }
        let mut day_counts = ACTUAL.count_days(cp_date, self.maturity_date);
        while day_counts < 360 {
            // 小于360天说明是一年多次付息的情况,排除该付息日继续向前找
            cp_date = cp_date - offset;
            day_counts = ACTUAL.count_days(cp_date, self.maturity_date);
        }
        ensure!(
            day_counts < 380,
            "Last coupon year days is too long: {}",
            day_counts
        );
        Ok(day_counts)
    }

    #[inline]
    /// 确保日期在有效范围内
    fn ensure_date_valid(&self, date: NaiveDate) -> Result<NaiveDate> {
        if date < self.carry_date {
            eprintln!(
                "Calculating date {} is before the bond {} 's carry date {}, adjust date to carry date",
                date,
                self.code(),
                self.carry_date
            );
            return Ok(self.carry_date);
        } else if date >= self.maturity_date {
            eprintln!(
                "Calculating date {} is after the bond {} 's maturity date {}, the result may be incorrect",
                date,
                self.code(),
                self.maturity_date
            );
            return Ok(self.maturity_date);
        }
        Ok(date)
    }

    /// 获取上一付息日和下一付息日
    pub fn get_nearest_cp_date(&self, date: NaiveDate) -> Result<(NaiveDate, NaiveDate)> {
        if self.is_zero_coupon() {
            bail!("Zero Coupon bond does not have coupon dates");
        }
        let date = self.ensure_date_valid(date)?;
        let date_offset = self.get_cp_offset()?;
        let mut cp_date = self.carry_date;
        let mut cp_date_next = cp_date + date_offset;
        // 最多一年付息两次，目前超长期国债为50年，理论上不会超过200次循环
        for _ in 0..220 {
            if date >= cp_date && date < cp_date_next {
                return Ok((cp_date, cp_date_next));
            }
            cp_date = cp_date_next;
            cp_date_next = cp_date + date_offset;
        }
        bail!("Failed to find nearest coupon date");
    }

    /// 剩余的付息次数
    pub fn remain_cp_num(&self, date: NaiveDate, next_cp_date: Option<NaiveDate>) -> Result<i32> {
        use tea_calendar::Calendar;
        let mut next_cp_date =
            next_cp_date.unwrap_or_else(|| self.get_nearest_cp_date(date).unwrap().1);
        let mut cp_num = 0;
        let offset = self.get_cp_offset()?;
        // TODO: 数据是否确实存在到期日不同于发行日的情况？如果存在，是后延还是提前？
        // 当下一付息日正好等于到期日时，目前正好返回1，也是正确的
        // let maturity_date = self.maturity_date + Duration::days(3); // 减去3天避免节假日导致的计算偏差
        let maturity_date = self.mkt.find_workday(self.maturity_date, 0);
        while next_cp_date <= maturity_date {
            cp_num += 1;
            next_cp_date = next_cp_date + offset;
        }
        Ok(cp_num)
    }

    /// 剩余的付息次数
    pub fn remain_cp_num_until(
        &self,
        date: NaiveDate,
        until_date: NaiveDate,
        next_cp_date: Option<NaiveDate>,
    ) -> Result<i32> {
        let mut next_cp_date =
            next_cp_date.unwrap_or_else(|| self.get_nearest_cp_date(date).unwrap().1);
        if next_cp_date > until_date {
            // 对于正好相等的情况，由于应计利息会被重置为0，因此剩余付息次数不应该返回0
            // 否则计算的持有期收益将会不连续
            // 此处与原python代码处理不同，当期货缴款日正好是付息日时，按1处理
            return Ok(0);
        }
        let mut cp_num = 0;
        let offset = self.get_cp_offset()?;
        while next_cp_date <= until_date {
            cp_num += 1;
            next_cp_date = next_cp_date + offset;
        }
        Ok(cp_num)
    }

    /// 获得剩余的付息日期列表(不包含until_date)
    pub fn remain_cp_dates_until(
        &self,
        date: NaiveDate,
        until_date: NaiveDate,
        next_cp_date: Option<NaiveDate>,
    ) -> Result<Vec<NaiveDate>> {
        let mut next_cp_date =
            next_cp_date.unwrap_or_else(|| self.get_nearest_cp_date(date).unwrap().1);
        if next_cp_date >= until_date {
            return Ok(vec![]);
        }
        let mut cp_dates = vec![];
        let offset = self.get_cp_offset()?;
        while next_cp_date < until_date {
            cp_dates.push(next_cp_date);
            next_cp_date = next_cp_date + offset;
        }
        Ok(cp_dates)
    }
    /// 计算应计利息
    ///
    /// 银行间和交易所的计算规则不同,银行间是算头不算尾,而交易所是算头又算尾
    pub fn calc_accrued_interest(
        &self,
        calculating_date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>, // 前后付息日，如果已经计算完成可以直接传入避免重复计算
    ) -> Result<f64> {
        if self.is_zero_coupon() {
            return Ok(0.0);
        }
        let (pre_cp_date, next_cp_date) = if let Some(cp_dates) = cp_dates {
            cp_dates
        } else {
            self.get_nearest_cp_date(calculating_date)?
        };
        match self.mkt {
            Market::IB => {
                // 银行间是算头不算尾，计算实际天数（自然日）
                let inst_accrued_days = ACTUAL.count_days(pre_cp_date, calculating_date);
                let coupon = self.cp_rate * self.par_value / self.inst_freq as f64;
                // 当前付息周期实际天数
                let present_cp_period_days = ACTUAL.count_days(pre_cp_date, next_cp_date);
                Ok(coupon * inst_accrued_days as f64 / present_cp_period_days as f64)
            }
            Market::SH | Market::SSE | Market::SZE | Market::SZ => {
                // 交易所是算头又算尾
                let inst_accrued_days = 1 + ACTUAL.count_days(pre_cp_date, calculating_date);
                Ok(self.cp_rate * self.par_value * inst_accrued_days as f64 / 365.0)
            }
        }
    }

    /// 通过ytm计算债券全价
    pub fn calc_dirty_price_with_ytm(
        &self,
        ytm: f64,
        date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>,
        remain_cp_num: Option<i32>,
    ) -> Result<f64> {
        let ytm = self.check_ytm(ytm);
        let inst_freq = self.inst_freq as f64;
        let coupon = self.get_coupon();
        let (pre_cp_date, next_cp_date) = if let Some(cp_dates) = cp_dates {
            cp_dates
        } else {
            self.get_nearest_cp_date(date)?
        };
        let remain_days = ACTUAL.count_days(date, next_cp_date) as f64;
        let n = remain_cp_num.unwrap_or_else(|| self.remain_cp_num(date, None).unwrap());
        // TODO: take day_count into account
        if n <= 1 {
            let ty = self.get_last_cp_year_days()? as f64;
            let forward_value = self.par_value + coupon;
            let discount_factor = 1.0 + ytm * remain_days / ty;
            Ok(forward_value / discount_factor)
        } else {
            let ty = ACTUAL.count_days(pre_cp_date, next_cp_date) as f64;
            let coupon_cf = (0..n).fold(0., |acc, i| {
                let discount_factor = (1. + ytm / inst_freq).powf(remain_days / ty + i as f64);
                acc + coupon / discount_factor
            });
            let discount_factor = (1. + ytm / inst_freq).powf(remain_days / ty + (n - 1) as f64);
            Ok(self.par_value / discount_factor + coupon_cf)
        }
    }

    /// 通过ytm计算债券净价
    pub fn calc_clean_price_with_ytm(
        &self,
        ytm: f64,
        date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>,
        remain_cp_num: Option<i32>,
    ) -> Result<f64> {
        let cp_dates = if let Some(cd) = cp_dates {
            cd
        } else {
            self.get_nearest_cp_date(date)?
        };
        let remain_cp_num =
            remain_cp_num.unwrap_or_else(|| self.remain_cp_num(date, None).unwrap());
        let dirty_price =
            self.calc_dirty_price_with_ytm(ytm, date, Some(cp_dates), Some(remain_cp_num))?;
        let accrued_interest = self.calc_accrued_interest(date, Some(cp_dates))?;
        Ok(dirty_price - accrued_interest)
    }

    /// 通过债券全价计算ytm
    pub fn calc_ytm_with_price(
        &self,
        dirty_price: f64,
        date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>,
        remain_cp_num: Option<i32>,
    ) -> Result<f64> {
        match self.interest_type {
            InterestType::Fixed => {
                let inst_freq = self.inst_freq as f64;
                let coupon = self.get_coupon();
                let (pre_cp_date, next_cp_date) =
                    cp_dates.unwrap_or_else(|| self.get_nearest_cp_date(date).unwrap());
                let remain_days = ACTUAL.count_days(date, next_cp_date) as f64;

                let n = remain_cp_num.unwrap_or_else(|| self.remain_cp_num(date, None).unwrap());
                if n > 1 {
                    let ty = ACTUAL.count_days(pre_cp_date, next_cp_date) as f64;
                    // 不在最后一个付息周期内
                    use crate::utils::bisection_find_ytm;
                    let f = |ytm: f64| {
                        let coupon_cf = (0..n).fold(0., |acc, i| {
                            let discount_factor =
                                (1. + ytm / inst_freq).powf(remain_days / ty + i as f64);
                            acc + coupon / discount_factor
                        });
                        let discount_factor =
                            (1. + ytm / inst_freq).powf(remain_days / ty + (n - 1) as f64);
                        self.par_value / discount_factor + coupon_cf - dirty_price
                    };
                    Ok(bisection_find_ytm(f, 1e-4, 0.3, Some(12)))
                } else {
                    let ty = self.get_last_cp_year_days()? as f64;
                    // 只剩最后一次付息
                    let forward_value = self.par_value + coupon;
                    Ok((forward_value - dirty_price) / dirty_price / (remain_days / ty))
                }
            }
            _ => bail!("Unsupported interest type: {:?}", self.interest_type),
        }
    }

    /// 麦考利久期
    pub fn calc_macaulay_duration(
        &self,
        ytm: f64,
        date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>,
        remain_cp_num: Option<i32>,
    ) -> Result<f64> {
        let ytm = self.check_ytm(ytm);
        let inst_freq = self.inst_freq as f64;
        let coupon = self.get_coupon();
        let (pre_cp_date, next_cp_date) =
            cp_dates.unwrap_or_else(|| self.get_nearest_cp_date(date).unwrap());
        let remain_days = ACTUAL.count_days(date, next_cp_date) as f64;
        let ty = ACTUAL.count_days(pre_cp_date, next_cp_date) as f64;
        let n = remain_cp_num.unwrap_or_else(|| self.remain_cp_num(date, None).unwrap());
        let cashflow = (0..n)
            .map(|i| {
                let discount_factor = (1. + ytm / inst_freq).powf(remain_days / ty + i as f64);
                let cashflow = coupon / discount_factor;
                let time = remain_days / 365. + i as f64 / inst_freq;
                (cashflow, time)
            })
            .chain(std::iter::once((
                self.par_value / (1. + ytm / inst_freq).powf(remain_days / ty + (n - 1) as f64),
                remain_days / 365. + (n - 1) as f64 / inst_freq,
            )))
            .collect::<Vec<_>>();
        let p = cashflow.iter().map(|(cf, _t)| cf).sum::<f64>();
        let duration = cashflow.iter().map(|(cf, t)| cf * t).sum::<f64>() / p;
        Ok(duration)
    }

    #[inline]
    /// 修正久期
    pub fn calc_duration(
        &self,
        ytm: f64,
        date: NaiveDate,
        cp_dates: Option<(NaiveDate, NaiveDate)>,
        remain_cp_num: Option<i32>,
    ) -> Result<f64> {
        let ytm = self.check_ytm(ytm);
        let duration = self.calc_macaulay_duration(ytm, date, cp_dates, remain_cp_num)?;
        Ok(duration / (1. + ytm / self.inst_freq as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_attributes() {
        let json_str = r#"
        {
            "bond_code": "240012.IB",
            "mkt": "IB",
            "abbr": "24附息国债12",
            "par_value": 100.0,
            "cp_type": "Coupon_Bear",
            "interest_type": "Fixed",
            "cp_rate_1st": 0.0167,
            "base_rate": null,
            "rate_spread": null,
            "inst_freq": 1,
            "carry_date": "2024-06-15",
            "maturity_date": "2026-06-15",
            "day_count": "ACT/ACT"
        }
        "#;

        let bond_attributes: Bond = serde_json::from_str(json_str).unwrap();
        println!("{:?}", bond_attributes);
    }

    #[test]
    fn test_nearest_cp_date() {
        // test bond with annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };
        let date = NaiveDate::from_ymd_opt(2018, 3, 15).unwrap();
        let (pre_cp_date, next_cp_date) = bond.get_nearest_cp_date(date).unwrap();
        assert_eq!(pre_cp_date, NaiveDate::from_ymd_opt(2017, 6, 15).unwrap());
        assert_eq!(next_cp_date, NaiveDate::from_ymd_opt(2018, 6, 15).unwrap());
        let date = NaiveDate::from_ymd_opt(2018, 6, 15).unwrap();
        let (pre_cp_date, next_cp_date) = bond.get_nearest_cp_date(date).unwrap();
        assert_eq!(pre_cp_date, NaiveDate::from_ymd_opt(2018, 6, 15).unwrap());
        assert_eq!(next_cp_date, NaiveDate::from_ymd_opt(2019, 6, 15).unwrap());

        // test bond with semi-annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 2,
            ..Default::default()
        };
        let date = NaiveDate::from_ymd_opt(2018, 9, 15).unwrap();
        let (pre_cp_date, next_cp_date) = bond.get_nearest_cp_date(date).unwrap();
        assert_eq!(pre_cp_date, NaiveDate::from_ymd_opt(2018, 6, 15).unwrap());
        assert_eq!(next_cp_date, NaiveDate::from_ymd_opt(2018, 12, 15).unwrap());
        let date = NaiveDate::from_ymd_opt(2019, 3, 15).unwrap();
        let (pre_cp_date, next_cp_date) = bond.get_nearest_cp_date(date).unwrap();
        assert_eq!(pre_cp_date, NaiveDate::from_ymd_opt(2018, 12, 15).unwrap());
        assert_eq!(next_cp_date, NaiveDate::from_ymd_opt(2019, 6, 15).unwrap());
    }

    #[test]
    fn test_remain_cp_num() {
        // test bond with annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };
        let date = NaiveDate::from_ymd_opt(2018, 3, 15).unwrap();
        let remain_cp_num = bond.remain_cp_num(date, None).unwrap();
        assert_eq!(remain_cp_num, 7);

        // test bond with semi-annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 2,
            ..Default::default()
        };
        let date = NaiveDate::from_ymd_opt(2018, 9, 15).unwrap();
        let remain_cp_num = bond.remain_cp_num(date, None).unwrap();
        assert_eq!(remain_cp_num, 12);
    }

    #[test]
    fn test_get_last_cp_year_days() {
        // test bond with annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };
        let last_cp_year_days = bond.get_last_cp_year_days().unwrap();
        assert_eq!(last_cp_year_days, 366);

        // test bond with semi-annual coupon
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            inst_freq: 2,
            ..Default::default()
        };
        let last_cp_year_days = bond.get_last_cp_year_days().unwrap();
        assert_eq!(last_cp_year_days, 366);

        // test bond with annual coupon, non-leap year, maturity date January 18
        let bond = Bond {
            carry_date: NaiveDate::from_ymd_opt(2014, 1, 18).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2023, 1, 18).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };
        let last_cp_year_days = bond.get_last_cp_year_days().unwrap();
        assert_eq!(last_cp_year_days, 365);
    }
}
