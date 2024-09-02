mod bond;
mod future;
use chrono::NaiveDate;
// use tea_time::DateTime;

pub use bond::{Bond, CouponType, InterestType, Market};
pub use future::{Future, FutureType};

#[derive(Debug, Clone)]
pub struct FuturePrice<'a> {
    pub future: &'a Future,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct BondYtm<'a> {
    pub bond: &'a Bond,
    pub ytm: f64,
}

pub struct TfEvaluator<'a> {
    pub evaluating_date: NaiveDate,
    pub future_price: FuturePrice<'a>,
    pub bond_ytm: BondYtm<'a>,
    pub capital_rate: f64,
    pub b_inst_reinvest_rate: f64,
}
