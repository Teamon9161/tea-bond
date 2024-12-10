use super::Bond;
use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

impl Bond {
    #[inline]
    /// 从本地json文件读取Bond
    ///
    /// ```
    /// use tea_bond::Bond;
    /// let bond = Bond::read_json("240006.IB", None).unwrap();
    /// assert_eq!(bond.code(), "240006");
    /// assert_eq!(bond.cp_rate_1st, 0.0228)
    /// ```
    pub fn read_json(code: impl AsRef<str>, path: Option<&Path>) -> Result<Self> {
        let code = code.as_ref();
        let code: std::sync::Arc<str> = if !code.contains('.') {
            eprintln!(
                "code doesn't contain market type, use IB as default: {}",
                code
            );
            format!("{}.IB", code).into()
        } else {
            code.into()
        };
        let path = if let Some(path) = path {
            PathBuf::from(path).join(format!("{}.json", code))
        } else {
            if let Ok(path) = std::env::var("BONDS_INFO_PATH") {
                PathBuf::from(path).join(format!("{}.json", code))
            } else {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("bonds_info/{}.json", code))
            }
        };
        let file = File::open(&path)
            .map_err(|_| anyhow!("Read bond {} error: Can not open {:?}", code, &path))?;
        let bond = serde_json::from_reader(BufReader::new(file))?;
        Ok(bond)
    }
}
