use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::future_price::FuturePrice;
use super::Future;

impl From<&str> for Future {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Future {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for Future {
    #[inline]
    fn from(s: &String) -> Self {
        Self::new(s)
    }
}

impl From<Cow<'_, str>> for Future {
    #[inline]
    fn from(s: Cow<'_, str>) -> Self {
        Self::new(s)
    }
}

impl From<&Path> for Future {
    #[inline]
    fn from(path: &Path) -> Self {
        let code = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        Self::new(code)
    }
}

impl From<&PathBuf> for Future {
    #[inline]
    fn from(path: &PathBuf) -> Self {
        Self::from(path.as_path())
    }
}

impl<F: Into<Future>> From<(F, f64)> for FuturePrice {
    #[inline]
    fn from(t: (F, f64)) -> Self {
        FuturePrice {
            future: Arc::new(t.0.into()),
            price: t.1,
        }
    }
}

impl From<(Arc<Future>, f64)> for FuturePrice {
    #[inline]
    fn from(t: (Arc<Future>, f64)) -> Self {
        FuturePrice {
            future: t.0,
            price: t.1,
        }
    }
}
