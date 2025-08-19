use super::utils::get_str;
use chrono::{Datelike, NaiveDate};
use std::ffi::c_void;
use tea_bond::{BondYtm, FuturePrice, TfEvaluator};

fn create_date(year: u32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year as i32, month, day).unwrap()
}

/// 创建TfEvaluator实例
#[unsafe(no_mangle)]
pub extern "C" fn create_tf_evaluator(
    future_code_ptr: *mut u8,
    future_code_len: usize,
    future_price: f64,
    bond_code_ptr: *mut u8,
    bond_code_len: usize,
    bond_ytm: f64,
    capital_rate: f64,
    year: u32,
    month: u32,
    day: u32,
) -> *mut c_void {
    let future_code = get_str(future_code_ptr, future_code_len);
    let bond_code = get_str(bond_code_ptr, bond_code_len);
    let date = create_date(year, month, day);

    let future = FuturePrice {
        future: tea_bond::Future::new(future_code).into(),
        price: future_price,
    };

    let bond = match tea_bond::CachedBond::new(bond_code, None) {
        Ok(b) => BondYtm::new(b, bond_ytm),
        Err(e) => {
            eprintln!("Failed to create bond {bond_code}: {e:?}");
            return std::ptr::null_mut();
        }
    };

    let evaluator = TfEvaluator {
        date,
        future,
        bond,
        capital_rate,
        reinvest_rate: None,
        ..Default::default()
    };

    Box::into_raw(Box::new(evaluator)) as *mut c_void
}

/// 创建带再投资利率的TfEvaluator实例
#[unsafe(no_mangle)]
pub extern "C" fn create_tf_evaluator_with_reinvest(
    future_code_ptr: *mut u8,
    future_code_len: usize,
    future_price: f64,
    bond_code_ptr: *mut u8,
    bond_code_len: usize,
    bond_ytm: f64,
    capital_rate: f64,
    reinvest_rate: f64,
    year: u32,
    month: u32,
    day: u32,
) -> *mut c_void {
    let future_code = get_str(future_code_ptr, future_code_len);
    let bond_code = get_str(bond_code_ptr, bond_code_len);
    let date = create_date(year, month, day);

    let future = FuturePrice {
        future: tea_bond::Future::new(future_code).into(),
        price: future_price,
    };

    let bond = match tea_bond::CachedBond::new(bond_code, None) {
        Ok(b) => BondYtm::new(b, bond_ytm),
        Err(e) => {
            eprintln!("Failed to create bond {bond_code}: {e:?}");
            return std::ptr::null_mut();
        }
    };

    let evaluator = TfEvaluator {
        date,
        future,
        bond,
        capital_rate,
        reinvest_rate: Some(reinvest_rate),
        ..Default::default()
    };

    Box::into_raw(Box::new(evaluator)) as *mut c_void
}

/// 释放TfEvaluator实例
#[unsafe(no_mangle)]
pub extern "C" fn free_tf_evaluator(evaluator: *mut c_void) {
    if !evaluator.is_null() {
        let _evaluator = unsafe { Box::from_raw(evaluator as *mut TfEvaluator) };
    }
}

/// 判断债券是否是期货的可交割券
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_is_deliverable(evaluator: *const TfEvaluator) -> i32 {
    let evaluator = unsafe { &*evaluator };
    match evaluator.is_deliverable() {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

/// 计算应计利息
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_accrued_interest(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_accrued_interest() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.accrued_interest.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算交割应计利息
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_deliver_accrued_interest(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_deliver_accrued_interest() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.deliver_accrued_interest.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算转换因子
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_cf(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_cf() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.cf.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算债券全价
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_dirty_price(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_dirty_price() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.dirty_price.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算债券净价
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_clean_price(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_clean_price() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.clean_price.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算期货全价
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_future_dirty_price(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_future_dirty_price() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.future_dirty_price.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算交割成本
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_deliver_cost(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_deliver_cost() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.deliver_cost.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算基差
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_basis_spread(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_basis_spread() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.basis_spread.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算期现价差
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_f_b_spread(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_f_b_spread() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.f_b_spread.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算持有收益
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_carry(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_carry() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.carry.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算净基差
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_net_basis_spread(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_net_basis_spread() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.net_basis_spread.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算修正久期
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_duration(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_duration() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.duration.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算内部收益率(IRR)
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_irr(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_irr() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.irr.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算期货隐含收益率
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_future_ytm(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_future_ytm() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.future_ytm.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算到交割的剩余天数
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_remain_days_to_deliver(evaluator: *mut TfEvaluator) -> i32 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_remain_days_to_deliver() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.remain_days_to_deliver.unwrap_or(-1)
        }
        Err(_) => -1,
    }
}

/// 计算剩余付息次数
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_remain_cp_num(evaluator: *mut TfEvaluator) -> i32 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_remain_cp_num() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.remain_cp_num.unwrap_or(-1)
        }
        Err(_) => -1,
    }
}

/// 计算到交割的期间付息
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_remain_cp_to_deliver(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_remain_cp_to_deliver() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.remain_cp_to_deliver.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算加权平均到交割的期间付息
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_remain_cp_to_deliver_wm(evaluator: *mut TfEvaluator) -> f64 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_remain_cp_to_deliver() {
        Ok(eval) => {
            *evaluator = eval;
            evaluator.remain_cp_to_deliver_wm.unwrap_or(f64::NAN)
        }
        Err(_) => f64::NAN,
    }
}

/// 计算所有指标
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_calc_all(evaluator: *mut TfEvaluator) -> i32 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().calc_all() {
        Ok(eval) => {
            *evaluator = eval;
            1
        }
        Err(_) => 0,
    }
}

/// 获取期货交割日期
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_get_deliver_date(
    evaluator: *mut TfEvaluator,
    year: *mut u32,
    month: *mut u32,
    day: *mut u32,
) -> i32 {
    let evaluator = unsafe { &mut *evaluator };
    match evaluator.clone().with_deliver_date() {
        Ok(eval) => {
            *evaluator = eval;
            if let Some(deliver_date) = evaluator.deliver_date {
                unsafe {
                    *year = deliver_date.year() as u32;
                    *month = deliver_date.month();
                    *day = deliver_date.day();
                }
                1
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

/// 获取债券代码
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_bond_code(evaluator: *const TfEvaluator) -> *mut i8 {
    let evaluator = unsafe { &*evaluator };
    let code = evaluator.bond.bond.code();
    match std::ffi::CString::new(code) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 获取期货代码
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_future_code(evaluator: *const TfEvaluator) -> *mut i8 {
    let evaluator = unsafe { &*evaluator };
    let code = &evaluator.future.future.code;
    match std::ffi::CString::new(code.as_ref() as &str) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 获取债券收益率
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_bond_ytm(evaluator: *const TfEvaluator) -> f64 {
    let evaluator = unsafe { &*evaluator };
    evaluator.bond.ytm()
}

/// 获取期货价格
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_future_price(evaluator: *const TfEvaluator) -> f64 {
    let evaluator = unsafe { &*evaluator };
    evaluator.future.price
}

/// 获取资金成本率
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_capital_rate(evaluator: *const TfEvaluator) -> f64 {
    let evaluator = unsafe { &*evaluator };
    evaluator.capital_rate
}

/// 获取再投资利率
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_reinvest_rate(evaluator: *const TfEvaluator) -> f64 {
    let evaluator = unsafe { &*evaluator };
    evaluator.reinvest_rate.unwrap_or(f64::NAN)
}

/// 获取计算日期
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_get_date(
    evaluator: *const TfEvaluator,
    year: *mut u32,
    month: *mut u32,
    day: *mut u32,
) {
    let evaluator = unsafe { &*evaluator };
    let date = evaluator.date;
    unsafe {
        *year = date.year() as u32;
        *month = date.month();
        *day = date.day();
    }
}

/// 更新evaluator的信息
#[unsafe(no_mangle)]
pub extern "C" fn tf_evaluator_update_info(
    evaluator: *mut TfEvaluator,
    future_code_ptr: *mut u8,
    future_code_len: usize,
    future_price: f64,
    bond_code_ptr: *mut u8,
    bond_code_len: usize,
    bond_ytm: f64,
    capital_rate: f64,
    year: u32,
    month: u32,
    day: u32,
) -> i32 {
    let evaluator = unsafe { &mut *evaluator };
    let future_code = get_str(future_code_ptr, future_code_len);
    let bond_code = get_str(bond_code_ptr, bond_code_len);
    let date = create_date(year, month, day);

    let future = FuturePrice {
        future: tea_bond::Future::new(future_code).into(),
        price: future_price,
    };

    let bond_ytm = if bond_code != evaluator.bond.bond.code()
        && bond_code != evaluator.bond.bond.bond_code()
    {
        match tea_bond::CachedBond::new(bond_code, None) {
            Ok(b) => BondYtm::new(b, bond_ytm),
            Err(e) => {
                eprintln!("Failed to create bond {bond_code}: {e:?}");
                return 0;
            }
        }
    } else {
        evaluator.bond.clone()
    };
    *evaluator =
        (*evaluator)
            .clone()
            .update_with_new_info(date, future, bond_ytm, capital_rate, None);

    1
}

/// 释放C字符串
#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut i8) {
    if !s.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(s);
        }
    }
}
