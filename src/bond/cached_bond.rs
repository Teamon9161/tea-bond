use super::Bond;
use crate::SmallStr;
use anyhow::Result;
use parking_lot::Mutex;
use std::ops::Deref;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, LazyLock},
};

// dict to cache bonds
static BOND_DICT: LazyLock<Mutex<HashMap<SmallStr, Arc<Bond>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A cached bond that wraps an `Arc<Bond>` for efficient sharing and caching.
///
/// This struct is used to store bonds in a global cache (`BOND_DICT`) to avoid
/// redundant reads or computations. It implements `Deref` to allow direct access
/// to the underlying `Bond` type.
#[derive(Clone, PartialEq, Eq)]
pub struct CachedBond(Arc<Bond>);

/// Clears the global bond cache (`BOND_DICT`), freeing all cached bonds.
#[inline]
pub fn free_bond_dict() {
    BOND_DICT.lock().clear();
}

impl Deref for CachedBond {
    type Target = Bond;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for CachedBond {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Bond> for CachedBond {
    #[inline]
    fn from(bond: Bond) -> Self {
        Self::from_bond(bond)
    }
}

impl From<Arc<Bond>> for CachedBond {
    #[inline]
    fn from(bond: Arc<Bond>) -> Self {
        Self::from_bond(bond)
    }
}

impl CachedBond {
    /// Creates a new `CachedBond` from a bond code and an optional path.
    ///
    /// If the bond is already cached, it returns the cached instance. Otherwise,
    /// it reads the bond from the specified path (or the default path if `None`),
    /// caches it, and returns the new instance.
    ///
    /// # Arguments
    /// * `bond_code` - The code identifying the bond.
    /// * `path` - An optional path to read the bond from. If `None`, the default path is used.
    ///
    /// # Returns
    /// A `Result` containing the `CachedBond` if successful, or an error if the bond could not be read.
    pub fn new(bond_code: &str, path: Option<&Path>) -> Result<Self> {
        {
            // Check if the bond is already in the cache
            let bond_dict = BOND_DICT.lock();
            if let Some(bond) = bond_dict.get(bond_code) {
                return Ok(Self(bond.clone()));
            }
        }
        // Read the bond from the specified path or default path
        let bond_rs = Arc::new(Bond::read_json(bond_code, path)?);
        {
            // Insert the bond into the cache
            BOND_DICT.lock().insert(bond_code.into(), bond_rs.clone());
        }
        Ok(Self(bond_rs))
    }

    /// Creates a `CachedBond` from a bond (or an `Arc<Bond>`) and caches it.
    ///
    /// If the bond is already cached, it returns the cached instance. Otherwise,
    /// it caches the bond and returns the new instance.
    ///
    /// # Arguments
    /// * `bond` - A bond or an `Arc<Bond>` to cache.
    ///
    /// # Returns
    /// A `CachedBond` instance.
    #[inline]
    pub fn from_bond(bond: impl Into<Arc<Bond>>) -> Self {
        let bond = bond.into();
        let code = bond.bond_code();
        {
            // Check if the bond is already in the cache
            let mut bond_dict = BOND_DICT.lock();
            if bond_dict.get(code).is_none() {
                // Insert the bond into the cache
                bond_dict.insert(code.into(), bond.clone());
            }
        }
        Self(bond)
    }
}

impl<'de> serde::Deserialize<'de> for CachedBond {
    #[inline]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bond = Bond::deserialize(deserializer)?;
        Ok(Self::from_bond(bond))
    }
}