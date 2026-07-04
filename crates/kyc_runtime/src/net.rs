use std::ffi::c_void;

/// Create a TCP socket, bind to port, and start listening.
/// Returns the socket fd, or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_listen(port: i32) -> i32 {
    unsafe {
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
        if fd < 0 {
            return -1;
        }

        // SO_REUSEADDR to avoid "address already in use"
        let optval: i32 = 1;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &optval as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );

        // Bind to 0.0.0.0:port
        let mut addr: libc::sockaddr_in = std::mem::zeroed();
        addr.sin_family = libc::AF_INET as libc::sa_family_t;
        addr.sin_port = (port as u16).to_be();
        addr.sin_addr.s_addr = 0; // INADDR_ANY = 0

        let bind_result = libc::bind(
            fd,
            &addr as *const libc::sockaddr_in as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_in>() as u32,
        );
        if bind_result < 0 {
            libc::close(fd);
            return -1;
        }

        let listen_result = libc::listen(fd, 128);
        if listen_result < 0 {
            libc::close(fd);
            return -1;
        }

        fd
    }
}

/// Accept a connection. Returns the client fd, or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_accept(fd: i32) -> i32 {
    unsafe {
        let mut addr: libc::sockaddr_in = std::mem::zeroed();
        let mut addrlen: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        libc::accept(
            fd,
            &mut addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
            &mut addrlen,
        )
    }
}

/// Read from a socket into a heap-allocated null-terminated string.
/// Returns pointer to the string, or null on error. Caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_read(fd: i32, count: i32) -> *mut u8 {
    if fd < 0 || count <= 0 {
        return std::ptr::null_mut();
    }
    let buf = crate::ky_alloc(count as i64);
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let n = libc::read(fd, buf as *mut c_void, count as usize);
        if n <= 0 {
            crate::ky_free(buf);
            return std::ptr::null_mut();
        }
        *buf.add(n as usize) = 0;
    }
    buf
}

/// Write bytes to a socket. Returns bytes written, or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_write(fd: i32, buf: *const u8, len: i32) -> i32 {
    if fd < 0 || buf.is_null() || len <= 0 {
        return -1;
    }
    unsafe { libc::write(fd, buf as *const c_void, len as usize) as i32 }
}

/// Close a socket. Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_close(fd: i32) -> i32 {
    if fd < 0 {
        return -1;
    }
    unsafe { libc::close(fd) }
}
