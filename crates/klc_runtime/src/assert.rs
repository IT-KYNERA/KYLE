use std::process;

fn kl_failed(msg: &str) -> ! {
    eprintln!("KL ASSERT FAILED: {}", msg);
    process::abort();
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_assert(condition: i32) {
    if condition == 0 {
        kl_failed("assertion failed: condition was false");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_assert_eq(a: i64, b: i64) {
    if a != b {
        kl_failed(&format!("assertion failed: {} != {}", a, b));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_assert_ne(a: i64, b: i64) {
    if a == b {
        kl_failed(&format!("assertion failed: {} == {}", a, b));
    }
}
