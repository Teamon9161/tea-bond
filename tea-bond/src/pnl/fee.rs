use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fee {
    /// 按成交金额百分比：fee = rate * amount
    Percent { fee: f64 },

    /// 按成交数量：fee = per_qty * qty
    PerQty { fee: f64 },

    /// 按笔数：fee = per_trade * trade_num
    PerTrade { fee: f64 },

    /// 多个 Fee 相加
    Sum { items: Vec<Fee> },

    /// 上限： fee = min(cap, inner_fee)
    Min { cap: f64, fee: Box<Fee> },

    /// 下限（最少多少钱）： fee = max(floor, inner_fee)
    Max { floor: f64, fee: Box<Fee> },

    /// 固定手续费
    Fixed { fee: f64 },

    /// 零手续费
    #[default]
    Zero,
}

impl Fee {
    /// 计算手续费
    ///
    /// - `qty`: 成交数量
    /// - `amount`: 成交金额（比如价格 * 数量，如果是债券可以用净价/全价 * 面值 * 手数等）
    /// - `trade_num`: 交易笔数（一般一次撮合就是 1）
    pub fn amount(&self, qty: f64, amount: f64, trade_num: u64) -> f64 {
        match self {
            Fee::Zero => 0.0,

            Fee::Fixed { fee } => *fee,

            Fee::Percent { fee } => amount.abs() * fee,

            Fee::PerQty { fee } => qty.abs() * fee,

            Fee::PerTrade { fee } => *fee * trade_num as f64,

            Fee::Sum { items } => items.iter().map(|f| f.amount(qty, amount, trade_num)).sum(),

            // 手续费上限： min(cap, fee)
            Fee::Min { cap, fee } => {
                let inner = fee.amount(qty, amount, trade_num);
                inner.min(*cap)
            }

            // 手续费下限（至少 floor）： max(floor, fee)
            Fee::Max { floor, fee } => {
                let inner = fee.amount(qty, amount, trade_num);
                inner.max(*floor)
            }
        }
    }
}

impl Add for Fee {
    type Output = Fee;

    fn add(self, rhs: Fee) -> Fee {
        match (self, rhs) {
            (Fee::Sum { mut items }, Fee::Sum { items: mut items2 }) => {
                items.append(&mut items2);
                Fee::Sum { items }
            }
            (Fee::Sum { mut items }, other) => {
                items.push(other);
                Fee::Sum { items }
            }
            (other, Fee::Sum { mut items }) => {
                items.insert(0, other);
                Fee::Sum { items }
            }
            (lhs, rhs) => Fee::Sum {
                items: vec![lhs, rhs],
            },
        }
    }
}
