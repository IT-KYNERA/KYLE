use std::collections::HashMap;
<<<<<<< HEAD
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
=======
use crate::memory::{kl_alloc, kl_free};

#[repr(C)]
pub struct KlDict {
    data: *mut HashMap<String, i64>,
}

fn kl_str_to_rust(ptr: *const u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = crate::string::kl_strlen(ptr) as usize;
    if len == 0 {
        return String::new();
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(slice).to_string()
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_new() -> *mut KlDict {
    let map = Box::into_raw(Box::new(HashMap::new()));
    let dict = kl_alloc(std::mem::size_of::<KlDict>() as i64) as *mut KlDict;
    if dict.is_null() {
        unsafe { drop(Box::from_raw(map)); }
        return std::ptr::null_mut();
    }
    unsafe {
        (*dict).data = map;
    }
    dict
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_free(dict: *mut KlDict) {
    if dict.is_null() {
        return;
    }
    unsafe {
        if !(*dict).data.is_null() {
            drop(Box::from_raw((*dict).data));
        }
        kl_free(dict as *mut u8);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_get(dict: *mut KlDict, key: *const u8) -> i64 {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let key_str = kl_str_to_rust(key);
    unsafe {
        let map = &*((*dict).data);
        map.get(&key_str).copied().unwrap_or(0)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_set(dict: *mut KlDict, key: *const u8, value: i64) {
    if dict.is_null() || key.is_null() {
        return;
    }
    let key_str = kl_str_to_rust(key);
    unsafe {
        let map = &mut *((*dict).data);
        map.insert(key_str, value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_len(dict: *mut KlDict) -> i64 {
    if dict.is_null() {
        return 0;
    }
    unsafe {
        let map = &*((*dict).data);
        map.len() as i64
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_contains(dict: *mut KlDict, key: *const u8) -> i32 {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let key_str = kl_str_to_rust(key);
    unsafe {
        let map = &*((*dict).data);
        if map.contains_key(&key_str) { 1 } else { 0 }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_dict_remove(dict: *mut KlDict, key: *const u8) {
    if dict.is_null() || key.is_null() {
        return;
    }
    let key_str = kl_str_to_rust(key);
    unsafe {
        let map = &mut *((*dict).data);
        map.remove(&key_str);
    }
>>>>>>> origin/main
}
