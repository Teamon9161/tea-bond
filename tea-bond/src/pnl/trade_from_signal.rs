use std::fmt::Debug;

use itertools::izip;
use serde::Deserialize;
use tevec::prelude::{EPS, IsNone, Number, Vec1View};

#[derive(Default, Debug)]
pub struct Trade<T> {
    pub time: T,
    pub price: f64,
    pub qty: f64,
}

impl<T: PartialEq + Debug> std::ops::Add<Trade<T>> for Trade<T> {
    type Output = Trade<T>;
    #[inline]
    fn add(self, rhs: Trade<T>) -> Trade<T> {
        if (self.time == rhs.time) && (self.price == rhs.price) {
            Trade {
                time: self.time,
                price: self.price,
                qty: self.qty + rhs.qty,
            }
        } else if self.time == rhs.time {
            let amt = self.qty * self.price + rhs.qty * rhs.price;
            let price = if self.qty + rhs.qty != 0. {
                amt / (self.qty + rhs.qty)
            } else {
                amt
            };
            Trade {
                time: self.time,
                price,
                qty: self.qty + rhs.qty,
            }
        } else {
            panic!("trade time or price is not equal, {self:?} != {rhs:?}");
        }
    }
}

impl<T: PartialEq + Debug> std::ops::Add<Option<Trade<T>>> for Trade<T> {
    type Output = Trade<T>;
    #[inline]
    fn add(self, rhs: Option<Trade<T>>) -> Trade<T> {
        if let Some(rt) = rhs { self + rt } else { self }
    }
}

#[derive(Deserialize)]
pub struct TradeFromPosOpt {
    pub cash: Option<f64>,
    pub multiplier: f64,
    pub qty_tick: f64,
    pub stop_on_finish: bool,
    #[serde(default)]
    pub finish_price: Option<f64>,
    #[serde(default)]
    pub min_adjust_amt: Option<f64>,
    #[serde(default)]
    pub keep_shape: Option<bool>,
}

const INIT_TRADE_COUNT: usize = 512;

fn quantize_inside(q: f64, tick: f64) -> f64 {
    if tick <= 0.0 {
        return q;
    }
    if q >= 0.0 {
        (q / tick).floor() * tick // 买单向下取整
    } else {
        (q / tick).ceil() * tick // 卖单向上取整（数值更接近 0）
    }
}

#[allow(clippy::collapsible_if)]
pub fn trading_from_pos<DT, T, VT, V>(
    time_vec: &VT,
    pos_vec: &V,
    open_vec: &V,
    opt: &TradeFromPosOpt,
) -> Vec<Option<Trade<DT>>>
where
    DT: Clone + PartialEq + Debug,
    T: IsNone,
    T::Inner: Number,
    VT: Vec1View<DT>,
    V: Vec1View<T>,
{
    let mut last_pos = 0.;
    let mut open_price: f64 = 0.;
    let mut open_qty: f64 = 0.;
    let cash = opt.cash.unwrap();
    let min_adjust_amt = opt.min_adjust_amt.unwrap_or(0.);
    let mut trades = Vec::with_capacity(INIT_TRADE_COUNT);
    let keep_shape = opt.keep_shape.unwrap_or(false);

    // 记录最后一个可用 (time, price)，用于 stop_on_finish
    let mut last_tp: Option<(DT, f64)> = None;

    izip!(time_vec.titer(), pos_vec.titer(), open_vec.titer()).for_each(|(time, pos, open)| {
        if pos.not_none() && open.not_none() {
            let pos = pos.unwrap().f64();
            let price = open.unwrap().f64();
            // 记录最新可用的时间与价格
            last_tp = Some((time.clone(), price));

            let dpos = pos - last_pos;
            if dpos.abs() > EPS {
                // 目标名义 -> 成交量（正买负卖）
                let qty = if pos.abs() > EPS {
                    let p = if open_price > 0. { open_price } else { price };
                    let raw_qty = dpos * cash / (p * opt.multiplier);
                    // 量化到最小变动单位（朝 0 截断，避免超买/超卖）
                    quantize_inside(raw_qty, opt.qty_tick)
                } else {
                    -open_qty
                };
                let adjust_amt = qty.abs() * price * opt.multiplier;

                if adjust_amt > min_adjust_amt {
                    if open_qty == 0. {
                        // 开仓情况
                        open_price = price;
                    } else if dpos.signum() == open_qty.signum() {
                        open_price = (open_price * open_qty + qty * price) / (qty + open_qty);
                    } else if open_qty.abs() > qty.abs() {
                        // 反向加仓, 价格为新的开仓价格
                        open_price = price;
                    };
                    // 减仓情况的价格不改变
                    trades.push(Some(Trade {
                        time: time.clone(),
                        price,
                        qty,
                    }));
                    open_qty += qty;
                    last_pos = pos;
                } else if keep_shape {
                    trades.push(None);
                }
            } else if keep_shape {
                trades.push(None);
            }
        } else if keep_shape {
            trades.push(None)
        }
    });

    // 收尾是否强制平仓
    if opt.stop_on_finish && (open_qty != 0.0) {
        if let Some((t, p)) = last_tp {
            let p = opt.finish_price.unwrap_or(p);
            let trade = Trade {
                time: t,
                price: p,
                qty: -open_qty,
            };
            if keep_shape {
                let last_trade = trades.pop().flatten();
                trades.push(Some(trade + last_trade));
            } else {
                trades.push(Some(trade))
            }
        }
    }
    trades
}
