use std::collections::HashSet;

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_new() -> *mut std::ffi::c_void {
    let set = Box::new(HashSet::<i64>::new());
    Box::into_raw(set) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_free(set: *mut std::ffi::c_void) {
    if set.is_null() { return; }
    unsafe { drop(Box::from_raw(set as *mut HashSet<i64>)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_add(set: *mut std::ffi::c_void, val: i64) {
    if set.is_null() { return; }
    let s = unsafe { &mut *(set as *mut HashSet<i64>) };
    s.insert(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_contains(set: *mut std::ffi::c_void, val: i64) -> i32 {
    if set.is_null() { return 0; }
    let s = unsafe { &*(set as *const HashSet<i64>) };
    if s.contains(&val) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_remove(set: *mut std::ffi::c_void, val: i64) -> i32 {
    if set.is_null() { return 0; }
    let s = unsafe { &mut *(set as *mut HashSet<i64>) };
    if s.remove(&val) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_len(set: *mut std::ffi::c_void) -> i64 {
    if set.is_null() { return 0; }
    let s = unsafe { &*(set as *const HashSet<i64>) };
    s.len() as i64
}
