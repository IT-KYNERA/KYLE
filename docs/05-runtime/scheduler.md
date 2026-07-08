# Async Scheduler

> Scheduler de tareas asíncronas: thread pool para async/await.
> Crate: `kyc_runtime/src/async_.rs` (152 líneas), `kyc_runtime/src/task.rs` (41 líneas).

## Responsabilidad

El scheduler gestiona la ejecución de tareas `async fn` y bloques `async:` mediante
un thread pool global. Cada tarea se ejecuta en un hilo del pool y devuelve un
resultado que se obtiene vía `await`.

## Thread Pool

```rust
pub struct Executor {
    workers: Vec<thread::JoinHandle<()>>,
    sender: crossbeam_channel::Sender<BoxedJob>,
    receiver: crossbeam_channel::Receiver<BoxedJob>,
}
```

- Pool global inicializado con `available_parallelism()` workers
- Configurable vía variable de entorno `KL_WORKERS`
- Comunicación mediante channels `crossbeam_channel`

## Funciones

### ky_spawn_task

```rust
pub unsafe extern "C" fn ky_spawn_task(fn_ptr: usize, arg: i64) -> *mut Arc<Mutex<Option<i64>>>
```

Spawnear una función asíncrona en el thread pool.

- `fn_ptr`: puntero a la función a ejecutar
- `arg`: argumento i64
- Retorna: handle opaque para `ky_await_task`

```ky
task = async: heavy_computation()
result = await task
```

### ky_await_task

```rust
pub unsafe extern "C" fn ky_await_task(handle: *mut Arc<Mutex<Option<i64>>>) -> i64
```

Esperar el resultado de una tarea asíncrona.

- Bloquea el hilo actual hasta que la tarea completa
- Puede llamarse múltiples veces (el resultado se cachea)
- Retorna: el valor i64 producido por la tarea

### ky_yield

```rust
pub unsafe extern "C" fn ky_yield()
```

Cede el paso al scheduler. Permite que otra tarea se ejecute.

## Task internals

```rust
struct Task {
    future: BoxedFuture<'static, i64>,
    poll_state: PollState,
    result: Option<i64>,
}

enum PollState {
    Pending,
    Ready(i64),
    Panicked(String),
}
```

## Inicialización

El scheduler se inicializa bajo demanda (lazy). La primera llamada a `ky_spawn_task`
crea el thread pool.

```rust
fn ensure_executor() -> &'static Executor {
    static EXECUTOR: OnceLock<Executor> = OnceLock::new();
    EXECUTOR.get_or_init(|| Executor::new(worker_count()))
}
```

## Configuración

```bash
export KL_WORKERS=8    # Limitar a 8 workers (default: available_parallelism)
```

## Ver también

- `03-language/concurrency/async-await.md` — Sintaxis async/await
- `thread.md` — Hilos del sistema operativo
- `startup.md` — Inicialización del runtime
