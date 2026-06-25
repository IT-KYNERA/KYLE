use std::collections::HashMap;
use std::ffi::{CStr, CString};
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

/// Create a dict and immediately populate it from a JSON string.
/// Returns a pointer to the dict, or null on parse error.
#[unsafe(no_mangle)]
pub extern "C" fn kl_json_parse(json: *const u8) -> *mut std::ffi::c_void {
    if json.is_null() {
        return std::ptr::null_mut();
    }
    let s = unsafe { std::ffi::CStr::from_ptr(json) }
        .to_str().unwrap_or("").trim().to_string();
    if s.is_empty() {
        return std::ptr::null_mut();
    }
    // Basic JSON object parser (handles {"key":123,"key2":456})
    let mut map = Box::new(HashMap::<String, i64>::new());
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len()-1];
        for part in inner.split(',') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some(eq_pos) = part.find(':') {
                let key_part = part[..eq_pos].trim().trim_matches('"');
                let val_part = part[eq_pos+1..].trim();
                if let Ok(val) = val_part.parse::<i64>() {
                    map.insert(key_part.to_string(), val);
                }
            }
        }
    }
    Box::into_raw(map) as *mut std::ffi::c_void
}

/// Serialize a dict to a JSON string.
/// Returns a heap-allocated C string (must be freed by caller with kl_free).
#[unsafe(no_mangle)]
pub extern "C" fn kl_json_stringify(dict: *mut std::ffi::c_void) -> *mut u8 {
    if dict.is_null() {
        return std::ptr::null_mut();
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let mut result = String::from('{');
    for (i, (key, val)) in map.iter().enumerate() {
        if i > 0 { result.push(','); }
        result.push_str(&format!("\"{}\":{}", key, val));
    }
    result.push('}');
    // Allocate on heap for RAII
    let c_str = CString::new(result).unwrap_or_default();
    let ptr = c_str.into_bytes_with_nul().leak().as_mut_ptr();
    ptr
}
