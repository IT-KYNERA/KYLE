use std::process;

fn ky_failed(msg: &str) -> ! {
    eprintln!("KL ASSERT FAILED: {}", msg);
    process::abort();
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert(condition: i32) {
    if condition == 0 {
        ky_failed("assertion failed: condition was false");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert_eq(a: i64, b: i64) {
    if a != b {
        ky_failed(&format!("assertion failed: {} != {}", a, b));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert_ne(a: i64, b: i64) {
    if a == b {
        ky_failed(&format!("assertion failed: {} == {}", a, b));
    }
}
