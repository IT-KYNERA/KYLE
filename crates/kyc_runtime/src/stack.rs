#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_new() -> *mut std::ffi::c_void {
    let s = Box::new(Vec::<i64>::new());
    Box::into_raw(s) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_free(s: *mut std::ffi::c_void) {
    if s.is_null() { return; }
    unsafe { drop(Box::from_raw(s as *mut Vec<i64>)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_push(s: *mut std::ffi::c_void, val: i64) {
    if s.is_null() { return; }
    let ss = unsafe { &mut *(s as *mut Vec<i64>) };
    ss.push(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_pop(s: *mut std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &mut *(s as *mut Vec<i64>) };
    ss.pop().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_peek(s: *const std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    ss.last().copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_len(s: *const std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    ss.len() as i64
}
