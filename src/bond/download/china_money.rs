use crate::bond::{Bond, BondDayCount, CouponType, InterestType};
use crate::SmallStr;
use anyhow::{anyhow, bail, Result};

const IB_SEARCH_URL: &str =
    "https://iftp.chinamoney.com.cn/ses/rest/cm-u-notice-ses-cn/queryBondOrEnty";
const IB_BOND_DETAIL_URL: &str =
    "https://iftp.chinamoney.com.cn/ags/ms/cm-u-bond-md/BondDetailInfo";

// const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36";
// const USER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

fn ib_get_coupon_interest_type(typ: &str) -> Result<(CouponType, InterestType)> {
    match typ {
        "附息式固定利率" => Ok((CouponType::CouponBear, InterestType::Fixed)),
        "零息式" => Ok((CouponType::OneTime, InterestType::Fixed)),
        "贴现式" => Ok((CouponType::ZeroCoupon, InterestType::Zero)),
        "未计息" => Ok((CouponType::ZeroCoupon, InterestType::Zero)),
        typ => bail!(
            "Cannot infer coupon type and interest type from IB: {}",
            typ
        ),
    }
}

fn ib_get_inst_freq(cp_type: CouponType, freq: &str) -> Result<i32> {
    match cp_type {
        CouponType::CouponBear => match freq {
            "年" => Ok(1),
            "半年" => Ok(2),
            _ => bail!("Cannot infer inst freq from IB for coupon type: {}", freq),
        },
        CouponType::OneTime => Ok(1),
        CouponType::ZeroCoupon => Ok(0),
    }
}

impl Bond {
    pub async fn ib_download_from_china_money(code: &str) -> Result<Bond> {
        let client = reqwest::ClientBuilder::new()
            .use_rustls_tls()
            // .user_agent(USER_AGENT)
            .build()?;
        // search bond defined code using code
        let url = format!("{}?searchValue={}&verify=false", IB_SEARCH_URL, code);
        // let search_res = client.post(url).send().await?.text().await?;
        // dbg!(search_res);
        let search_res: serde_json::Value = client.post(url).send().await?.json().await?;
        // println!("{:#?}", search_res);
        let data = &search_res["data"]
            .get("bondOrEntyResult")
            .ok_or_else(|| anyhow!("No bond found for code: {}, search data is null", code))?;
        let defined_code = if data.is_null() {
            bail!(
                "No bond found for code: {}, bond or enty result is null",
                code
            );
        } else {
            let data = data.as_object().unwrap();
            if data.is_empty() {
                bail!("Cann't find code: {} in IB search result", code);
            }
            data["definedCode"].as_str().unwrap()
        };
        // println!("defined_code: {}", defined_code);
        // download bond detail info using defined code
        let info_result: serde_json::Value = client
            .post(IB_BOND_DETAIL_URL)
            .form(&serde_json::json!({
                "bondDefinedCode": defined_code,
            }))
            .send()
            .await?
            .json()
            .await?;
        let info = &info_result["data"]["bondBaseInfo"];
        // println!("{info:#?}");
        if info["bondCode"].as_str().unwrap() != code {
            bail!("Downloaded bond {} failed", code);
        } else {
            let (cp_type, interest_type) =
                ib_get_coupon_interest_type(info["couponType"].as_str().unwrap())?;
            let (base_rate, rate_spread) = if let InterestType::Floating = interest_type {
                bail!("Get base rate & rate spread for floating bond in IB is not implemented yet");
            } else {
                (None, None)
            };
            let bond = Bond {
                bond_code: SmallStr::new(code) + ".IB",
                mkt: crate::Market::IB,
                abbr: info["bondName"].as_str().unwrap().into(),
                par_value: info["parValue"].as_str().unwrap().parse().unwrap(),
                cp_type,
                interest_type,
                cp_rate_1st: (info["parCouponRate"]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap_or(0.)
                    * 100.)
                    .round()
                    / 10000.,
                base_rate,
                rate_spread,
                inst_freq: ib_get_inst_freq(cp_type, info["couponFrqncy"].as_str().unwrap())?,
                carry_date: info["frstValueDate"].as_str().unwrap().parse().unwrap(),
                maturity_date: info["mrtyDate"].as_str().unwrap().parse().unwrap(),
                day_count: BondDayCount::default(),
            };
            if bond.cp_rate_1st != 0. {
                assert!(bond.cp_type != CouponType::ZeroCoupon);
                assert!(bond.inst_freq != 0);
            }
            Ok(bond)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ib_download() -> Result<()> {
        let bond = Bond::download("249946.IB").await?;
        dbg!(bond);
        Ok(())
    }
}
