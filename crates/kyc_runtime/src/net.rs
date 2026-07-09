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

/// Read an i32 value from a memory address.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_read_i32(ptr: *const u8) -> i32 {
    if ptr.is_null() { return 0; }
    unsafe { *(ptr as *const i32) }
}

/// Read a pointer value from a memory address.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_read_ptr(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { *(ptr as *const *mut u8) }
}

/// Write i32 to memory.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_write_i32(ptr: *mut u8, val: i32) {
    if ptr.is_null() { return; }
    unsafe { *(ptr as *mut i32) = val; }
}

/// Close a socket. Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_close(fd: i32) -> i32 {
    if fd < 0 {
        return -1;
    }
    unsafe { libc::close(fd) }
}

/// SHA-1 hash of a byte buffer.
/// `data` must be null-terminated. `out` must be 20 bytes.
/// Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_sha1(data: *const u8) -> *mut u8 {
    if data.is_null() {
        return std::ptr::null_mut();
    }
    let len = crate::ky_strlen(data);
    let data_slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let hash = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, data_slice);
    let hash_bytes = hash.as_ref();
    let out_len = hash_bytes.len();
    let buf = crate::ky_alloc((out_len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(hash_bytes.as_ptr(), buf, out_len);
        *buf.add(out_len) = 0;
    }
    buf
}

/// Base64 encode bytes. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_base64_encode(data: *const u8) -> *mut u8 {
    if data.is_null() {
        return std::ptr::null_mut();
    }
    let data_len = crate::ky_strlen(data);
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len as usize) };
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data_slice);
    let bytes = encoded.as_bytes();
    let len = bytes.len();
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
    buf
}

/// Perform WebSocket server-side upgrade handshake.
/// `key` is the Sec-WebSocket-Key header value (null-terminated).
/// Returns accept value as heap-allocated string, or null on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_accept(key: *const u8) -> *mut u8 {
    if key.is_null() { return std::ptr::null_mut(); }
    let key_str = unsafe { std::ffi::CStr::from_ptr(key .cast()) }
        .to_str().unwrap_or("");
    let concat = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key_str);
    let hash = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, concat.as_bytes());
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(hash.as_ref());
    let bytes = encoded.as_bytes();
    let len = bytes.len();
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
    buf
}

/// Read a WebSocket frame from a socket.
/// Returns a heap-allocated WsFrame struct (8 bytes: opcode + payload_len + payload_ptr),
/// or null on error.
/// Caller must ky_free the returned struct.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_read_frame(fd: i32) -> *mut u8 {
    unsafe {
        // Read first 2 bytes
        let mut header = [0u8; 2];
        let n = libc::read(fd, header.as_mut_ptr() as *mut c_void, 2);
        if n != 2 { return std::ptr::null_mut(); }

        let fin = (header[0] & 0x80) != 0;
        let opcode = header[0] & 0x0F;
        let masked = (header[1] & 0x80) != 0;
        let mut payload_len = (header[1] & 0x7F) as u64;

        // Extended payload length
        if payload_len == 126 {
            let mut ext = [0u8; 2];
            if libc::read(fd, ext.as_mut_ptr() as *mut c_void, 2) != 2 { return std::ptr::null_mut(); }
            payload_len = u16::from_be_bytes(ext) as u64;
        } else if payload_len == 127 {
            let mut ext = [0u8; 8];
            if libc::read(fd, ext.as_mut_ptr() as *mut c_void, 8) != 8 { return std::ptr::null_mut(); }
            payload_len = u64::from_be_bytes(ext);
        }

        // Read mask key if present
        let mask_key: u32;
        if masked {
            let mut mask_bytes = [0u8; 4];
            if libc::read(fd, mask_bytes.as_mut_ptr() as *mut c_void, 4) != 4 { return std::ptr::null_mut(); }
            mask_key = u32::from_be_bytes(mask_bytes);
        } else {
            mask_key = 0;
        }

        // Read payload
        let payload_size = payload_len as usize;
        let payload_buf = crate::ky_alloc((payload_size + 1) as i64);
        if payload_buf.is_null() { return std::ptr::null_mut(); }
        if payload_size > 0 {
            let n = libc::read(fd, payload_buf as *mut c_void, payload_size);
            if n as usize != payload_size {
                crate::ky_free(payload_buf);
                return std::ptr::null_mut();
            }
            // Unmask if needed
            if masked {
                for i in 0..payload_size {
                    let key_byte = ((mask_key >> (24 - (i % 4) * 8)) & 0xFF) as u8;
                    *payload_buf.add(i) ^= key_byte;
                }
            }
        }
        *payload_buf.add(payload_size) = 0; // null-terminate

        // Allocate WsFrame struct: [opcode: i32, fin: i32, payload_len: i32, payload: ptr]
        let frame = crate::ky_alloc(32) as *mut i32;
        if frame.is_null() { crate::ky_free(payload_buf); return std::ptr::null_mut(); }
        *frame = opcode as i32;
        *frame.add(1) = if fin { 1 } else { 0 };
        *frame.add(2) = payload_size as i32;
        *(frame.add(3) as *mut *mut u8) = payload_buf;
        frame as *mut u8
    }
}

/// Send a WebSocket frame.
/// Returns bytes written or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_send_frame(fd: i32, opcode: i32, payload: *const u8, payload_len: i32) -> i32 {
    if fd < 0 { return -1; }
    let plen = payload_len as usize;
    let mut header_buf: Vec<u8> = Vec::new();
    header_buf.push(0x80 | (opcode as u8)); // FIN + opcode
    if plen < 126 {
        header_buf.push(plen as u8);
    } else if plen < 65536 {
        header_buf.push(126);
        header_buf.extend_from_slice(&(plen as u16).to_be_bytes());
    } else {
        header_buf.push(127);
        header_buf.extend_from_slice(&(plen as u64).to_be_bytes());
    }

    // Write header
    let total = unsafe {
        let n = libc::write(fd, header_buf.as_ptr() as *const c_void, header_buf.len());
        if n < 0 { return -1; }
        let mut written = n as i32;
        // Write payload
        if plen > 0 && !payload.is_null() {
            let n2 = libc::write(fd, payload as *const c_void, plen);
            if n2 < 0 { return -1; }
            written += n2 as i32;
        }
        written
    };
    total
}
