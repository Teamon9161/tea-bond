mod fee;
mod trade_from_signal;

pub use fee::Fee;
use std::path::PathBuf;
pub use trade_from_signal::{TradeFromPosOpt, trading_from_pos};

use crate::CachedBond;
use chrono::NaiveDate;

use anyhow::{Result, anyhow};
use itertools::Either;
use itertools::izip;
use serde::Deserialize;
use tea_calendar::Calendar;
use tevec::prelude::{EPS, IsNone, Number, Vec1, Vec1View};

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
    #[serde(default)]
    pub avg_capital_spread: f64, // 平均资金加点
    #[serde(default)]
    pub capital: f64, // 累计资金
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
    capital_rate_vec: Option<&V>,
    capital_spread_vec: Option<&V>,
    opt: &BondTradePnlOpt,
) -> Result<Vec<PnlReport>>
where
    T: IsNone,
    T::Inner: Number,
    V: Vec1View<T>,
    VT: Vec1View<Option<NaiveDate>>,
{
    if qty_vec.is_empty() {
        return Ok(Vec::empty());
    }
    let multiplier = opt.multiplier;
    let mut state = opt.begin_state;
    let mut last_settle_time = None;
    let mut last_capital_rate = 0.;
    let mut last_cp_date = Default::default();
    let mut accrued_interest = 0.;
    let mut next_day_coupon: f64 = 0.;
    let mut duration_days = 0.; // 用于统计不同天的交易记录中间的天数间隔
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
    let capital_rate_vec = match capital_rate_vec {
        Some(v) => Either::Left(
            v.titer()
                .map(|v| if v.is_none() { 0. } else { v.unwrap().f64() }),
        ),
        None => Either::Right(std::iter::repeat(0.)),
    };
    let capital_spread_vec = match capital_spread_vec {
        Some(v) => Either::Left(
            v.titer()
                .map(|v| if v.is_none() { 0. } else { v.unwrap().f64() }),
        ),
        None => Either::Right(std::iter::repeat(0.)),
    };
    izip!(
        settle_time_vec.titer(),
        qty_vec.titer(),
        clean_price_vec.titer(),
        clean_close_vec.titer(),
        capital_rate_vec,
        capital_spread_vec,
    )
    .map(
        |(settle_time, qty, clean_price, clean_close, capital_rate, capital_spread)| {
            let qty = if qty.is_none() {
                0.
            } else {
                qty.unwrap().f64()
            };

            let (trade_price, close): (Option<f64>, Option<f64>) = if let Some(bond) = &symbol {
                if !bond.is_zero_coupon() {
                    let settle_time = settle_time.ok_or_else(|| {
                        anyhow!("Settle time should not be none when calc bond trade pnl")
                    })?;
                    // === 新的一天重新计算持仓的相关信息
                    if last_settle_time != Some(settle_time) {
                        // 计算中间持仓的资金
                        duration_days = last_settle_time
                            .map(|t| settle_time.signed_duration_since(t).num_days() as f64)
                            .unwrap_or(0.);
                        state.capital -= duration_days
                            * (last_capital_rate + state.avg_capital_spread)
                            * state.pos
                            * bond.par_value
                            * multiplier
                            / 365.;
                        if next_day_coupon != 0. {
                            state.coupon_paid += next_day_coupon;
                            next_day_coupon = 0.;
                        }
                        // 新的一天重新计算相关信息
                        let cp_dates = bond.get_nearest_cp_date(settle_time)?;
                        accrued_interest =
                            bond.calc_accrued_interest(settle_time, Some(cp_dates))?;
                        last_cp_date = bond.mkt.find_workday(cp_dates.0, 0);
                        // 当天初始仓位会产生的票息
                        if settle_time == last_cp_date {
                            // 调节应计利息
                            accrued_interest = coupon_paid;
                            next_day_coupon += coupon_paid * multiplier * state.pos;
                        }
                        last_settle_time = Some(settle_time);
                        if capital_rate != 0. {
                            last_capital_rate = capital_rate;
                        }
                    }
                    // === 交易当天付息调整
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
                        state.realized_pnl +=
                            (trade_price - state.pos_price) * multiplier * qty_chg;
                        if qty.abs() > prev_pos.abs() {
                            // 反向开仓
                            state.pos_price = trade_price;
                            state.avg_capital_spread = capital_spread;
                        } else {
                            if capital_spread != 0. {
                                state.avg_capital_spread = if state.pos.abs() < EPS {
                                    0.
                                } else {
                                    (state.avg_capital_spread * prev_pos.abs()
                                        - capital_spread.abs() * qty.abs())
                                        / state.pos.abs()
                                };
                            }
                        }
                    } else {
                        // 加仓
                        state.pos_price = (state.pos_price * prev_pos.abs()
                            + qty.abs() * trade_price)
                            / state.pos.abs();
                        state.avg_price = state.amt / (state.pos * multiplier);
                        if capital_spread != 0. {
                            state.avg_capital_spread = (state.avg_capital_spread * prev_pos.abs()
                                + capital_spread.abs() * qty.abs())
                                / state.pos.abs();
                        }
                    }
                    if state.pos.abs() <= EPS {
                        state.avg_price = 0.;
                        state.pos_price = 0.;
                        state.avg_capital_spread = 0.;
                    }
                } else {
                    // 之前仓位是0
                    state.avg_price = trade_price;
                    state.pos_price = state.avg_price;
                    state.avg_capital_spread = capital_spread;
                }
            }
            if let Some(close) = close {
                let close = close.f64();
                state.pnl =
                    state.pos * close * multiplier + state.coupon_paid - state.amt - state.fee
                        + state.capital;
                state.unrealized_pnl = state.pos * (close - state.pos_price) * multiplier;
            }
            // println!("PNL Report: {:?}", state);
            Ok(state)
        },
    )
    .collect()
}
