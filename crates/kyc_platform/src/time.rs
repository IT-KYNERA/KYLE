use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Current time in milliseconds since epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Current time in microseconds since epoch.
pub fn now_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as i64
}

pub fn sleep_ms(ms: i32) {
    let sec = (ms as i64) / 1000;
    let nsec = ((ms as i64) % 1000) * 1_000_000;
    let ts = libc::timespec { tv_sec: sec as libc::time_t, tv_nsec: nsec as libc::c_long };
    unsafe { libc::nanosleep(&ts, std::ptr::null_mut()); }
}
