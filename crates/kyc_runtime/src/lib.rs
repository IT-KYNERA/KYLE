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
pub mod net;
pub mod datetime;
pub mod uuid;
pub mod bytes;
pub mod date;
pub mod decimal;
pub mod channel;
pub mod thread;
pub mod url;
pub mod regex;

pub use memory::{ky_alloc, ky_free, ky_retain, ky_release};
pub use io::{ky_print, ky_println, ky_input, ky_input_with_prompt, ky_open, ky_read_str, ky_write_str, ky_close, ky_sleep, ky_now};
pub use string::{ky_i64_to_str, ky_f32_to_str, ky_str_to_i64, ky_strlen, ky_concat, ky_str_contains, ky_str_to_upper, ky_str_to_lower, ky_str_trim, ky_str_replace,
    ky_char_at, ky_is_digit, ky_is_alpha, ky_is_alnum, ky_is_whitespace, ky_is_upper, ky_is_lower, ky_ord, ky_substr, ky_eq_str, ky_from_cstr, ky_getenv, ky_setenv,
    ky_str_builder_new, ky_str_builder_append, ky_str_builder_to_str, ky_str_builder_free};
pub use list::{ky_list_new, ky_list_free, ky_list_push, ky_list_pop, ky_list_get, ky_list_set, ky_list_len, ky_init_args};
pub use async_::{ky_spawn_task, ky_await_task, ky_yield};
pub use thread::{ky_spawn_thread, ky_join_thread};
pub use channel::{ky_channel_new, ky_channel_send, ky_channel_recv, ky_channel_close, ky_channel_len, ky_channel_free};
pub use dict::{ky_dict_new, ky_dict_free, ky_dict_get, ky_dict_set, ky_dict_len, ky_struct_to_json, ky_json_to_struct};
pub use net::{ky_tcp_listen, ky_tcp_accept, ky_tcp_read, ky_tcp_write, ky_tcp_close, ky_ptr_read_i32, ky_ptr_read_ptr, ky_ptr_write_i32, ky_sha1, ky_base64_encode, ky_ws_accept, ky_ws_read_frame, ky_ws_send_frame};
pub use datetime::{ky_datetime_now, ky_datetime_parse, ky_datetime_format, ky_datetime_year, ky_datetime_month, ky_datetime_day, ky_datetime_hour, ky_datetime_minute, ky_datetime_second, ky_datetime_add_days, ky_datetime_add_hours, ky_datetime_diff, ky_datetime_from_ymdhms};
pub use uuid::{ky_uuid_v4, ky_uuid_parse};
pub use bytes::{ky_bytes_new, ky_bytes_free, ky_bytes_get, ky_bytes_set, ky_bytes_to_hex, ky_bytes_from_hex, ky_bytes_to_base64};
pub use date::{ky_date_today, ky_date_from_ymd, ky_date_parse, ky_date_year, ky_date_month, ky_date_day, ky_date_weekday, ky_date_add_days, ky_date_format, ky_time_from_hms, ky_time_now, ky_time_parse, ky_time_hour, ky_time_minute, ky_time_second};
pub use decimal::{ky_decimal_from_str, ky_decimal_to_str, ky_decimal_round, ky_decimal_truncate};
pub use url::{ky_url_scheme, ky_url_host, ky_url_port, ky_url_path, ky_url_query};
pub use regex::{ky_regex_new, ky_regex_free, ky_regex_is_match, ky_regex_find, ky_regex_replace};

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
