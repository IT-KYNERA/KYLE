# Async Scheduler

> Scheduler de tareas asíncronas: thread pool para async/await.
> Crate: `kyc_runtime/src/async_.rs` (152 líneas), `kyc_runtime/src/task.rs` (41 líneas).

## Responsabilidad

El scheduler gestiona la ejecución de tareas `async fn` y bloques `async:` mediante
un thread pool global. Cada tarea se ejecuta en un hilo del pool y devuelve un
resultado i64 que se obtiene vía `await`.

## Thread Pool

```rust
type TaskFn = Box<dyn FnOnce() + Send>;

pub struct Executor {
    running: Arc<AtomicBool>,
    task_sender: mpsc::Sender<TaskFn>,
    workers: Vec<thread::JoinHandle<()>>,
}
```

- Pool global inicializado con `available_parallelism()` workers
- Configurable vía variable de entorno `KL_WORKERS`
- Comunicación mediante `std::sync::mpsc` channel

## Funciones

### ky_spawn_task

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_spawn_task(
    func: Option<unsafe extern "C" fn(i64) -> i64>,
    arg: i64,
) -> i64
```

Spawnear una función en el thread pool.

- `func`: puntero a función con C calling convention
- `arg`: argumento i64 único
- Retorna: handle i64 opaco para `ky_await_task` (un `Arc` convertido a i64)

```ky
task = async: heavy_computation()
result = await task
```

### ky_await_task

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_await_task(handle: i64) -> i64
```

Esperar el resultado de una tarea asíncrona.

- `handle`: i64 devuelto por `ky_spawn_task`
- Bloquea el hilo actual hasta que la tarea completa
- Retorna: el valor i64 producido por la tarea

### ky_yield

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ky_yield()
```

Cede el paso al scheduler. Permite que otra tarea se ejecute.

## Task internals

```rust
pub struct Task<T> {
    id: u64,
    future: Arc<Mutex<BoxedFuture<T>>>,
}
```

Donde `BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>`.

## Inicialización lazy

El scheduler se inicializa bajo demanda. La primera llamada a `ky_spawn_task`
crea el thread pool.

```rust
static EXECUTOR: OnceLock<Executor> = OnceLock::new();
```

## Configuración

```bash
export KL_WORKERS=8    # Limitar a 8 workers (default: available_parallelism)
```

## Ver también

- `03-language/concurrency/async-await.md` — Sintaxis async/await
- `thread.md` — Hilos del sistema operativo
- `startup.md` — Inicialización del runtime
