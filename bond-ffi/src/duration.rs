use super::utils::get_str;
use std::ffi::c_void;
use tea_time::{DateTime, TimeDelta};

#[unsafe(no_mangle)]
pub extern "C" fn parse_duration(ptr: *mut u8, len: usize) -> *mut c_void {
    let duration_str = get_str(ptr, len);
    let duration = TimeDelta::parse(duration_str).unwrap();
    Box::into_raw(Box::new(duration)) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn datetime_sub_datetime(dt1: i64, dt2: i64) -> *mut c_void {
    let dt1: DateTime = dt1.into();
    let dt2: DateTime = dt2.into();
    Box::into_raw(Box::new(dt1 - dt2)) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn datetime_add_duration(dt: i64, duration: *const TimeDelta) -> i64 {
    let duration = unsafe { &*duration };
    let dt: DateTime = dt.into();
    (dt + *duration).0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn datetime_sub_duration(dt: i64, duration: *const TimeDelta) -> i64 {
    let dt: DateTime = dt.into();
    let duration = unsafe { &*duration };
    (dt - *duration).0
}
