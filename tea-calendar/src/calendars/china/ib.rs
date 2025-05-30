use crate::{Calendar, china::SSE, date};
use chrono::NaiveDate;
// 银行间债券市场
#[derive(Debug, Clone, Copy, Default)]
pub struct IB;

/// 中国工作周末列表
pub const WORKING_WEEKENDS: &[NaiveDate] = &[
    // 2005
    date!(2005, 2, 5),
    date!(2005, 2, 6),
    date!(2005, 4, 30),
    date!(2005, 5, 8),
    date!(2005, 10, 8),
    date!(2005, 10, 9),
    date!(2005, 12, 31),
    // 2006年
    date!(2006, 1, 28),
    date!(2006, 4, 29),
    date!(2006, 4, 30),
    date!(2006, 9, 30),
    date!(2006, 12, 30),
    date!(2006, 12, 31),
    // 2007年
    date!(2007, 2, 17),
    date!(2007, 2, 25),
    date!(2007, 4, 28),
    date!(2007, 4, 29),
    date!(2007, 9, 29),
    date!(2007, 9, 30),
    date!(2007, 12, 29),
    // 2008年
    date!(2008, 2, 2),
    date!(2008, 2, 3),
    date!(2008, 5, 4),
    date!(2008, 9, 27),
    date!(2008, 9, 28),
    // 2009年
    date!(2009, 1, 4),
    date!(2009, 1, 24),
    date!(2009, 2, 1),
    date!(2009, 5, 31),
    date!(2009, 9, 27),
    date!(2009, 10, 10),
    // 2010年
    date!(2010, 2, 20),
    date!(2010, 2, 21),
    date!(2010, 6, 12),
    date!(2010, 6, 13),
    date!(2010, 9, 19),
    date!(2010, 9, 25),
    date!(2010, 9, 26),
    date!(2010, 10, 9),
    // 2011年
    date!(2011, 1, 30),
    date!(2011, 2, 12),
    date!(2011, 4, 2),
    date!(2011, 10, 8),
    date!(2011, 10, 9),
    date!(2011, 12, 31),
    // 2012年
    date!(2012, 1, 21),
    date!(2012, 1, 29),
    date!(2012, 3, 31),
    date!(2012, 4, 1),
    date!(2012, 4, 28),
    date!(2012, 9, 29),
    // 2013年
    date!(2013, 1, 5),
    date!(2013, 1, 6),
    date!(2013, 2, 16),
    date!(2013, 2, 17),
    date!(2013, 4, 7),
    date!(2013, 4, 27),
    date!(2013, 4, 28),
    date!(2013, 6, 8),
    date!(2013, 6, 9),
    date!(2013, 9, 22),
    date!(2013, 9, 29),
    date!(2013, 10, 12),
    // 2014年
    date!(2014, 1, 26),
    date!(2014, 2, 8),
    date!(2014, 5, 4),
    date!(2014, 9, 28),
    date!(2014, 10, 11),
    // 2015年
    date!(2015, 1, 4),
    date!(2015, 2, 15),
    date!(2015, 2, 28),
    date!(2015, 9, 6),
    date!(2015, 10, 10),
    // 2016年
    date!(2016, 2, 6),
    date!(2016, 2, 14),
    date!(2016, 6, 12),
    date!(2016, 9, 18),
    date!(2016, 10, 8),
    date!(2016, 10, 9),
    // 2017年
    date!(2017, 1, 22),
    date!(2017, 2, 4),
    date!(2017, 4, 1),
    date!(2017, 5, 27),
    date!(2017, 9, 30),
    // 2018年
    date!(2018, 2, 11),
    date!(2018, 2, 24),
    date!(2018, 4, 8),
    date!(2018, 4, 28),
    date!(2018, 9, 29),
    date!(2018, 9, 30),
    date!(2018, 12, 29),
    // 2019年
    date!(2019, 2, 2),
    date!(2019, 2, 3),
    date!(2019, 4, 28),
    date!(2019, 5, 5),
    date!(2019, 9, 29),
    date!(2019, 10, 12),
    // 2020年
    date!(2020, 1, 19),
    date!(2020, 4, 26),
    date!(2020, 5, 9),
    date!(2020, 6, 28),
    date!(2020, 9, 27),
    date!(2020, 10, 10),
    // 2021年
    date!(2021, 2, 7),
    date!(2021, 2, 20),
    date!(2021, 4, 25),
    date!(2021, 5, 8),
    date!(2021, 9, 18),
    date!(2021, 9, 26),
    date!(2021, 10, 9),
    // 2022年
    date!(2022, 1, 29),
    date!(2022, 1, 30),
    date!(2022, 4, 2),
    date!(2022, 4, 24),
    date!(2022, 5, 7),
    date!(2022, 10, 8),
    date!(2022, 10, 9),
    // 2023年
    date!(2023, 1, 28),
    date!(2023, 1, 29),
    date!(2023, 4, 23),
    date!(2023, 5, 6),
    date!(2023, 6, 25),
    date!(2023, 10, 7),
    date!(2023, 10, 8),
    // 2024年
    date!(2024, 2, 4),
    date!(2024, 2, 9),
    date!(2024, 2, 18),
    date!(2024, 4, 7),
    date!(2024, 4, 28),
    date!(2024, 5, 11),
    date!(2024, 9, 14),
    date!(2024, 9, 29),
    date!(2024, 10, 12),
    // 2025年
    date!(2025, 1, 26),
    date!(2025, 2, 8),
    date!(2025, 4, 27),
    date!(2025, 9, 28),
    date!(2025, 10, 11),
];

impl Calendar for IB {
    #[inline]
    fn is_business_day(&self, date: NaiveDate) -> bool {
        SSE.is_business_day(date) || WORKING_WEEKENDS.contains(&date)
    }
}
