use super::{Bond, BondDayCount, CouponType, InterestType, Market};
use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params};

fn get_coupon_type(i: i32) -> Result<CouponType> {
    match i {
        505001000 => Ok(CouponType::CouponBear),  // 附息
        505002000 => Ok(CouponType::OneTime),  // 到期一次还本付息
        505003000 => Ok(CouponType::ZeroCoupon),  // 贴现
        _ => bail!("Unknown coupon type enum: {}", i),
    }
}

fn get_interest_type(i: Option<i32>) -> Result<InterestType> {
    match i {
        Some(501001000) => Ok(InterestType::Floating), // 浮动利率
        Some(501002000) => Ok(InterestType::Fixed), // 固定利率
        Some(501003000) => Ok(InterestType::Progressive),  // 累进利率
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
                "M6" => 2,
                "M4" => 3,
                "M3" => 4,
                "M2" => 6,
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
        let code: std::borrow::Cow<'_, str> = if !code.contains('.') {
            format!("{code}.IB").into()
        } else {
            code.into()
        };
        let table = table_name.unwrap_or("bond_info");
        if table.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
            bail!("Invalid table name: {table}");
        }
        let sql = format!("select s_info_windcode, s_info_name, b_info_par, b_info_coupon, b_info_interesttype, b_info_couponrate, b_info_spread, b_info_interestfrequency, b_info_carrydate, b_info_maturitydate, b_tendrst_referyield, b_info_issueprice from {table} where s_info_windcode = ?");
        con.query_row(&sql, params![code], |row| {
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
                interest_type: get_interest_type(row.get::<_, Option<f64>>(4)?.map(|v| v as i32)).unwrap(),
                cp_rate: get_coupon_rate(row.get::<_, Option<f64>>(5)?, row.get::<_, Option<f64>>(10)?),
                base_rate: None,
                rate_spread: row.get(6)?,
                inst_freq: get_inst_freq(&coupon_type, row.get::<_, Option<String>>(7)?.as_deref()),
                carry_date: NaiveDate::parse_from_str(row.get::<_, String>(8)?.as_str(), "%Y%m%d").unwrap(),
                maturity_date: NaiveDate::parse_from_str(row.get::<_, String>(9)?.as_str(), "%Y%m%d").unwrap(),
                day_count: BondDayCount::ActAct,
                issue_price: row.get(11)?,
            })
        })
        .with_context(|| format!("Can not find bond {} in duckdb", code))
    }
}


#[cfg(test)]
mod tests {
    use crate::day_counter::{DayCountRule, ACTUAL};

    use super::*;
    #[test]
    fn test_read_duckdb() {
        let code = "259960";
        let date = NaiveDate::from_ymd_opt(2025, 11 ,25).unwrap(); 
        let con = Connection::open(r"C:\Users\Teamon\data\wind.duckdb").unwrap();
        let bond = Bond::read_duckdb(&con, None, code).unwrap();
        let bond2 = Bond::read_json(code, None).unwrap();
        
        println!("{:?}\n{:?}", bond, bond2);
        let ai1 = bond.calc_accrued_interest(date, None).unwrap();
        let ai2 = bond2.calc_accrued_interest(date, None).unwrap();
        println!("{ai1}\n{ai2}");
        println!("{}", bond.calc_ytm_with_price(99.69, NaiveDate::from_ymd_opt(2025, 9 ,25).unwrap(), None, None).unwrap());
        println!("{}, {}", ACTUAL.count_days(bond.carry_date, date), ACTUAL.count_days(bond.carry_date, bond.maturity_date));
        todo!();
    }
}
