use super::{
    enums::{BondDayCount, CouponType, InterestType, Market},
    Bond,
};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

impl Default for Bond {
    fn default() -> Self {
        Bond {
            bond_code: "".to_string(),
            mkt: Market::default(),
            abbr: "".to_string(),
            par_value: 100.0,
            cp_type: CouponType::default(),
            interest_type: InterestType::default(),
            cp_rate_1st: 0.03,
            base_rate: None,
            rate_spread: None,
            inst_freq: 1,
            carry_date: NaiveDate::from_ymd_opt(2019, 6, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            day_count: BondDayCount::default(),
        }
    }
}

#[inline]
/// 将字符串转换为日期
///
/// 仅用于从json文件反序列化日期
pub(super) fn str_to_date<'de, D>(deserializer: D) -> std::result::Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

impl Eq for Bond {}

impl PartialEq for Bond {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bond_code == other.bond_code
    }
}
