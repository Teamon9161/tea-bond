// china_money / sse public-web fallbacks are unused for now — Wind is the only
// download path. Left on disk (not `mod`-included) so they're easy to bring back.
#[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
mod wind;

use super::Bond;
use anyhow::{Result, bail};

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

    /// Make sure every bond `code` in `codes` is available in the local cache,
    /// deduplicating first and downloading whatever is missing from Wind in a
    /// single batch call. Errors (rather than silently degrading) if any code
    /// still can't be resolved afterwards — callers that vectorize over a bond
    /// column should call this once up front instead of tolerating missing
    /// bond info row by row.
    #[cfg(all(feature = "wind", any(target_os = "linux", target_os = "windows")))]
    pub fn ensure_cached<'a>(codes: impl IntoIterator<Item = &'a str>) -> Result<()> {
        use std::collections::HashSet;

        let mut seen: HashSet<String> = HashSet::new();
        let mut missing: Vec<(String, super::Market)> = Vec::new();
        let mut bad_format: Vec<String> = Vec::new();

        for code in codes {
            if code.is_empty() {
                continue;
            }
            let (bare, market_str) = code.split_once('.').unwrap_or((code, "IB"));
            let normalized = format!("{bare}.{market_str}");
            if !seen.insert(normalized.clone()) {
                continue;
            }
            if Self::read(&normalized, None, false).is_ok() {
                continue;
            }
            match market_str.parse::<super::Market>() {
                Ok(market) => missing.push((normalized, market)),
                Err(_) => bad_format.push(normalized),
            }
        }

        if !bad_format.is_empty() {
            bail!("Cannot resolve market for bond code(s): {}", bad_format.join(", "));
        }
        if missing.is_empty() {
            return Ok(());
        }

        println!(
            "Downloading {} bond(s) info from Wind: {}",
            missing.len(),
            missing
                .iter()
                .map(|(code, _)| code.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let wind_codes: Vec<String> = missing
            .iter()
            .map(|(code, market)| {
                let bare = code.split_once('.').map(|(b, _)| b).unwrap_or(code);
                format!("{bare}.{}", wind::market_suffix(*market))
            })
            .collect();
        let wind_code_refs: Vec<&str> = wind_codes.iter().map(String::as_str).collect();
        let results = Self::wind_download_batch(&wind_code_refs)?;

        let mut ok_bonds: Vec<Bond> = Vec::with_capacity(missing.len());
        let mut failed: Vec<String> = Vec::new();
        for ((code, _), result) in missing.iter().zip(results) {
            match result {
                Ok(bond) => ok_bonds.push(bond),
                Err(e) => failed.push(format!("{code}: {e}")),
            }
        }
        // Persist to the in-memory cache immediately, flush to disk once at the end.
        for bond in &ok_bonds {
            bond.save_disk(false)?;
        }
        if let Some(last) = ok_bonds.last() {
            last.save_disk(true)?;
        }

        if !failed.is_empty() {
            bail!(
                "Failed to download {} bond(s) info from Wind:\n{}",
                failed.len(),
                failed.join("\n")
            );
        }
        Ok(())
    }

    /// No Wind download available on this platform — just check everything's
    /// already cached locally instead of silently letting callers compute with
    /// missing bond info.
    #[cfg(not(all(feature = "wind", any(target_os = "linux", target_os = "windows"))))]
    pub fn ensure_cached<'a>(codes: impl IntoIterator<Item = &'a str>) -> Result<()> {
        use std::collections::HashSet;

        let mut seen: HashSet<String> = HashSet::new();
        let mut missing: Vec<String> = Vec::new();
        for code in codes {
            if code.is_empty() {
                continue;
            }
            let normalized = if code.contains('.') {
                code.to_string()
            } else {
                format!("{code}.IB")
            };
            if !seen.insert(normalized.clone()) {
                continue;
            }
            if Self::read(&normalized, None, false).is_err() {
                missing.push(normalized);
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        bail!(
            "Missing bond info for {} bond(s) and Wind download is not available on this \
             platform: {}",
            missing.len(),
            missing.join(", ")
        );
    }
}
