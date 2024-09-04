use super::Bond;
use anyhow::Result;
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
    pub fn read_json(code: &str, path: Option<&Path>) -> Result<Self> {
        let bond = if let Some(path) = path {
            let file = File::open(path)?;
            serde_json::from_reader(BufReader::new(file))?
        } else {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("bonds_info/{}.json", code));
            let file = File::open(path)?;
            serde_json::from_reader(BufReader::new(file))?
        };
        Ok(bond)
    }
}
