use std::ffi::CString;
use std::os::unix::io::RawFd;
use libc::{c_int, size_t};

pub fn open(path: &str) -> Result<i32, String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        Err(format!("cannot open '{}'", path))
    } else {
        Ok(fd)
    }
}

pub fn create(path: &str) -> Result<i32, String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let mode = (libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) as libc::c_uint;
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, mode) };
    if fd < 0 {
        Err(format!("cannot create '{}'", path))
    } else {
        Ok(fd)
    }
}

pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize, String> {
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as size_t) };
    if n < 0 {
        Err("read error".to_string())
    } else {
        Ok(n as usize)
    }
}

pub fn write(fd: i32, data: &[u8]) -> Result<usize, String> {
    let n = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len() as size_t) };
    if n < 0 {
        Err("write error".to_string())
    } else {
        Ok(n as usize)
    }
}

pub fn close(fd: i32) {
    unsafe { libc::close(fd); }
}

pub fn exists(path: &str) -> bool {
    CString::new(path)
        .map(|p| unsafe { libc::access(p.as_ptr(), 0) == 0 })
        .unwrap_or(false)
}

pub fn read_to_string(path: &str) -> Result<String, String> {
    let fd = open(path)?;
    let mut buf = vec![0u8; 4096];
    let mut result = Vec::new();
    loop {
        let n = read(fd, &mut buf)?;
        if n == 0 { break; }
        result.extend_from_slice(&buf[..n]);
    }
    close(fd);
    String::from_utf8(result).map_err(|e| format!("invalid utf-8: {}", e))
}

pub fn write_string(path: &str, data: &str) -> Result<(), String> {
    let fd = create(path)?;
    write(fd, data.as_bytes())?;
    close(fd);
    Ok(())
}
