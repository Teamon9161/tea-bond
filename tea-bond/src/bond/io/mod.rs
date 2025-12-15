#[cfg(feature = "duckdb")]
mod duck;
mod persist;
mod wind_sql_row;

use super::Bond;
use anyhow::Result;
pub use persist::free_bond_map;
use std::{
    borrow::Cow,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};
pub use wind_sql_row::WindSqlRow;

#[inline]
pub fn default_dir() -> PathBuf {
    match std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        Some(home) => PathBuf::from(home).join("tea-bond"),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    }
}

impl Bond {
    pub fn get_json_save_path(code: &str, path: Option<&Path>) -> PathBuf {
        let base_dir = if let Some(path) = path {
            PathBuf::from(path)
        } else if let Ok(path) = std::env::var("BONDS_INFO_PATH") {
            PathBuf::from(path)
        } else {
            default_dir().join("bonds_info")
        };

        if let Err(err) = fs::create_dir_all(&base_dir) {
            eprintln!("Failed to create bonds_info dir {:?}: {}", base_dir, err);
        }

        base_dir.join(format!("{code}.json"))
    }

    pub fn read(code: impl AsRef<str>, path: Option<&Path>, download: bool) -> Result<Arc<Self>> {
        let code = code.as_ref();
        let code: Cow<'_, str> = if !code.contains('.') {
            format!("{code}.IB").into()
        } else {
            code.into()
        };
        #[cfg(feature = "duckdb")]
        {
            use duck::DUCKDB_TABLE_PATH;
            if let Ok(con) = duckdb::Connection::open(DUCKDB_TABLE_PATH.as_str()) {
                if let Ok(bond) = Bond::read_duckdb(&con, None, code.as_ref()) {
                    bond.save_disk(false)?; // save to cache
                    return Ok(Arc::new(bond));
                }
            }
        }
        if let Ok(bond) = Bond::read_disk(&code) {
            return Ok(bond);
        }
        let bond = Bond::read_json(code, path, download)?;
        bond.save_disk(false)?; // save to cache
        Ok(Arc::new(bond))
    }

    /// 从本地json文件读取Bond
    ///
    /// ```
    /// use tea_bond::Bond;
    /// let bond = Bond::read_json("240006.IB", None, false).unwrap();
    /// assert_eq!(bond.code(), "240006");
    /// assert_eq!(bond.cp_rate, 0.0228)
    /// ```
    #[allow(clippy::collapsible_else_if, unused_variables)]
    pub fn read_json(code: impl AsRef<str>, path: Option<&Path>, download: bool) -> Result<Self> {
        let code = code.as_ref();
        let code: Cow<'_, str> = if !code.contains('.') {
            format!("{code}.IB").into()
        } else {
            code.into()
        };
        let path = Bond::get_json_save_path(&code, path);
        if let Ok(file) = File::open(&path) {
            Ok(serde_json::from_reader(BufReader::new(file))?)
        } else {
            // try download bond from china money
            #[cfg(feature = "download")]
            if download {
                let rt = tokio::runtime::Runtime::new()?;
                let bond = rt.block_on(async { Self::download(&code).await })?;
                // bond.save_json(&path)?;
                bond.save_disk(true)?;
                return Ok(bond);
            }
            // #[cfg(not(feature = "download"))]
            anyhow::bail!("Read bond {} error: Can not open {:?}", code, &path)
        }
    }

    /// Saves the `Bond` instance to a JSON file at the specified path.
    ///
    /// If the provided path is a directory, the bond will be saved as a JSON file
    /// named after the bond's code (e.g., `{bond_code}.json`) within that directory.
    /// If the path is a file, the bond will be saved directly to that file.
    ///
    /// The method ensures that the parent directory of the final path exists by
    /// creating it if necessary. The bond data is serialized to JSON and written
    /// to the file in a pretty-printed format.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the bond should be saved. This can be either a directory
    ///   or a file path.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the bond is successfully saved. If an error occurs during
    /// directory creation, file creation, or JSON serialization, an error is returned.
    #[inline]
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        println!("Save bond: {} to path {:?}", self.code(), &path);

        // Determine if the path is a directory or a file
        let final_path = if path.is_dir() {
            // If it's a directory, append the bond code with .json extension
            path.join(format!("{}.json", self.bond_code()))
        } else {
            // If it's a file, use the path as is
            path.to_path_buf()
        };

        // Create the parent directory if it doesn't exist
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the file and write the bond data
        let file = File::create(&final_path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
}
