use super::{Bond, BondDayCount, CouponType, InterestType, Market};
use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params};

fn get_coupon_type(i: i32) -> Result<CouponType> {
    match i {
        505001000 => Ok(CouponType::CouponBear),  // 附息
        505002000 => Ok(CouponType::ZeroCoupon),  // 零息
        505003000 => Ok(CouponType::ZeroCoupon),  // 贴现
        _ => bail!("Unknown coupon type enum: {}", i),
    }
}

fn get_interest_type(i: Option<i32>) -> Result<InterestType> {
    match i {
        Some(505001000) => Ok(InterestType::Floating), // 浮动利率
        Some(505002000) => Ok(InterestType::Fixed), // 固定利率
        Some(505003000) => Ok(InterestType::Progressive),  // 累进利率
        Some(i) => bail!("Unknown interest type enum: {}", i),
        None => Ok(InterestType::Zero),  // 零息
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
        CouponType::CouponBear => if let Some(freq) = freq {
            match freq {
                "Y1" => 1,
                _ => todo!(),
            }
        } else {
            0
        },
        CouponType::OneTime => 1,
        CouponType::ZeroCoupon => 0,
    }
}

impl Bond {
    pub fn read_duckdb(con: &Connection, table_name: Option<&str>, code: &str) -> Result<Bond> {
        // 只取一行，使用参数化查询避免拼接带来的转义问题
        let code = if code.contains('.') {
            code.to_string()
        } else {
            format!("{code}.IB")
        };
        let sql = "select s_info_windcode, s_info_name, b_info_par, b_info_coupon, b_info_interesttype, b_info_couponrate, b_info_spread, b_info_interestfrequency, b_info_carrydate, b_info_maturitydate, b_tendrst_referyield from ?1 where s_info_windcode = ?2";
        con.query_row(sql, params![table_name.unwrap_or_else(|| "bond_info"), code], |row| {
            let bond_code: String = row.get(0)?;
            let market = bond_code
                .split('.')
                .nth(1)
                .and_then(|m| m.parse::<Market>().ok())
                .unwrap_or_default();
            let coupon_type = get_coupon_type(row.get::<_, i32>(3)?).unwrap();
            Ok(Bond {
                bond_code: bond_code.into(),
                mkt: market,
                abbr: row.get::<_, String>(1)?.into(),
                par_value: row.get::<_, f64>(2)?,
                cp_type: coupon_type,
                interest_type: get_interest_type(row.get::<_, Option<i32>>(4)?).unwrap(),
                cp_rate: get_coupon_rate(row.get::<_, Option<f64>>(5)?, row.get::<_, Option<f64>>(12)?),
                base_rate: None,
                rate_spread: row.get::<_, Option<f64>>(6)?,
                inst_freq: get_inst_freq(&coupon_type, row.get::<_, Option<String>>(7)?.as_deref()),
                carry_date: row.get::<_, NaiveDate>(8)?,
                maturity_date: row.get::<_, NaiveDate>(9)?,
                day_count: BondDayCount::ActAct,
            })
        })
        .with_context(|| format!("Can not find bond {}", code))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_duckdb() {
        let con = Connection::open(r"C:\Users\Teamon\data\wind.duckdb").unwrap();
        let bond = Bond::read_duckdb(&con, None, "250215").unwrap();
        println!("{:?}", bond);
        todo!();
    }
}