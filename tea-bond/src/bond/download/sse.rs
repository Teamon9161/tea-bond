use crate::bond::{Bond, BondDayCount, CouponType, InterestType};
use crate::SmallStr;
use anyhow::{anyhow, bail, Result};
use chrono::NaiveDate;
use compact_str::ToCompactString;
use std::time::{SystemTime, UNIX_EPOCH};

const SSE_API_URL: &str = "https://query.sse.com.cn/sseQuery/commonSoaQuery.do";
const BASE_QUERY_PARAMS: [(&str, &str); 7] = [
    ("isPagination", "true"),
    ("pageHelp.pageSize", "25"),
    ("pageHelp.pageNo", "1"),
    ("pageHelp.beginPage", "1"),
    ("pageHelp.cacheSize", "1"),
    ("pageHelp.endPage", "1"),
    ("sqlId", "CP_ZQ_ZQLB"),
];

use rand::Rng;

fn generate_callback() -> String {
    let random_num = rand::rng().random_range(10000000..99999999);
    format!("jsonpCallback{random_num}")
}

fn extract_json(response: String) -> Result<serde_json::Value> {
    let start = response
        .find('(')
        .ok_or(anyhow!("No opening parenthesis"))?
        + 1;
    let end = response
        .rfind(')')
        .ok_or(anyhow!("No closing parenthesis"))?;
    Ok(serde_json::from_str(&response[start..end])?)
}

fn sse_get_coupon_inst_freq(
    pay_typ: &str,
    interest_type: &str,
) -> Result<(CouponType, InterestType, i32)> {
    let interest_type = match interest_type {
        "固定利息" => Some(InterestType::Fixed),
        "浮动利息" => Some(InterestType::Floating),
        _ => None,
    };
    match pay_typ {
        "按半年付息" => Ok((
            CouponType::CouponBear,
            interest_type.unwrap_or(InterestType::Fixed),
            2,
        )),
        "按年付息" => Ok((
            CouponType::CouponBear,
            interest_type.unwrap_or(InterestType::Fixed),
            1,
        )),
        "到期一次还本付息" => Ok((
            CouponType::OneTime,
            interest_type.unwrap_or(InterestType::Fixed),
            1,
        )),
        typ => bail!(
            "Cannot infer coupon type and interest type from SSE: {}",
            typ
        ),
    }
}

impl Bond {
    pub async fn sh_download_from_sse(code: &str) -> Result<Bond> {
        let client = reqwest::Client::new();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_compact_string();
        let response = client
            .get(SSE_API_URL)
            .query(&[(
                "jsonCallBack",
                format!("jsonpCallback{}", generate_callback()),
            )])
            .query(&BASE_QUERY_PARAMS)
            .query(&[
                ("BOND_CODE", code),
                ("BOND_TYPE", "全部"),
                ("_", ts.as_str()),
            ])
            .header("Referer", "https://www.sse.com.cn/")
            .send()
            .await?
            .text()
            .await?;
        // dbg!(&info);
        let response = extract_json(response)?;
        let results = response["result"]
            .as_array()
            .ok_or_else(|| anyhow!("Can not find bond {} from SSE, the results is empty", code))?;
        if results.is_empty() {
            bail!("Can not find bond {} from SSE, the results is empty", code)
        }
        for res in results {
            if res["BOND_CODE"].as_str().unwrap() == code {
                // println!("{:#?}", res);
                let (cp_type, interest_type, inst_freq) = sse_get_coupon_inst_freq(
                    res["PAY_TYPE"].as_str().unwrap(),
                    res["INTEREST_TYPE"].as_str().unwrap(),
                )?;
                let (base_rate, rate_spread) = if let InterestType::Floating = interest_type {
                    bail!("Get base rate & rate spread for floating bond in SSE is not implemented yet");
                } else {
                    (None, None)
                };
                let bond = Bond {
                    bond_code: SmallStr::new(code) + ".SH",
                    mkt: crate::Market::SH,
                    abbr: res["BOND_ABBR"].as_str().unwrap().into(),
                    par_value: res["FACE_VALUE"].as_str().unwrap().parse().unwrap(),
                    cp_type,
                    interest_type,
                    cp_rate: (res["FACE_RATE"]
                        .as_str()
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.)
                        * 100.)
                        .round()
                        / 10000.,
                    base_rate,
                    rate_spread,
                    inst_freq,
                    carry_date: NaiveDate::parse_from_str(
                        res["START_DATE"].as_str().unwrap(),
                        "%Y%m%d",
                    )?,
                    maturity_date: NaiveDate::parse_from_str(
                        res["END_DATE"].as_str().unwrap(),
                        "%Y%m%d",
                    )?,
                    day_count: BondDayCount::default(),
                };
                return Ok(bond);
            }
        }
        bail!("Can not find bond {} in search result", code)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_sh_download_from_sse() -> Result<()> {
//         let bond = Bond::sh_download_from_sse("019743").await?;
//         dbg!(bond);
//         Ok(())
//     }
// }
