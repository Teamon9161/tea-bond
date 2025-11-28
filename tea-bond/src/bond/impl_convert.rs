use super::Bond;
use super::{CachedBond, bond_ytm::BondYtm};
use crate::SmallStr;
use anyhow::{Error, Result};
use std::borrow::Cow;
// use std::path::{Path, PathBuf};
use std::sync::Arc;

impl TryFrom<&str> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: &str) -> Result<Self> {
        CachedBond::new(s, None)
    }
}

impl TryFrom<usize> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: usize) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl TryFrom<i32> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: i32) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl TryFrom<&String> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: &String) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<String> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: String) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<SmallStr> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: SmallStr) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<Cow<'_, str>> for CachedBond {
    type Error = Error;

    #[inline]
    fn try_from(s: Cow<'_, str>) -> Result<Self> {
        s.as_ref().try_into()
    }
}

impl From<Bond> for CachedBond {
    #[inline]
    fn from(bond: Bond) -> CachedBond {
        Arc::new(bond).into()
    }
}

impl From<Arc<Bond>> for CachedBond {
    #[inline]
    fn from(bond: Arc<Bond>) -> Self {
        Self::from_bond(bond)
    }
}

impl<S: TryInto<CachedBond>> TryFrom<(S, f64)> for BondYtm {
    type Error = S::Error;

    #[inline]
    fn try_from(t: (S, f64)) -> Result<Self, Self::Error> {
        BondYtm::try_new(t.0, t.1)
    }
}
