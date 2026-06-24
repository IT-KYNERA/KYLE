use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_new() -> *mut std::ffi::c_void {
    let map = Box::new(HashMap::<String, i64>::new());
    Box::into_raw(map) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_set(dict: *mut std::ffi::c_void, key: *const c_char, val: i64) {
    if dict.is_null() || key.is_null() {
        return;
    }
    let map = unsafe { &mut *(dict as *mut HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("").to_string();
    map.insert(key_str, val);
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_get(dict: *mut std::ffi::c_void, key: *const c_char) -> i64 {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    map.get(key_str).copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_len(dict: *mut std::ffi::c_void) -> i64 {
    if dict.is_null() {
        return 0;
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    map.len() as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_free(dict: *mut std::ffi::c_void) {
    if dict.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(dict as *mut HashMap<String, i64>)); }
}
