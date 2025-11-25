use super::{Bond, BondDayCount, CouponType, InterestType, Market};
use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params};

fn get_coupon_type(i: i32) -> Result<CouponType> {
    match i {
        505001000 => Ok(CouponType::CouponBear),
        505002000 => Ok(CouponType::ZeroCoupon),
        505003000 => Ok(CouponType::ZeroCoupon),
        _ => bail!("Unknown coupon type enum: {}", i),
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
        let sql = "select s_info_windcode, s_info_name, b_info_par, b_info_couponrate, b_info_interesttype, b_info_spread, b_info_carrydate, b_infomaturitydate, b_info_interestfrequency from ?1 where s_info_windcode = ?2";
        // let sql = format!(
        //     "select s_info_windcode, s_info_name, b_info_par, b_info_couponrate, b_info_spread, b_info_carrydate, b_infomaturitydate, b_info_termyear_ from {} where s_info_windcode = ?",
        //     table_name.unwrap_or("bond_info")
        // );
        con.query_row(sql, params![table_name.unwrap_or_else(|| "bond_info"), code], |row| {
            let bond_code: String = row.get(0)?;
            let market = bond_code
                .split('.')
                .nth(1)
                .and_then(|m| m.parse::<Market>().ok())
                .unwrap_or_default();
            Ok(Bond {
                bond_code: bond_code.into(),
                mkt: market,
                abbr: row.get::<_, String>(1)?.into(),
                par_value: row.get::<_, f64>(2)?,
                cp_type: CouponType::CouponBear,
                interest_type: InterestType::Fixed,
                cp_rate: row.get::<_, f64>(3)?,
                base_rate: None,
                rate_spread: row.get::<_, Option<f64>>(4)?,
                inst_freq: 1,
                carry_date: row.get::<_, NaiveDate>(5)?,
                maturity_date: row.get::<_, NaiveDate>(6)?,
                day_count: BondDayCount::ActAct,
            })
        })
        .with_context(|| format!("Can not find bond {}", code))
    }
}
