use super::Future;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
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

impl Deref for FuturePrice {
    type Target = Future;

    fn deref(&self) -> &Self::Target {
        &self.future
    }
}

impl FuturePrice {
    #[inline]
    pub fn new(future: impl Into<Future>, price: f64) -> Self {
        FuturePrice {
            future: Arc::new(future.into()),
            price,
        }
    }

    #[inline]
    pub fn with_price(self, price: f64) -> Self {
        FuturePrice { price, ..self }
    }
}
