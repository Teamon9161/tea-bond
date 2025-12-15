use std::ffi::c_void;

use chrono::{Datelike, Local, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
pub type FfiDateTime = NaiveDateTime;

#[unsafe(no_mangle)]
pub extern "C" fn build_datetime_ns(val: i64) -> *mut c_void {
    let dt = Utc.timestamp_nanos(val).naive_utc();
    Box::into_raw(Box::new(dt)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn build_datetime_from_utc_ns(val: i64) -> *mut c_void {
    let dt = Local.timestamp_nanos(val).naive_local();
    Box::into_raw(Box::new(dt)) as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_datetime(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(ptr as *mut usize) };
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_hour(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.hour()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_year(ptr: *const FfiDateTime) -> i32 {
    let dt = unsafe { &*ptr };
    dt.year()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_month(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.month()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_day(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.day()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_minute(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.minute()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_second(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.second()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_datetime_nanosecond(ptr: *const FfiDateTime) -> u32 {
    let dt = unsafe { &*ptr };
    dt.nanosecond()
}

#[unsafe(no_mangle)]
pub extern "C" fn local_timestamp_nanos(ptr: *const FfiDateTime) -> i64 {
    let dt = unsafe { &*ptr };
    dt.and_local_timezone(Local)
        .unwrap()
        // .naive_local()
        // .and_utc()
        .timestamp_nanos_opt()
        .unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn timestamp_nanos(ptr: *const FfiDateTime) -> i64 {
    let dt = unsafe { &*ptr };
    dt.and_utc().timestamp_nanos_opt().unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn utc_timestamp_to_local(val: i64) -> i64 {
    Local.timestamp_nanos(val).timestamp_nanos_opt().unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn datetime_with_time(
    ptr: *const FfiDateTime,
    hour: u32,
    minute: u32,
    secs: u32,
    nsecs: u32,
) -> *mut c_void {
    let dt = unsafe { &*ptr };
    let time = NaiveTime::from_hms_nano_opt(hour, minute, secs, nsecs).unwrap();
    let dt = dt.and_utc().with_time(time).unwrap().naive_utc();
    Box::into_raw(Box::new(dt)) as *mut c_void
}
