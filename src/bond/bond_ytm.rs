use std::sync::Arc;

use super::Bond;

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
    pub fn with_ytm(self, ytm: f64) -> Self {
        BondYtm { ytm, ..self }
    }
}
