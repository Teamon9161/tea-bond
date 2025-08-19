use super::Bond;
use super::CachedBond;
use anyhow::Result;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct BondYtm {
    pub bond: CachedBond,
    ytm: f64,
}

impl Default for BondYtm {
    fn default() -> Self {
        BondYtm {
            bond: Bond::default().into(),
            ytm: f64::NAN,
        }
    }
}

impl Deref for BondYtm {
    type Target = Bond;

    fn deref(&self) -> &Self::Target {
        &self.bond
    }
}

impl BondYtm {
    #[inline]
    pub fn new(bond: impl Into<CachedBond>, ytm: f64) -> Self {
        let bond = bond.into();
        let ytm = bond.check_ytm(ytm);
        BondYtm { bond, ytm }
    }

    #[inline]
    pub fn try_new<B: TryInto<CachedBond>>(bond: B, ytm: f64) -> Result<Self, B::Error> {
        Ok(Self::new(bond.try_into()?, ytm))
    }

    #[inline]
    pub fn with_ytm(self, ytm: f64) -> Self {
        let ytm = self.bond.check_ytm(ytm);
        BondYtm { ytm, ..self }
    }

    #[inline]
    pub fn ytm(&self) -> f64 {
        self.ytm
    }
}
