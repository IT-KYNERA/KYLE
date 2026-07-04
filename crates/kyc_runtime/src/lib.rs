#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod memory;
pub mod io;
pub mod string;
pub mod list;
pub mod dict;
pub mod async_;
pub mod assert;
pub mod error;
pub mod panic;

pub use memory::{ky_alloc, ky_free, ky_retain, ky_release};
pub use io::{ky_print, ky_println, ky_input, ky_input_with_prompt, ky_open, ky_read_str, ky_write_str, ky_close, ky_sleep, ky_now};
pub use string::{ky_i64_to_str, ky_str_to_i64, ky_strlen, ky_concat, ky_str_contains, ky_str_to_upper, ky_str_to_lower, ky_str_trim, ky_str_replace,
    ky_char_at, ky_is_digit, ky_is_alpha, ky_is_alnum, ky_is_whitespace, ky_is_upper, ky_is_lower, ky_ord, ky_substr, ky_eq_str, ky_from_cstr, ky_getenv, ky_setenv};
pub use list::{ky_list_new, ky_list_free, ky_list_push, ky_list_pop, ky_list_get, ky_list_set, ky_list_len, ky_init_args};
pub use async_::{ky_spawn_task, ky_await_task, ky_yield};
pub use dict::{ky_dict_new, ky_dict_free, ky_dict_get, ky_dict_set, ky_dict_len};

/// Power: compute base ** exp for i64 values. Returns i64 (truncated).
#[unsafe(no_mangle)]
pub extern "C" fn ky_pow(base: i64, exp: i64) -> i64 {
    if exp == 0 { return 1; }
    if exp < 0 { return 0; } // floor for negative exponents
    let mut result: i64 = 1;
    for _ in 0..exp {
        result = result.wrapping_mul(base);
    }
    result
}

/// `x +% p` = x + (x * p / 100)
#[unsafe(no_mangle)]
pub extern "C" fn ky_add_pct(x: i64, p: i64) -> i64 {
    x + (x * p / 100)
}

/// `x -% p` = x - (x * p / 100)  
#[unsafe(no_mangle)]
pub extern "C" fn ky_sub_pct(x: i64, p: i64) -> i64 {
    x - (x * p / 100)
}

/// `x *% p` = x * p / 100 (percentage of)
#[unsafe(no_mangle)]
pub extern "C" fn ky_mul_pct(x: i64, p: i64) -> i64 {
    x * p / 100
}
