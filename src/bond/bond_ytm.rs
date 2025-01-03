use super::Bond;
use super::CachedBond;
use anyhow::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct BondYtm {
    pub bond: CachedBond,
    pub ytm: f64,
}

impl Default for BondYtm {
    fn default() -> Self {
        BondYtm {
            bond: Bond::default().into(),
            ytm: f64::NAN,
        }
    }
}

impl BondYtm {
    #[inline]
    pub fn new(bond: impl Into<Bond>, ytm: f64) -> Self {
        BondYtm {
            bond: CachedBond::from_bond(bond.into()),
            ytm,
        }
    }

    #[inline]
    pub fn try_new(bond: impl TryInto<Bond, Error = Error>, ytm: f64) -> Result<Self> {
        Ok(Self::new(bond.try_into()?, ytm))
    }

    #[inline]
    pub fn with_ytm(self, ytm: f64) -> Self {
        BondYtm { ytm, ..self }
    }
}
