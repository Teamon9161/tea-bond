mod ib;
mod others;
mod sse;

use chrono::NaiveDate;
pub use ib::IB;
pub use others::{CFFEX, SZE};
pub use sse::SSE;

use crate::Calendar;

#[derive(Debug, Clone, Copy, Default)]
pub struct China<M>(M);

impl<M: Calendar> Calendar for China<M> {
    #[inline]
    fn is_business_day(&self, date: NaiveDate) -> bool {
        self.0.is_business_day(date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Calendar, date};

    #[test]
    fn test_sse() {
        assert_eq!(SSE.is_business_day(date!(2025, 5, 21)), true); // working day
        assert_eq!(SSE.is_business_day(date!(2025, 5, 2)), false); // holiday
        assert_eq!(SSE.is_business_day(date!(2025, 5, 17)), false); // weekend
        assert_eq!(SSE.is_business_day(date!(2025, 4, 27)), false); // working weekend
    }

    #[test]
    fn test_ib() {
        assert_eq!(IB.is_business_day(date!(2025, 5, 21)), true); // working day
        assert_eq!(IB.is_business_day(date!(2025, 5, 2)), false); // holiday
        assert_eq!(IB.is_business_day(date!(2025, 5, 17)), false); // weekend
        assert_eq!(IB.is_business_day(date!(2025, 4, 27)), true); // working weekend
    }
}
