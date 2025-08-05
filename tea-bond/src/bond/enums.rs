use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tea_calendar::{china::*, Calendar};
#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum CouponType {
    /// 附息债券
    #[serde(alias = "Coupon_Bear")]
    #[default]
    CouponBear,
    /// 零息债券
    #[serde(alias = "Zero_Coupon")]
    ZeroCoupon,
    /// 一次性付息
    #[serde(alias = "One_Time")]
    OneTime,
}

impl FromStr for CouponType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Coupon_Bear" | "CouponBear" => Ok(CouponType::CouponBear),
            "Zero_Coupon" | "ZeroCoupon" => Ok(CouponType::ZeroCoupon),
            "One_Time" | "OneTime" => Ok(CouponType::OneTime),
            _ => anyhow::bail!("Unknown coupon type: {}", s),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum InterestType {
    /// 固定利率
    #[default]
    Fixed,
    /// 浮动利率
    Floating,
    /// 累进利率
    Progressive,
    /// 零息
    Zero,
}

impl FromStr for InterestType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fixed" | "Fix" | "fix" | "fixed" => Ok(InterestType::Fixed),
            "Floating" | "floating" | "Float" | "float" => Ok(InterestType::Floating),
            "Progressive" | "progressive" => Ok(InterestType::Progressive),
            "Zero" | "zero" => Ok(InterestType::Zero),
            _ => anyhow::bail!("Unknown interest type: {}", s),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum Market {
    /// 银行间
    #[default]
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

impl FromStr for Market {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IB" => Ok(Market::IB),
            "SSE" => Ok(Market::SSE),
            "SH" => Ok(Market::SH),
            "SZE" => Ok(Market::SZE),
            "SZ" => Ok(Market::SZ),
            _ => anyhow::bail!("Unknown market: {}", s),
        }
    }
}

impl Calendar for Market {
    fn is_business_day(&self, date: NaiveDate) -> bool {
        match self {
            Market::IB => IB.is_business_day(date),
            Market::SSE => SSE.is_business_day(date),
            Market::SH => SSE.is_business_day(date),
            Market::SZE => SZE.is_business_day(date),
            Market::SZ => SZE.is_business_day(date),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
/// 计息基准
pub enum BondDayCount {
    // 实际天数/实际天数
    #[serde(alias = "ACT/ACT")]
    #[default]
    ActAct,
    // 实际天数/365
    #[serde(alias = "A/365")]
    Act365,
    // 实际天数/360
    #[serde(alias = "A/360")]
    Act360,
    #[serde(alias = "A/365F")]
    Act365F,
    #[serde(alias = "T/365")]
    Thirty365,
    #[serde(alias = "T/360")]
    Thirty360,
    Bus,
    #[serde(alias = "BUSIB")]
    BusIB,
    #[serde(alias = "BUSSSE")]
    BusSSE,
}

impl FromStr for BondDayCount {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACT/ACT" | "Act/Act" => Ok(BondDayCount::ActAct),
            "A/365" | "Act/365" => Ok(BondDayCount::Act365),
            "A/360" | "Act/360" => Ok(BondDayCount::Act360),
            "A/365F" | "Act/365F" => Ok(BondDayCount::Act365F),
            "T/365" | "Thirty/365" => Ok(BondDayCount::Thirty365),
            "T/360" | "Thirty/360" => Ok(BondDayCount::Thirty360),
            "BUS" | "Bus" => Ok(BondDayCount::Bus),
            "BUSIB" | "BUS/IB" | "Bus/IB" => Ok(BondDayCount::BusIB),
            "BUSSSE" | "BUS/SSE" | "Bus/SSE" => Ok(BondDayCount::BusSSE),
            _ => anyhow::bail!("Unknown day count: {}", s),
        }
    }
}
