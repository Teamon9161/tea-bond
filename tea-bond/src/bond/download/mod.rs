mod china_money;
mod sse;
#[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
mod wind;

use super::{Bond, Market};
use anyhow::{Result, bail};

impl Bond {
    pub async fn download(code: &str) -> Result<Bond> {
        println!("Download bond: {code}");
        let (code, market) = if let Some((code, market)) = code.split_once(".") {
            (code, market.parse()?)
        } else {
            (code, Market::IB)
        };
        // 本机装了 Wind 终端时优先走 Wind，字段更全（含发行价），失败再退回公开接口
        #[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
        {
            let wind_code = format!("{}.{}", code, wind::market_suffix(market));
            match Self::wind_download(&wind_code) {
                Ok(bond) => return Ok(bond),
                Err(e) => eprintln!("Download bond {wind_code} from Wind failed: {e}"),
            }
        }
        match market {
            Market::IB => Self::ib_download_from_china_money(code, None).await,
            Market::SH => Self::sh_download_from_sse(code).await,
            market => bail!(
                "Download bond from Market {:#?} is not supported yet",
                market
            ),
        }
    }
}
