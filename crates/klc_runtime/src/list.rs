use crate::memory::{kl_alloc, kl_free};

#[repr(C)]
pub struct KlList {
    data: *mut i64,
    len: i64,
    cap: i64,
}

const INITIAL_CAP: i64 = 4;

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_new() -> *mut KlList {
    let list = kl_alloc(std::mem::size_of::<KlList>() as i64) as *mut KlList;
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let data = kl_alloc(INITIAL_CAP * std::mem::size_of::<i64>() as i64) as *mut i64;
    if data.is_null() {
        kl_free(list as *mut u8);
        return std::ptr::null_mut();
    }
    unsafe {
        (*list).data = data;
        (*list).len = 0;
        (*list).cap = INITIAL_CAP;
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_free(list: *mut KlList) {
    if list.is_null() {
        return;
    }
    unsafe {
        if !(*list).data.is_null() {
            kl_free((*list).data as *mut u8);
        }
        kl_free(list as *mut u8);
    }
}

fn grow(list: *mut KlList) {
    unsafe {
        let new_cap = (*list).cap * 2;
        let new_data = kl_alloc(new_cap * std::mem::size_of::<i64>() as i64) as *mut i64;
        if new_data.is_null() {
            return;
        }
        for i in 0..(*list).len {
            std::ptr::write(new_data.add(i as usize), std::ptr::read((*list).data.add(i as usize)));
        }
        kl_free((*list).data as *mut u8);
        (*list).data = new_data;
        (*list).cap = new_cap;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_push(list: *mut KlList, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        if (*list).len >= (*list).cap {
            grow(list);
        }
        std::ptr::write((*list).data.add((*list).len as usize), val);
        (*list).len += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_get(list: *const KlList, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        if index < 0 || index >= (*list).len {
            return 0;
        }
        std::ptr::read((*list).data.add(index as usize))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_set(list: *mut KlList, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        if index < 0 || index >= (*list).len {
            return;
        }
        std::ptr::write((*list).data.add(index as usize), val);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_pop(list: *mut KlList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        if (*list).len <= 0 {
            return 0;
        }
        (*list).len -= 1;
        std::ptr::read((*list).data.add((*list).len as usize))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_len(list: *const KlList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe { (*list).len }
}

/// Convert C's argc/argv to a Kyle list<str>.
/// Skips argv[0] (program name).
#[unsafe(no_mangle)]
pub extern "C" fn kl_init_args(argc: i32, argv: *mut *mut u8) -> *mut KlList {
    let list = kl_list_new();
    if list.is_null() {
        return std::ptr::null_mut();
    }
    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };
        if cstr.is_null() {
            continue;
        }
        let len = unsafe {
            let mut n: i32 = 0;
            while *cstr.add(n as usize) != 0 { n += 1; }
            n
        };
        // Allocate Kyle string (ptr + len layout: ptr to bytes, but we store null-terminated)
        let kstr = kl_alloc(len as i64 + 1) as *mut u8;
        if kstr.is_null() { continue; }
        for j in 0..len {
            unsafe { *kstr.add(j as usize) = *cstr.add(j as usize); }
        }
        unsafe { *kstr.add(len as usize) = 0; }
        kl_list_push(list, kstr as i64);
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_slice(list: *mut KlList, start: i64, end: i64) -> *mut KlList {
    if list.is_null() { return std::ptr::null_mut(); }
    let result = kl_list_new();
    if result.is_null() { return std::ptr::null_mut(); }
    unsafe {
        let len = (*list).len;
        let s = if start < 0 { 0 } else if start > len { len } else { start };
        let e = if end < 0 { len } else if end > len { len } else { end };
        let s = s.min(e);
        for i in s..e {
            let val = std::ptr::read((*list).data.add(i as usize));
            kl_list_push(result, val);
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_list_extend(dest: *mut KlList, src: *mut KlList) {
    if dest.is_null() || src.is_null() { return; }
    unsafe {
        let src_len = (*src).len;
        for i in 0..src_len {
            let val = std::ptr::read((*src).data.add(i as usize));
            kl_list_push(dest, val);
        }
    }
}
