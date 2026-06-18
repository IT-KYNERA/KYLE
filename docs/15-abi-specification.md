# KL ABI Specification v1.0

## Philosophy

The KL ABI defines how compiled code interacts with:
- The operating system
- C libraries (FFI)
- Other KL modules
- The RAII runtime (memory allocator, destructors)
- The async runtime

---

## Name Mangling

KL uses a simple name mangling scheme for symbol export:

```text
KL_<module_hash>_<name>
```

Module hash is a stable hash of the fully qualified module path.

C ABI exports use no mangling (for FFI compatibility):

```text
<name>
```

Declared via:

```kl
extern fn printf(fmt: str, ...) -> i32
```

---

## Data Layout

Primitive types match C ABI sizes:

```text
i8   = 1 byte,  signed
i16  = 2 bytes, signed
i32  = 4 bytes, signed
i64  = 8 bytes, signed
u8   = 1 byte,  unsigned
u16  = 2 bytes, unsigned
u32  = 4 bytes, unsigned
u64  = 8 bytes, unsigned
f32  = 4 bytes, IEEE 754
f64  = 8 bytes, IEEE 754
bool = 1 byte  (0 or 1)
char = 4 bytes (Unicode code point)
```

Struct layout matches C struct layout (no reordering):

```text
Fields are laid out in declaration order.
Alignment follows the largest field alignment.
Padding is added between fields as required.
```

---

## Calling Convention

KL uses the platform default C calling convention for all external calls:

```text
macOS ARM64:    ARM64 AAPCS
Linux x86_64:   System V AMD64 ABI
Windows x86_64: Microsoft x64 ABI
```

Internal KL-to-KL calls may use a faster convention determined by the compiler.

---

## RAII Runtime ABI

KL uses RAII (Resource Acquisition Is Initialization) with compiler-inferred ownership. No garbage collector.
Memory is managed at compile time: values are either moved (zero-cost memcpy) or reference-counted (automatic retain/release).

The runtime exposes:

```kl
extern fn kl_alloc(size: i64) -> *void    # heap allocation for shared values
extern fn kl_free(ptr: *void)              # deallocation when refcount reaches 0
extern fn kl_retain(ptr: *void)            # increment reference count
extern fn kl_release(ptr: *void)           # decrement reference count, free if 0
```

### Stack vs Heap

```text
Stack allocation: default for values that don't escape their scope.
  - Compiler inserts alloca + memcpy for moves.
  - Destructor called at scope exit (LLVM lifetime.end).

Heap allocation: for values that escape or are shared.
  - Compiler inserts kl_alloc + kl_retain/kl_release.
  - Refcount determined by ownership inference pass.
```

### Destructors

```text
Classes can define a destructor. The compiler inserts destructor calls:
  - At scope exit for stack-allocated values
  - In kl_release when refcount reaches 0
```

---

## Async Runtime ABI

The async runtime is linked as a static library:

```text
Runtime state is initialized in main() before user code.
Each async task is a stackful coroutine with a fixed-size stack.
Task switching is done by the work-stealing scheduler.
```

---

## FFI with C

Any `extern fn` declaration generates a C-compatible symbol.

The developer must ensure:

```text
Types match C sizes (checked by compiler warnings).
Pointers from FFI are *void and must be manually managed.
Memory allocated by C must be freed by C (or explicitly).
```

The `kl.toml` manifest specifies C libraries to link:

```toml
[ffi]
libraries = ["ssl", "crypto", "pthread"]
link_paths = ["/usr/local/lib"]
include_dirs = ["/usr/local/include"]
```

---

## Versioning Policy

KL follows Semantic Versioning (SemVer 2.0):

```text
MAJOR: Breaking changes to the language, ABI, or standard library.
MINOR: New features, no breaking changes.
PATCH: Bug fixes, performance improvements, no API changes.
```

ABI stability is guaranteed within a MAJOR version.

```text
Code compiled with KL 1.x is link-compatible with any other 1.x code.
Code compiled with different MAJOR versions is NOT link-compatible.
```

Pre-release versions:

```text
1.0.0-alpha.1
1.0.0-beta.1
1.0.0-rc.1
```

---

# Version

```text
KL ABI Specification v1.0
```
