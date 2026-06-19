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
pub extern "C" fn kl_list_len(list: *const KlList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe { (*list).len }
}
