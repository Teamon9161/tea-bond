use crate::SmallStr;
use crate::bond::{Bond, BondDayCount, CouponType, InterestType, Market};
use chrono::NaiveDate;
// use serde::{Deserialize, Serialize};
use anyhow::{Result, bail};
// use crate::bond::impl_traits::{deserialize_date, serialize_date};

#[derive(Debug, Clone)]
pub struct WindSqlRow {
    pub s_info_windcode: SmallStr,
    pub s_info_name: SmallStr,
    pub b_info_par: f64,
    pub b_info_coupon: i32,
    pub b_info_interesttype: Option<i32>,
    pub b_info_couponrate: Option<f64>,
    pub b_info_spread: Option<f64>,
    pub b_info_interestfrequency: Option<SmallStr>,
    pub b_info_carrydate: NaiveDate,
    pub b_info_maturitydate: NaiveDate,
    pub b_tendrst_referyield: Option<f64>,
    pub b_info_issueprice: Option<f64>,
}

impl Default for WindSqlRow {
    fn default() -> Self {
        Self {
            s_info_windcode: "".into(),
            s_info_name: "".into(),
            b_info_par: 100.0,
            b_info_coupon: 505001000,
            b_info_interesttype: None,
            b_info_couponrate: Some(0.03),
            b_info_spread: None,
            b_info_interestfrequency: Some("Y1".into()),
            b_info_carrydate: NaiveDate::default(),
            b_info_maturitydate: NaiveDate::default(),
            b_tendrst_referyield: None,
            b_info_issueprice: None,
        }
    }
}

fn get_coupon_type(i: i32) -> Result<CouponType> {
    match i {
        505001000 => Ok(CouponType::CouponBear), // 附息
        505002000 => Ok(CouponType::OneTime),    // 到期一次还本付息
        505003000 => Ok(CouponType::ZeroCoupon), // 贴现
        _ => bail!("Unknown coupon type enum: {}", i),
    }
}

fn get_interest_type(i: Option<i32>) -> Result<InterestType> {
    match i {
        Some(501001000) => Ok(InterestType::Floating), // 浮动利率
        Some(501002000) => Ok(InterestType::Fixed),    // 固定利率
        Some(501003000) => Ok(InterestType::Progressive), // 累进利率
        Some(i) => bail!("Unknown interest type enum: {}", i),
        None => Ok(InterestType::Zero), // 零息
    }
}

#[inline]
fn round(f: f64, precision: i32) -> f64 {
    let factor = 10f64.powi(precision);
    (f * factor).round() / factor
}

fn get_coupon_rate(cp_rate: Option<f64>, float_rate: Option<f64>) -> f64 {
    match (cp_rate, float_rate) {
        (Some(r), _) => round(r * 0.01, 6),
        (None, Some(r)) => round(r * 0.01, 6),
        (None, None) => f64::NAN,
    }
}

fn get_inst_freq(coupon_type: &CouponType, freq: Option<&str>) -> i32 {
    match coupon_type {
        CouponType::CouponBear => {
            if let Some(freq) = freq {
                match freq {
                    "Y1" => 1,
                    "M6" => 2,
                    "M4" => 3,
                    "M3" => 4,
                    "M2" => 6,
                    "M1" => 12,
                    _ => -1,
                }
            } else {
                0
            }
        }
        CouponType::OneTime => 1,
        CouponType::ZeroCoupon => 0,
    }
}

impl TryFrom<WindSqlRow> for Bond {
    type Error = anyhow::Error;
    fn try_from(row: WindSqlRow) -> Result<Self> {
        let market = row
            .s_info_windcode
            .split('.')
            .nth(1)
            .and_then(|m| m.parse::<Market>().ok())
            .unwrap_or_default();
        let cp_type = get_coupon_type(row.b_info_coupon)?;
        Ok(Bond {
            bond_code: row.s_info_windcode,
            mkt: market,
            abbr: row.s_info_name,
            par_value: row.b_info_par,
            cp_type,
            interest_type: get_interest_type(row.b_info_interesttype)?,
            cp_rate: get_coupon_rate(row.b_info_couponrate, row.b_tendrst_referyield),
            base_rate: None,
            rate_spread: row.b_info_spread,
            inst_freq: get_inst_freq(&cp_type, row.b_info_interestfrequency.as_deref()),
            carry_date: row.b_info_carrydate,
            maturity_date: row.b_info_maturitydate,
            issue_price: row.b_info_issueprice,
            day_count: BondDayCount::default(),
        })
    }
}
