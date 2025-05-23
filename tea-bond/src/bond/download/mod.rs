mod china_money;
mod sse;
use super::{Bond, Market};
use anyhow::{bail, Result};

impl Bond {
    pub async fn download(code: &str) -> Result<Bond> {
        println!("Download bond: {code}");
        let (code, market) = if let Some((code, market)) = code.split_once(".") {
            (code, market.parse()?)
        } else {
            (code, Market::IB)
        };
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
