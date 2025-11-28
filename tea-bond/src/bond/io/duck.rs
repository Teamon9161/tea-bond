use super::{WindSqlRow, default_dir};
use crate::bond::Bond;
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use duckdb::{Connection, Row, params};
use std::sync::{Arc, LazyLock};

pub static DUCKDB_TABLE_PATH: LazyLock<String> = LazyLock::new(|| {
    std::env::var("BONDS_INFO_DUCKDB_TABLE").unwrap_or_else(|_| {
        default_dir()
            .join("bonds_info.duckdb")
            .to_string_lossy()
            .into()
    })
});

impl TryFrom<&Row<'_>> for WindSqlRow {
    type Error = duckdb::Error;
    fn try_from(value: &Row<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(WindSqlRow {
            s_info_windcode: value.get::<_, Arc<str>>("s_info_windcode")?.as_ref().into(),
            s_info_name: value.get::<_, Arc<str>>("s_info_name")?.as_ref().into(),
            b_info_par: value.get("b_info_par")?,
            b_info_coupon: value.get("b_info_coupon")?,
            b_info_interesttype: value
                .get::<_, Option<f64>>("b_info_interesttype")?
                .map(|v| v as i32),
            b_info_couponrate: value.get("b_info_couponrate")?,
            b_info_spread: value.get("b_info_spread")?,
            b_info_interestfrequency: value
                .get::<_, Option<Arc<str>>>("b_info_interestfrequency")?
                .map(|s| s.as_ref().into()),
            b_info_carrydate: NaiveDate::parse_from_str(
                value.get::<_, Arc<str>>("b_info_carrydate")?.as_ref(),
                "%Y%m%d",
            )
            .expect("Parse b_info_carrydate failed"),
            b_info_maturitydate: NaiveDate::parse_from_str(
                value.get::<_, Arc<str>>("b_info_maturitydate")?.as_ref(),
                "%Y%m%d",
            )
            .expect("Parse b_info_maturitydate failed"),
            b_tendrst_referyield: value.get("b_tendrst_referyield")?,
            b_info_issueprice: value.get("b_info_issueprice")?,
            // ..Default::default()
        })
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
        if table
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '_')
        {
            bail!("Invalid table name: {table}");
        }
        let sql = format!(
            "select s_info_windcode, s_info_name, b_info_par, b_info_coupon, b_info_interesttype, b_info_couponrate, b_info_spread, b_info_interestfrequency, b_info_carrydate, b_info_maturitydate, b_tendrst_referyield, b_info_issueprice from {table} where s_info_windcode = ?"
        );
        con.query_row(&sql, params![code], |row| {
            let row: WindSqlRow = row.try_into()?;
            Ok(row.try_into().unwrap())
        })
        .with_context(|| format!("Can not find bond {} in duckdb", code))
    }
}
