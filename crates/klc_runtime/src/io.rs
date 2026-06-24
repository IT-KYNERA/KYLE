#[repr(C)]
struct timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

unsafe extern "C" {
    fn write(fd: i64, buf: *const u8, count: i64) -> i64;
    fn read(fd: i32, buf: *mut u8, count: i64) -> i64;
    fn open(path: *const u8, flags: i32, mode: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn nanosleep(req: *const timespec, rem: *mut timespec) -> i32;
}

fn write_stdout(buf: &[u8]) {
    unsafe {
        write(1, buf.as_ptr(), buf.len() as i64);
    }
}

fn write_int(val: i64) {
    let mut buf = [0u8; 20];
    let mut n = if val < 0 {
        -val
    } else {
        val
    };
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if val < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    write_stdout(&buf[i..]);
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_print(ptr: *const u8, len: i32) {
    if ptr.is_null() || len <= 0 {
        return;
    }
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len as usize);
        write_stdout(slice);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_println(ptr: *const u8, len: i32) {
    kl_print(ptr, len);
    write_stdout(b"\n");
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_print_int(val: i64) {
    write_int(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn kl_println_int(val: i64) {
    write_int(val);
    write_stdout(b"\n");
}

/// Read a line from stdin, return heap-allocated null-terminated string.
/// Caller must free with kl_free.
#[unsafe(no_mangle)]
pub extern "C" fn kl_input() -> *mut u8 {
    kl_input_with_prompt(std::ptr::null(), 0)
}

/// Read a line from stdin with an optional prompt.
/// If prompt is non-null and prompt_len > 0, the prompt is printed first.
/// Returns heap-allocated null-terminated string (caller must kl_free).
#[unsafe(no_mangle)]
pub extern "C" fn kl_input_with_prompt(prompt: *const u8, prompt_len: i32) -> *mut u8 {
    if !prompt.is_null() && prompt_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(prompt, prompt_len as usize) };
        write_stdout(slice);
    }
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            let trimmed = line.trim_end_matches('\n');
            let len = trimmed.len();
            let ptr = crate::kl_alloc((len + 1) as i64);
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(trimmed.as_ptr(), ptr, len);
                    *ptr.add(len) = 0;
                }
            }
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

// open flags
const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_RDWR: i32 = 2;
const O_CREAT: i32 = 64;
const O_TRUNC: i32 = 512;
const O_APPEND: i32 = 1024;

fn mode_from_string(mode: *const u8) -> i32 {
    if mode.is_null() { return O_RDONLY; }
    let c = unsafe { *mode };
    match c {
        b'r' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { O_RDWR } else { O_RDONLY }
        }
        b'w' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { O_RDWR | O_CREAT | O_TRUNC } else { O_WRONLY | O_CREAT | O_TRUNC }
        }
        b'a' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { O_RDWR | O_CREAT | O_APPEND } else { O_WRONLY | O_CREAT | O_APPEND }
        }
        _ => O_RDONLY,
    }
}

/// Open a file. Returns fd (>=0) or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn kl_open(path: *const u8, mode: *const u8) -> i32 {
    if path.is_null() { return -1; }
    let flags = mode_from_string(mode);
    unsafe { open(path, flags, 0o644) }
}

/// Read from a file descriptor into a heap-allocated null-terminated string.
/// Returns pointer to the string. Caller must free with kl_free.
/// On error, returns null pointer.
#[unsafe(no_mangle)]
pub extern "C" fn kl_read_str(fd: i32, count: i32) -> *mut u8 {
    if count <= 0 { return std::ptr::null_mut(); }
    let ptr = crate::kl_alloc(count as i64);
    if ptr.is_null() { return std::ptr::null_mut(); }
    let n = unsafe { read(fd, ptr, count as i64) };
    if n < 0 {
        crate::kl_free(ptr);
        return std::ptr::null_mut();
    }
    unsafe { *ptr.add(n as usize) = 0; }
    ptr
}

/// Write a null-terminated string to a file descriptor.
/// Uses C strlen to determine length. Returns bytes written or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn kl_write_str(fd: i32, buf: *const u8) -> i32 {
    if buf.is_null() { return -1; }
    let mut len: i64 = 0;
    while unsafe { *buf.add(len as usize) } != 0 { len += 1; }
    let result = unsafe { write(fd as i64, buf, len) };
    result as i32
}

/// Close a file descriptor. Returns 0 on success, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn kl_close(fd: i32) -> i32 {
    unsafe { close(fd) }
}

/// Sleep for a given number of milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn kl_sleep(ms: i32) {
    if ms <= 0 { return; }
    let ts = timespec {
        tv_sec: (ms as i64) / 1000,
        tv_nsec: ((ms as i64) % 1000) * 1_000_000,
    };
    unsafe { nanosleep(&ts, std::ptr::null_mut()); }
}

/// Get current time in milliseconds since epoch.
#[unsafe(no_mangle)]
pub extern "C" fn kl_now() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(_) => -1,
    }
}
