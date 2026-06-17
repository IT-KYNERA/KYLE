// klc_runtime::gc — Garbage collector wrapper (Boehm GC)
//
// Reference: docs/09-memory-model.md
// The Boehm-Demers-Weiser conservative garbage collector is used
// to manage heap memory for KL programs.

/// Wrapper around the Boehm GC for KL runtime memory management.
pub struct Gc;

impl Gc {
    /// Initialize the garbage collector.
    /// Must be called once at program startup before any allocations.
    pub fn init() {
        // In the real implementation:
        // unsafe { boehm::GC_init(); }
    }

    /// Allocate a value of type T on the GC-managed heap.
    pub fn alloc<T>(value: T) -> *mut T {
        // In the real implementation:
        // let ptr = boehm::GC_malloc(size_of::<T>()) as *mut T;
        // unsafe { ptr.write(value); }
        // ptr
        Box::into_raw(Box::new(value))
    }

    /// Allocate zero-initialized memory for an array of T.
    pub fn alloc_array<T>(count: usize) -> *mut T {
        // In the real implementation:
        // boehm::GC_malloc(count * size_of::<T>()) as *mut T
        let layout = std::alloc::Layout::array::<T>(count)
            .expect("invalid layout");
        unsafe { std::alloc::alloc_zeroed(layout) as *mut T }
    }

    /// Force a garbage collection cycle.
    pub fn collect() {
        // In the real implementation:
        // boehm::GC_gcollect();
    }

    /// Enable or disable the GC.
    pub fn enable(_enabled: bool) {
        // In the real implementation:
        // if enabled { boehm::GC_enable(); }
        // else { boehm::GC_disable(); }
    }
}
