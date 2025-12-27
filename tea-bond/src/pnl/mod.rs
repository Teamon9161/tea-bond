mod fee;
mod trade_from_signal;

pub use fee::Fee;
use std::path::PathBuf;
pub use trade_from_signal::{TradeFromPosOpt, trading_from_pos};

use crate::CachedBond;
use chrono::{Days, NaiveDate};

use anyhow::{Result, anyhow};
use itertools::izip;
use serde::Deserialize;
use tea_calendar::Calendar;
use tevec::prelude::{EPS, IsNone, Number, Vec1, Vec1View};
pub const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct PnlReport {
    pub pos: f64,
    pub avg_price: f64,
    pub pnl: f64,
    pub realized_pnl: f64,
    pub pos_price: f64,
    pub unrealized_pnl: f64,
    pub coupon_paid: f64,
    pub amt: f64,
    pub fee: f64,
}

#[derive(Deserialize)]
pub struct BondTradePnlOpt {
    pub bond_info_path: Option<PathBuf>,
    pub multiplier: f64,
    pub fee: Fee,
    pub begin_state: PnlReport,
}

pub fn calc_bond_trade_pnl<T, V, VT>(
    symbol: Option<&str>,
    settle_time_vec: &VT,
    qty_vec: &V,
    clean_price_vec: &V,
    clean_close_vec: &V,
    opt: &BondTradePnlOpt,
) -> Result<Vec<PnlReport>>
where
    T: IsNone,
    T::Inner: Number,
    V: Vec1View<T>,
    VT: Vec1View<Option<i32>>,
{
    if qty_vec.is_empty() {
        return Ok(Vec::empty());
    }
    let multiplier = opt.multiplier;
    let mut state = opt.begin_state;
    let mut last_settle_time = None;
    let mut last_cp_date = EPOCH;
    let mut accrued_interest = 0.;
    let mut next_day_coupon: f64 = 0.;
    let symbol = if let Some(bond) = symbol {
        if bond.is_empty() {
            None
        } else {
            CachedBond::new(bond, opt.bond_info_path.as_deref()).ok()
        }
    } else {
        None
    };
    let coupon_paid = symbol.as_ref().map(|bond| bond.get_coupon()).unwrap_or(0.);
    izip!(
        settle_time_vec.titer(),
        qty_vec.titer(),
        clean_price_vec.titer(),
        clean_close_vec.titer(),
    )
    .map(|(settle_time, qty, clean_price, clean_close)| {
        let qty = if qty.is_none() {
            0.
        } else {
            qty.unwrap().f64()
        };

        let (trade_price, close): (Option<f64>, Option<f64>) = if let Some(bond) = &symbol {
            if !bond.is_zero_coupon() {
                let settle_time = EPOCH
                    .checked_add_days(Days::new(settle_time.ok_or_else(|| {
                        anyhow!("Settle time should not be none when calc bond trade pnl")
                    })? as u64))
                    .unwrap();
                if last_settle_time != Some(settle_time) {
                    if next_day_coupon != 0. {
                        state.coupon_paid += next_day_coupon;
                        next_day_coupon = 0.;
                    }
                    // 新的一天重新计算相关信息
                    let cp_dates = bond.get_nearest_cp_date(settle_time)?;
                    accrued_interest = bond.calc_accrued_interest(settle_time, Some(cp_dates))?;
                    last_cp_date = bond.mkt.find_workday(cp_dates.0, 0);
                    // 当天初始仓位会产生的票息
                    if settle_time == last_cp_date {
                        // 调节应计利息
                        accrued_interest = coupon_paid;
                        next_day_coupon += coupon_paid * multiplier * state.pos;
                    }
                    last_settle_time = Some(settle_time);
                }
                // 交易当天会产生付息
                if (settle_time == last_cp_date) & (qty != 0.) {
                    next_day_coupon += coupon_paid * multiplier * qty;
                }
            }
            (
                clean_price.map(|v| v.f64() + accrued_interest),
                clean_close.map(|v| v.f64() + accrued_interest),
            )
        } else {
            (clean_price.map(|v| v.f64()), clean_close.map(|v| v.f64()))
        };
        if qty != 0. {
            let trade_price = trade_price
                .ok_or_else(|| anyhow!("Trade price should not be none"))?
                .f64();
            let prev_pos = state.pos;
            let trade_amt = qty * trade_price * multiplier; // with sign
            state.pos += qty;
            state.amt += trade_amt;
            state.fee += opt.fee.amount(qty, trade_amt, 1); // Fee model will take into account the sign of the trade amount and quantity.
            if prev_pos.abs() > EPS {
                if qty.signum() != prev_pos.signum() {
                    // 减仓
                    let qty_chg = qty.abs().min(prev_pos.abs()) * prev_pos.signum();
                    state.realized_pnl += (trade_price - state.pos_price) * multiplier * qty_chg;
                    if qty.abs() > prev_pos.abs() {
                        // 反向开仓
                        state.pos_price = trade_price;
                    }
                } else {
                    state.pos_price = (state.pos_price * prev_pos.abs() + qty.abs() * trade_price)
                        / state.pos.abs();
                    state.avg_price = state.amt / (state.pos * multiplier)
                }
                if state.pos.abs() <= EPS {
                    state.avg_price = 0.;
                    state.pos_price = 0.;
                    // state.amt = 0.;
                }
            } else {
                // 之前仓位是0
                state.avg_price = trade_price;
                state.pos_price = state.avg_price;
            }
        }
        if let Some(close) = close {
            let close = close.f64();
            state.pnl = state.pos * close * multiplier + state.coupon_paid - state.amt - state.fee;
            state.unrealized_pnl = state.pos * (close - state.pos_price) * multiplier;
        }
        // println!("PNL Report: {:?}", state);
        Ok(state)
    })
    .collect()
}
