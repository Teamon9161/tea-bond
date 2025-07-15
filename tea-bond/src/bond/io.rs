use super::Bond;
use anyhow::Result;
use std::{
    borrow::Cow,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

// pub static mut BONDS_INFO_PATH: Option<PathBuf> = None;

impl Bond {
    pub fn get_save_path(code: &str, path: Option<&Path>) -> PathBuf {
        if let Some(path) = path {
            PathBuf::from(path).join(format!("{code}.json"))
        } else if let Ok(path) = std::env::var("BONDS_INFO_PATH") {
            PathBuf::from(path).join(format!("{code}.json"))
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("bonds_info/{code}.json"))
        }
    }

    /// 从本地json文件读取Bond
    ///
    /// ```
    /// use tea_bond::Bond;
    /// let bond = Bond::read_json("240006.IB", None).unwrap();
    /// assert_eq!(bond.code(), "240006");
    /// assert_eq!(bond.cp_rate, 0.0228)
    /// ```
    #[allow(clippy::collapsible_else_if)]
    pub fn read_json(code: impl AsRef<str>, path: Option<&Path>) -> Result<Self> {
        let code = code.as_ref();
        let code: Cow<'_, str> = if !code.contains('.') {
            // dbg!(
            //     "Read bond from json file,code doesn't contain market type, use IB as default: {}",
            //     code
            // );
            format!("{code}.IB").into()
        } else {
            code.into()
        };
        let path = Bond::get_save_path(&code, path);
        if let Ok(file) = File::open(&path) {
            Ok(serde_json::from_reader(BufReader::new(file))?)
        } else {
            // try download bond from china money
            #[cfg(feature = "download")]
            {
                let rt = tokio::runtime::Runtime::new()?;
                let bond = rt.block_on(async { Self::download(&code).await })?;
                bond.save(&path)?;
                Ok(bond)
            }
            #[cfg(not(feature = "download"))]
            bail!("Read bond {} error: Can not open {:?}", code, &path)
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
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
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
            std::fs::create_dir_all(parent)?;
        }

        // Create the file and write the bond data
        let file = File::create(&final_path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
}
