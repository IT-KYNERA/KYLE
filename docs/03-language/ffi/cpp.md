# C++ Interop

> Interoperabilidad con C++.
> **Pendiente de implementación.** Actualmente Kyle solo soporta C FFI directo.

## Estado actual

Kyle puede llamar funciones C (vía `extern "C"`) pero NO puede llamar funciones
C++ directamente. Para usar bibliotecas C++, necesitas un wrapper C.

## Workaround: wrapper C

```cpp
// cpp_lib_wrapper.cpp
extern "C" {
    #include "cpp_lib.h"
    // Wrapper functions that call C++ internally
    void* create_object() { return new CppClass(); }
    void destroy_object(void* obj) { delete static_cast<CppClass*>(obj); }
    int call_method(void* obj, int arg) {
        return static_cast<CppClass*>(obj)->method(arg);
    }
}
```

```ky
@link "cpp_lib_wrapper"
extern fn create_object() ptr
extern fn destroy_object(obj: ptr)
extern fn call_method(obj: ptr, arg: i32) i32
```

## Limitaciones

| Aspecto | Soporte |
|---------|---------|
| C `extern "C"` | ✅ Completo |
| C++ name mangling | ❌ No soportado |
| C++ classes directo | ❌ No soportado |
| C++ exceptions | ❌ No soportado |
| C++ templates | ❌ No soportado |
| Wrapper C manual | ✅ Funciona |

## Ver también

- `abi.md` — ABI y calling convention
- `native-libraries.md` — Linkear bibliotecas nativas
