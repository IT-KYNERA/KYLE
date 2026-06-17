// klc_runtime — Runtime support for compiled KL programs
//
// Depends on: libc, Boehm GC (linked externally)
//
// Responsibilities:
//   - Garbage collector wrapper (Boehm GC)
//   - Async executor (work-stealing scheduler)
//   - Task system
//   - Channel implementation
//   - Mutex and atomics
//   - Error type support
//   - Panic handler

#![allow(dead_code)]

pub mod gc;
pub mod async_;
pub mod task;
pub mod channel;
pub mod error;
pub mod panic;
