use anyhow::{bail, Result};
use chrono::{Datelike, Months, NaiveDate};
use serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum CouponType {
    /// 附息债券
    #[serde(alias = "Coupon_Bear")]
    CouponBear,
    /// 零息债券
    #[serde(alias = "Zero_Coupon")]
    ZeroCoupon,
    /// 一次性付息
    #[serde(alias = "One_Time")]
    OneTime,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum InterestType {
    /// 固定利率
    Fixed,
    /// 浮动利率
    Floating,
    /// 累进利率
    Progressive,
    /// 零息
    Zero,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Market {
    /// 银行间
    IB,
    /// 上交所
    SSE,
    /// 上交所（同义词）
    SH,
    /// 深交所
    SZE,
    /// 深交所（同义词）
    SZ,
}

#[derive(Debug, Deserialize)]
pub struct Bond {
    pub bond_code: String,           // 债券代码
    pub mkt: Market,                 // 市场
    pub abbr: String,                // 债券简称
    pub par_value: f64,              // 债券面值
    pub cp_type: CouponType,         // 息票品种
    pub interest_type: InterestType, // 息票利率类型
    pub cp_rate_1st: f64,            // 票面利率, 浮动付息债券仅表示发行时票面利率
    pub base_rate: Option<f64>,      // 基准利率, 浮动付息债券适用
    pub rate_spread: Option<f64>,    // 固定利差, 浮动付息债券适用
    pub inst_freq: i32,              // 年付息次数
    #[serde(deserialize_with = "str_to_date")]
    pub carry_date: NaiveDate, // 起息日
    #[serde(deserialize_with = "str_to_date")]
    pub maturity_date: NaiveDate, // 到期日
    pub day_count: String,           // 计息基准, 如A/365F
}

#[inline]
fn natural_days_between(date1: NaiveDate, date2: NaiveDate) -> i32 {
    (date1 - date2).num_days() as i32
}

#[inline]
fn str_to_date<'de, D>(deserializer: D) -> std::result::Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

impl Bond {
    #[inline]
    pub fn code(&self) -> &str {
        self.bond_code.split_once('.').unwrap().0
    }

    #[inline]
    pub fn is_zero_coupon(&self) -> bool {
        self.cp_type == CouponType::ZeroCoupon
    }

    /// 获得付息间隔
    pub fn get_cp_offset(&self) -> Result<Months> {
        match self.inst_freq {
            0 => Ok(Months::new(0)),
            1 => Ok(Months::new(12)),
            2 => Ok(Months::new(6)),
            _ => bail!("Invalid inst_freq: {}", self.inst_freq),
        }
    }

    /// 获取上一付息日和下一付息日
    pub fn get_nearest_cp_date(&self, date: NaiveDate) -> Result<(NaiveDate, NaiveDate)> {
        if self.is_zero_coupon() {
            bail!("Zero Coupon bond does not have coupon dates");
        }
        let date_offset = self.get_cp_offset()?;
        let mut cp_date = self.carry_date;
        let mut cp_date_next = cp_date + date_offset;
        // if date < cp_date {
        //     return Ok((None, Some(cp_date)));
        // } else if cp_date_next > self.maturity_date {
        //     return Ok((Some(cp_date), None));
        // }
        // 最多一年付息两次，不会超过200次循环
        for _ in 0..220 {
            if date >= cp_date && date < cp_date_next {
                return Ok((cp_date, cp_date_next));
            }
            cp_date = cp_date_next;
            cp_date_next = cp_date + date_offset;
        }
        bail!("Failed to find nearest coupon date");
    }

    #[inline]
    /// 计算应计利息
    ///
    /// 银行间和交易所的计算规则不同,银行间是算头不算尾,而交易所是算头又算尾
    pub fn calc_accrued_interest(&self, calculating_date: NaiveDate) -> Result<f64> {
        if self.is_zero_coupon() {
            return Ok(0.0);
        }
        let (pre_cp_date, next_cp_date) = self.get_nearest_cp_date(calculating_date)?;

        match self.mkt {
            Market::IB => {
                // 银行间是算头不算尾
                let inst_accrued_days = natural_days_between(pre_cp_date, calculating_date);
                let coupon = self.cp_rate_1st * self.par_value / self.inst_freq as f64;
                // 当前付息周期实际天数
                let present_cp_period_days = natural_days_between(pre_cp_date, next_cp_date);
                Ok(coupon * inst_accrued_days as f64 / present_cp_period_days as f64)
            }
            Market::SH | Market::SSE | Market::SZE | Market::SZ => {
                // 交易所是算头又算尾
                let inst_accrued_days = 1 + natural_days_between(pre_cp_date, calculating_date);
                Ok(self.cp_rate_1st * self.par_value * inst_accrued_days as f64 / 365.0)
            }
        }
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
}
