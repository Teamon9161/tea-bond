use chrono::NaiveDate;
// use std::ffi::{c_char, c_void, CStr};
use std::ffi::c_void;
use std::slice;
use tea_bond::{Bond, CachedBond};

fn create_date(year: u32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year as i32, month, day).unwrap()
}

fn get_str<'a>(ptr: *mut u8, len: usize) -> &'a str {
    let code_slice = unsafe { slice::from_raw_parts(ptr, len) };
    unsafe { std::str::from_utf8_unchecked(code_slice) }
}

#[no_mangle]
pub extern "C" fn create_bond(code_ptr: *mut u8, code_len: usize) -> *mut c_void {
    let code = get_str(code_ptr, code_len);
    let bond = CachedBond::new(code, None).unwrap();
    bond.into_raw() as *mut c_void
}

#[no_mangle]
pub extern "C" fn bond_duration(
    bond: *const Bond,
    ytm: f64,
    year: u32,
    month: u32,
    day: u32,
) -> f64 {
    let date = create_date(year, month, day);
    let bond = unsafe { &*bond };
    bond.calc_duration(ytm, date, None, None).unwrap()
}
