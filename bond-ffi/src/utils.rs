pub fn get_str<'a>(ptr: *mut u8, len: usize) -> &'a str {
    let code_slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    unsafe { std::str::from_utf8_unchecked(code_slice) }
}
