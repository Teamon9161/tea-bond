use super::utils::get_str;
use chrono::NaiveDate;
use std::ffi::{c_char, c_void};
use tea_bond::{Bond, CachedBond};

fn create_date(year: u32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year as i32, month, day).unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn create_bond(code_ptr: *mut u8, code_len: usize) -> *mut c_void {
    let code = get_str(code_ptr, code_len);
    let bond = CachedBond::new(code, None).unwrap();
    bond.into_raw() as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn free_bond(bond: *mut c_void) {
    let _bond = unsafe { Box::from_raw(bond as *mut CachedBond) };
}

#[unsafe(no_mangle)]
pub extern "C" fn bond_coupon_rate(bond: *const Bond) -> f64 {
    let bond = unsafe { &*bond };
    bond.cp_rate
}

#[unsafe(no_mangle)]
pub extern "C" fn bond_full_code(bond: *const Bond) -> *mut c_char {
    let bond = unsafe { &*bond };
    std::ffi::CString::new(bond.bond_code()).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn bond_calc_ytm(
    bond: *const Bond,
    dirty_price: f64,
    year: u32,
    month: u32,
    day: u32,
) -> f64 {
    let date = create_date(year, month, day);
    let bond = unsafe { &*bond };
    bond.calc_ytm_with_price(dirty_price, date, None, None)
        .unwrap()
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub extern "C" fn bond_accrued_interest(bond: *const Bond, year: u32, month: u32, day: u32) -> f64 {
    let date = create_date(year, month, day);
    let bond = unsafe { &*bond };
    bond.calc_accrued_interest(date, None).unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn bond_dirty_price(
    bond: *const Bond,
    ytm: f64,
    year: u32,
    month: u32,
    day: u32,
) -> f64 {
    let date = create_date(year, month, day);
    let bond = unsafe { &*bond };
    bond.calc_dirty_price_with_ytm(ytm, date, None, None)
        .unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn bond_clean_price(
    bond: *const Bond,
    ytm: f64,
    year: u32,
    month: u32,
    day: u32,
) -> f64 {
    let date = create_date(year, month, day);
    let bond = unsafe { &*bond };
    bond.calc_clean_price_with_ytm(ytm, date, None, None)
        .unwrap()
}
