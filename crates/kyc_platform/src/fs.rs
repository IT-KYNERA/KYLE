use std::ffi::CString;
use libc::size_t;

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

/// Copy a file from src to dst. Returns Ok(()) on success.
pub fn copy(src: &str, dst: &str) -> Result<(), String> {
    let data = read_to_string(src)?;
    write_string(dst, &data)
}

/// Remove a file. Returns Ok(()) on success.
pub fn remove(path: &str) -> Result<(), String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let r = unsafe { libc::unlink(cpath.as_ptr()) };
    if r != 0 {
        Err(format!("cannot remove '{}'", path))
    } else {
        Ok(())
    }
}

/// Create a directory. Returns Ok(()) on success.
pub fn create_dir(path: &str) -> Result<(), String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let mode = (libc::S_IRWXU | libc::S_IRGRP | libc::S_IXGRP | libc::S_IROTH | libc::S_IXOTH) as libc::c_uint;
    let r = unsafe { libc::mkdir(cpath.as_ptr(), mode as libc::mode_t) };
    if r != 0 {
        Err(format!("cannot create dir '{}'", path))
    } else {
        Ok(())
    }
}

/// Remove a directory. Returns Ok(()) on success.
pub fn remove_dir(path: &str) -> Result<(), String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let r = unsafe { libc::rmdir(cpath.as_ptr()) };
    if r != 0 {
        Err(format!("cannot remove dir '{}'", path))
    } else {
        Ok(())
    }
}

/// List directory entries. Returns a vector of entry names (without full path).
pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let cpath = CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
    let dir = unsafe { libc::opendir(cpath.as_ptr()) };
    if dir.is_null() {
        return Err(format!("cannot open dir '{}'", path));
    }
    let mut entries = Vec::new();
    loop {
        let entry = unsafe { libc::readdir(dir) };
        if entry.is_null() { break; }
        let name_ptr = unsafe { (*entry).d_name.as_ptr() };
        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) }
            .to_str().unwrap_or("").to_string();
        if name != "." && name != ".." {
            entries.push(name);
        }
    }
    unsafe { libc::closedir(dir); }
    Ok(entries)
}

/// Returns true if path is a directory.
pub fn is_dir(path: &str) -> bool {
    let cpath = match CString::new(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::stat(cpath.as_ptr(), &mut stat) == 0 {
            stat.st_mode & libc::S_IFMT == libc::S_IFDIR
        } else {
            false
        }
    }
}

/// Returns true if path is a regular file.
pub fn is_file(path: &str) -> bool {
    let cpath = match CString::new(path) {
        Ok(p) => p,
        Err(_) => return false,
    };
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::stat(cpath.as_ptr(), &mut stat) == 0 {
            stat.st_mode & libc::S_IFMT == libc::S_IFREG
        } else {
            false
        }
    }
}

/// Get file size in bytes. Returns -1 on error.
pub fn size(path: &str) -> i64 {
    let cpath = match CString::new(path) {
        Ok(p) => p,
        Err(_) => return -1,
    };
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::stat(cpath.as_ptr(), &mut stat) == 0 {
            stat.st_size
        } else {
            -1
        }
    }
}

/// Rename/move a file. Returns Ok(()) on success.
pub fn rename(src: &str, dst: &str) -> Result<(), String> {
    let csrc = CString::new(src).map_err(|e| format!("invalid src: {}", e))?;
    let cdst = CString::new(dst).map_err(|e| format!("invalid dst: {}", e))?;
    let r = unsafe { libc::rename(csrc.as_ptr(), cdst.as_ptr()) };
    if r != 0 {
        Err(format!("cannot rename '{}' to '{}'", src, dst))
    } else {
        Ok(())
    }
}
