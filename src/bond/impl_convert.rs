use super::bond_ytm::BondYtm;
use super::Bond;
use anyhow::{Error, Result};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

impl<S: TryInto<Bond>> TryFrom<(S, f64)> for BondYtm {
    type Error = S::Error;

    #[inline]
    fn try_from(t: (S, f64)) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            bond: Arc::new(t.0.try_into()?),
            ytm: t.1,
        })
    }
}

impl TryFrom<(Arc<Bond>, f64)> for BondYtm {
    type Error = Error;

    #[inline]
    fn try_from(t: (Arc<Bond>, f64)) -> Result<Self> {
        Ok(Self {
            bond: t.0,
            ytm: t.1,
        })
    }
}
