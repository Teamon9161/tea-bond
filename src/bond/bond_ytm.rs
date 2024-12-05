use super::Bond;
use anyhow::{Error, Result};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct BondYtm {
    pub bond: Arc<Bond>,
    pub ytm: f64,
}

impl Default for BondYtm {
    fn default() -> Self {
        BondYtm {
            bond: Arc::new(Bond::default()),
            ytm: f64::NAN,
        }
    }
}

impl BondYtm {
    #[inline]
    pub fn new(bond: impl Into<Bond>, ytm: f64) -> Self {
        BondYtm {
            bond: Arc::new(bond.into()),
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
