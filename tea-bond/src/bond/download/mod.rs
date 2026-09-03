// china_money / sse public-web fallbacks are unused for now — Wind is the only
// download path. Left on disk (not `mod`-included) so they're easy to bring back.
#[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
mod wind;

use super::Bond;
use anyhow::Result;
#[cfg(not(all(feature = "wind", any(target_os = "linux", target_os = "windows"))))]
use anyhow::bail;

impl Bond {
    pub async fn download(code: &str) -> Result<Bond> {
        println!("Download bond: {code}");
        #[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
        {
            let (bare_code, market) = if let Some((bare_code, market)) = code.split_once(".") {
                (bare_code, market.parse()?)
            } else {
                (code, super::Market::IB)
            };
            let wind_code = format!("{}.{}", bare_code, wind::market_suffix(market));
            return Self::wind_download(&wind_code)
                .map_err(|e| anyhow::anyhow!("Download bond {wind_code} from Wind failed: {e}"));
        }
        #[cfg(not(all(feature = "wind", any(target_os = "linux", target_os = "windows"))))]
        bail!(
            "Download bond {code} is not supported on this platform: \
             Wind download only works on Windows/Linux"
        );
    }
}
