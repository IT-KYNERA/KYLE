# Drop / Free

> Los valores Move se liberan automáticamente al salir de scope.
> El compilador inserta llamadas a `ky_free` donde corresponde.

## Automático

No necesitas llamar `free()` manualmente. El borrow analysis inserta `ky_free`
al final del scope:

```ky
fn main() i32:
    s: str = "hola"
    println(s)
    # ← aquí el compilador inserta ky_free(s)
    0
```

## Equivalente manual (para FFI)

Cuando trabajas con punteros raw (`ptr`), la gestión es manual:

```ky
extern fn ky_alloc(size: i64) ptr
extern fn ky_free(ptr)

buf: ptr = ky_alloc(1024)
# usar buf...
ky_free(buf)
```

## Scope y free

```ky
fn test() i32:
    if true:
        s: str = "temporal"
        # ← ky_free(s) aquí (sale de scope del if)
    println("ok")
    # sin error: s ya se liberó
    0
```

## Orden de liberación

Las variables se liberan en orden inverso a su creación (LIFO):

```ky
a: str = "primero"
b: str = "segundo"
# ← ky_free(b)
# ← ky_free(a)
```

## Ver también

- `allocator.md` — Cómo funciona el allocador
- `move.md` — Cuándo se mueve ownership
