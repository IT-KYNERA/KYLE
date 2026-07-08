# Startup

> Secuencia de inicio del runtime de Kyle cuando un programa se ejecuta.

## Secuencia de inicio

Cuando el kernel/shell ejecuta un binario compilado con Kyle:

```
1. Cargador del SO carga el binario en memoria
2. C library initialization (__libc_init)
3. Rust runtime init (stack guard, thread local storage)
4. Kyle runtime init:
   a. Inicializar allocator
   b. Configurar panic handler
   c. Inicializar thread pool (lazy, primer uso)
5. Llamar a main() o kyle_main()
6. Ejecutar código del usuario
7. On exit:
   a. Destruir thread pool
   b. Liberar memoria global
   c. Llamar a exit()
```

## Entry Points

### Con `fn main() i32:`

```llvm
define i32 @main() {         # Entry point real
    call i32 @kyle_main()    # Delega al código Kyle
    ret i32 %0
}
```

### Con parámetro args

```llvm
define i32 @main(i32, ptr) {  # Entry point real
    call i32 @kyle_main()     # args se pasan como lista
    ret i32 %0
}
```

Si el código Kyle no tiene `fn main()`, el compilador genera un main implícito
que ejecuta el código del módulo como script.

## Inicialización lazy

El runtime usa inicialización lazy (via `OnceLock`) para:

- Thread pool (primer `ky_spawn_task`)
- Variables de entorno (primer `ky_getenv`)
- Recursos globales

Esto minimiza el overhead de programas que no usan características específicas.

## Ver también

- `scheduler.md` — Inicialización del thread pool
- `memory.md` — Inicialización del allocator
- `06-compiler/linker.md` — Cómo se enlaza el runtime
