use std::collections::VecDeque;

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_new() -> *mut std::ffi::c_void {
    let q = Box::new(VecDeque::<i64>::new());
    Box::into_raw(q) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_free(q: *mut std::ffi::c_void) {
    if q.is_null() { return; }
    unsafe { drop(Box::from_raw(q as *mut VecDeque<i64>)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_push(q: *mut std::ffi::c_void, val: i64) {
    if q.is_null() { return; }
    let qq = unsafe { &mut *(q as *mut VecDeque<i64>) };
    qq.push_back(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_pop(q: *mut std::ffi::c_void) -> i64 {
    if q.is_null() { return 0; }
    let qq = unsafe { &mut *(q as *mut VecDeque<i64>) };
    qq.pop_front().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_peek(q: *const std::ffi::c_void) -> i64 {
    if q.is_null() { return 0; }
    let qq = unsafe { &*(q as *const VecDeque<i64>) };
    qq.front().copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_queue_len(q: *const std::ffi::c_void) -> i64 {
    if q.is_null() { return 0; }
    let qq = unsafe { &*(q as *const VecDeque<i64>) };
    qq.len() as i64
}
