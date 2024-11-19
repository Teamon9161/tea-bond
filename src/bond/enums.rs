use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum InterestType {
    /// 固定利率
    Fixed,
    /// 浮动利率
    #[default]
    Floating,
    /// 累进利率
    Progressive,
    /// 零息
    Zero,
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
