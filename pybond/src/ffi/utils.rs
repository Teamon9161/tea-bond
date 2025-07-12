use std::path::PathBuf;
use std::sync::Mutex;

pub fn get_str<'a>(ptr: *mut u8, len: usize) -> &'a str {
    let code_slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    unsafe { std::str::from_utf8_unchecked(code_slice) }
}
// Global variable to store bond data path
static BOND_DATA_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 设置债券数据路径
#[no_mangle]
pub extern "C" fn set_bond_data_path(path_ptr: *mut u8, path_len: usize) {
    let path_str = get_str(path_ptr, path_len);
    let path = PathBuf::from(path_str);
    if let Ok(mut global_path) = BOND_DATA_PATH.lock() {
        *global_path = Some(path);
    }
}

/// 获取当前债券数据路径，如果未设置则返回默认路径
#[inline]
pub fn get_bond_data_path() -> Option<PathBuf> {
    if let Ok(global_path) = BOND_DATA_PATH.lock() {
        global_path.clone()
    } else {
        None
    }
}
