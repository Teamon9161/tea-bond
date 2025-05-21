use super::sse::SSE;
use crate::Calendar;
use chrono::NaiveDate;

// 中国金融期货交易所
#[derive(Debug, Clone, Copy, Default)]
pub struct CFFEX;

impl Calendar for CFFEX {
    #[inline]
    fn is_business_day(&self, date: NaiveDate) -> bool {
        SSE.is_business_day(date)
    }
}

// 中国深圳证券交易所
#[derive(Debug, Clone, Copy, Default)]
pub struct SZE;

impl Calendar for SZE {
    #[inline]
    fn is_business_day(&self, date: NaiveDate) -> bool {
        SSE.is_business_day(date)
    }
}
