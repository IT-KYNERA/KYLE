# Platform Abstraction

> Capa de abstracción de plataforma: interacción con el sistema operativo.
> Crate: `kyc_platform` (en desarrollo).

## Responsabilidad

La capa Platform abstrae las diferencias entre sistemas operativos, proporcionando
una API unificada para el runtime de Kyle. Es la única capa que conoce el SO subyacente.

## Status actual

| Módulo | Estado | Crate | Notas |
|--------|--------|-------|-------|
| File I/O | ✅ | `kyc_runtime/src/io.rs` | open/read/write/close |
| Time | ✅ | `kyc_runtime/src/datetime.rs` | now/parse/format |
| Networking | ✅ | `kyc_runtime/src/net.rs` | TCP listen/accept/read/write |
| Threads | ✅ | `kyc_runtime/src/thread.rs` | spawn/join |
| Environment | ✅ | `kyc_runtime/src/string.rs` | getenv/setenv |
| Filesystem (dirs) | ❌ | — | Pendiente |
| Process | ❌ | — | Pendiente |
| Signals | ❌ | — | Pendiente |
| Sockets (UDP) | ❌ | — | Pendiente |
| USB/Serial | ❌ | — | Pendiente |

## I/O Module

El módulo I/O actual (`io.rs`) proporciona:

```rust
// File operations
ky_open(path, mode) -> i32 fd       // Open file
ky_read_str(fd, len) -> ptr         // Read as string
ky_write_str(fd, buf) -> i32        // Write string
ky_close(fd) -> i32                 // Close fd
```

## Networking Module

```rust
// TCP operations
ky_tcp_listen(port) -> i32 fd
ky_tcp_accept(fd) -> i32 client_fd
ky_tcp_read(fd, len) -> ptr
ky_tcp_write(fd, buf, len) -> i32
ky_tcp_close(fd) -> i32

// WebSocket
ky_ws_accept(request) -> ptr
ky_ws_read_frame(fd) -> ptr
ky_ws_send_frame(fd, opcode, data, len) -> i32

// Crypto helpers
ky_sha1(data, len, output) -> i32
ky_base64_encode(data, len) -> ptr

// Pointer operations (for FFI)
ky_ptr_read_i32(ptr) -> i32
ky_ptr_read_ptr(ptr) -> ptr
ky_ptr_write_i32(ptr, i32)
```

## Time Module

```rust
ky_now() -> i64                     // Current timestamp (ms)
ky_sleep(ms)                        // Sleep milliseconds
```

## Platform Adapters

Cada SO tiene su propia implementación de la plataforma:

| SO | Status |
|----|--------|
| macOS (Apple Silicon) | ✅ Producción |
| Linux (ARM64) | ✅ CI tested |
| Linux (x86_64) | ✅ CI tested |
| WASM | 🔶 Experimental |
| Windows | 📅 Planeado |
| KYOS | 📅 Aspiracional |

## Ver también

- `06-compiler/linker.md` — Cómo se linkea con bibliotecas de plataforma
- `04-standard-library/io.md` — API de I/O para el usuario
- `04-standard-library/fs.md` — API de filesystem
