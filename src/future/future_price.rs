use super::Future;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FuturePrice {
    pub future: Arc<Future>,
    pub price: f64,
}

impl Default for FuturePrice {
    fn default() -> Self {
        FuturePrice {
            future: Arc::new(Future::default()),
            price: f64::NAN,
        }
    }
}
