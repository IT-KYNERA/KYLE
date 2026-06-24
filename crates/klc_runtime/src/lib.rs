#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod memory;
pub mod io;
pub mod string;
pub mod list;
pub mod dict;
pub mod async_;
pub mod task;
pub mod channel;
pub mod error;
pub mod panic;
pub mod thread;

pub use memory::{kl_alloc, kl_free, kl_retain, kl_release};
pub use io::{kl_print, kl_println, kl_print_int, kl_println_int, kl_input, kl_open, kl_read_str, kl_write_str, kl_close, kl_sleep, kl_now};
pub use string::{kl_i64_to_str, kl_strlen, kl_concat, kl_str_contains, kl_str_to_upper, kl_str_to_lower, kl_str_trim, kl_str_replace,
    kl_char_at, kl_is_digit, kl_is_alpha, kl_is_alnum, kl_is_whitespace, kl_is_upper, kl_is_lower, kl_ord, kl_substr, kl_eq_str};
pub use list::{kl_list_new, kl_list_free, kl_list_push, kl_list_pop, kl_list_get, kl_list_set, kl_list_len, kl_init_args};
pub use async_::Executor;
pub use task::Task;
pub use task::PollState;
pub use channel::Channel;
pub use error::KlError;
pub use thread::{kl_spawn_thread, kl_join_thread};
pub use dict::{kl_dict_new, kl_dict_free, kl_dict_get, kl_dict_set, kl_dict_len, kl_dict_contains, kl_dict_remove};
