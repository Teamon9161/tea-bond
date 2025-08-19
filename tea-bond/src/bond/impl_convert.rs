use super::Bond;
use super::{CachedBond, bond_ytm::BondYtm};
use crate::SmallStr;
use anyhow::{Error, Result};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

impl TryFrom<&str> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: &str) -> Result<Self> {
        Self::read_json(s, None)
    }
}

impl TryFrom<usize> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: usize) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl TryFrom<i32> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: i32) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl TryFrom<&String> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: &String) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<String> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: String) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<SmallStr> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: SmallStr) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<Cow<'_, str>> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: Cow<'_, str>) -> Result<Self> {
        s.as_ref().try_into()
    }
}

impl TryFrom<&Path> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(path: &Path) -> Result<Self> {
        let code = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let folder = path.parent();
        Self::read_json(code, folder)
    }
}

impl TryFrom<&PathBuf> for Bond {
    type Error = Error;

    #[inline]
    fn try_from(s: &PathBuf) -> Result<Self> {
        Self::try_from(s.as_path())
    }
}

impl From<(CachedBond, f64)> for BondYtm {
    #[inline]
    fn from(t: (CachedBond, f64)) -> Self {
        BondYtm::new(t.0, t.1)
    }
}
