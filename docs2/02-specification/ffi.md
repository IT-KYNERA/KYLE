# FFI — Foreign Function Interface

**Status:** 🔜 Phase 0 — being implemented

## `extern fn`

Declares a function implemented in an external library (C ABI).

```ky
extern fn socket(domain: i32, type_: i32, protocol: i32) i32
extern fn connect(sock: i32, addr: ptr, addrlen: i32) i32
extern fn send(sock: i32, buf: ptr, len: i64, flags: i32) i64
```

The compiler generates a declaration without a body, matching the C calling convention.

## `@link`

Specifies a native library to link against.

```ky
@link "libcurl"         # -lcurl
@link "libssl"          # -lssl
@link "libsqlite3"      # -lsqlite3
```

## `ptr` Type

Raw pointer type for FFI and unsafe operations.

```ky
buf: ptr = my_alloc(1024)
value = buf[0] as i8    # load byte
buf[4] = 0xFF as i8     # store byte
next = buf + 16         # pointer arithmetic
```

## Example: Complete FFI

```ky
@link "libcurl"

extern fn curl_easy_init() ptr
extern fn curl_easy_setopt(handle: ptr, option: i32, value: ptr) i32
extern fn curl_easy_perform(handle: ptr) i32
extern fn curl_easy_cleanup(handle: ptr)

fn http_get(url: str) str:
    curl = curl_easy_init()
    # ... configure and perform request ...
    curl_easy_cleanup(curl)
    response
```

## Safety

`extern fn` calls can only be used inside `unsafe` blocks, unless wrapped in a safe function.
