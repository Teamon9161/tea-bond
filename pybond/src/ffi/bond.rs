use chrono::NaiveDate;
use std::ffi::{c_char, c_void, CStr};
use tea_bond::{Bond, CachedBond};

fn create_date(year: u32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year as i32, month, day).unwrap()
}

#[no_mangle]
pub extern "C" fn create_bond(code: *const c_char) -> *mut c_void {
    let code = unsafe { CStr::from_ptr(code) };
    let bond = CachedBond::new(code.to_str().unwrap(), None).unwrap();
    bond.into_raw() as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn bond_duration(
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
