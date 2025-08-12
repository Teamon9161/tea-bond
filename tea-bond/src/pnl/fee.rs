use anyhow::{Error, bail};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default)]
pub enum SingleFee {
    // Represents a fixed fee for a trade.
    Trade(f64),
    // Represents a fee per quantity.
    Qty(f64),
    // Represents a percentage fee.
    Percent(f64),
    #[default]
    // Represents a zero fee.
    Zero,
}

impl SingleFee {
    #[inline]
    pub fn amount(&self, qty: f64, amount: f64, trade_num: u64) -> f64 {
        match self {
            SingleFee::Trade(fee) => fee * trade_num as f64,
            SingleFee::Qty(fee) => fee * qty.abs(),
            SingleFee::Percent(fee) => fee * amount.abs(),
            SingleFee::Zero => 0.0,
        }
    }
}

impl FromStr for SingleFee {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(SingleFee::Zero)
        } else if s.starts_with("Trade(") {
            let fee = s[6..s.len() - 1].parse()?;
            Ok(SingleFee::Trade(fee))
        } else if s.starts_with("Qty(") {
            let fee = s[4..s.len() - 1].parse()?;
            Ok(SingleFee::Qty(fee))
        } else if s.starts_with("Percent(") {
            let fee = s[8..s.len() - 1].parse()?;
            Ok(SingleFee::Percent(fee))
        } else {
            bail!("Invalid fee type")
        }
    }
}

pub struct Fee(pub Vec<SingleFee>);

impl Fee {
    #[inline]
    pub fn amount(&self, qty: f64, amount: f64, trade_num: u64) -> f64 {
        if self.0.is_empty() {
            0.0
        } else {
            self.0
                .iter()
                .map(|fee| fee.amount(qty, amount, trade_num))
                .sum()
        }
    }
}

impl FromStr for Fee {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fees = s
            .split('+')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Fee(fees))
    }
}
