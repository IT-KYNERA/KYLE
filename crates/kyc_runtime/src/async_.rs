use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

type TaskFn = Box<dyn FnOnce() + Send>;

static EXECUTOR: OnceLock<Executor> = OnceLock::new();

fn global_executor() -> &'static Executor {
    EXECUTOR.get_or_init(|| {
        let count = std::env::var("KL_WORKERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
            });
        Executor::new(count)
    })
}

pub struct Executor {
    running: Arc<AtomicBool>,
    task_sender: mpsc::Sender<TaskFn>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl Executor {
    fn new(worker_count: usize) -> Self {
        let (tx, rx) = mpsc::channel::<TaskFn>();
        let rx = Arc::new(Mutex::new(rx));
        let running = Arc::new(AtomicBool::new(true));
        let mut workers = Vec::with_capacity(worker_count);

        for _id in 0..worker_count {
            let rx = Arc::clone(&rx);
            let running = Arc::clone(&running);
            let handle = thread::spawn(move || {
                let rx = rx;
                loop {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    match rx.lock().expect("rx lock").recv() {
                        Ok(f) => f(),
                        Err(_) => break,
                    }
                }
            });
            workers.push(handle);
        }
        Self { running, task_sender: tx, workers }
    }

    fn spawn<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let _ = self.task_sender.send(Box::new(f));
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        drop(self.task_sender.clone());
    }
}

/// Spawn an async task on the global thread pool.
/// `func` is a C-callable function pointer.
/// Returns a handle (pointer to oneshot receiver).
#[unsafe(no_mangle)]
pub extern "C" fn ky_spawn_task(
    func: Option<unsafe extern "C" fn(i64) -> i64>,
    arg: i64,
) -> i64 {
    let (tx, rx) = mpsc::channel::<i64>();
    let handle = Box::into_raw(Box::new(rx)) as i64;

    let exec = global_executor();
    exec.spawn(move || {
        let result = func.map(|f| unsafe { f(arg) }).unwrap_or(0);
        let _ = tx.send(result);
    });

    handle
}

/// Await a task: blocks until completion, returns the result.
#[unsafe(no_mangle)]
pub extern "C" fn ky_await_task(handle: i64) -> i64 {
    if handle == 0 { return 0; }
    let rx = unsafe { Box::from_raw(handle as *mut mpsc::Receiver<i64>) };
    rx.recv().unwrap_or(0)
}

/// Cooperative yield hint.
#[unsafe(no_mangle)]
pub extern "C" fn ky_yield() {
    std::thread::yield_now();
}

/// Parallel for loop: executes `func(i)` for each i in [start, end)
/// across all thread pool workers. Blocks until all iterations complete.
/// Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_parallel_for(
    func: Option<unsafe extern "C" fn(i64) -> i64>,
    start: i64,
    end: i64,
) -> i64 {
    let n = end - start;
    if n <= 0 || func.is_none() { return 0; }
    let func = func.unwrap();
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = std::cmp::max(1, n / workers as i64);
    let counter = Arc::new(std::sync::atomic::AtomicI64::new(0));
    let total = ((n + chunk - 1) / chunk) as i64;

    let mut s = start;
    while s < end {
        let batch_start = s;
        let batch_end = std::cmp::min(s + chunk, end);
        let counter = Arc::clone(&counter);
        let exec = global_executor();
        exec.spawn(move || {
            for i in batch_start..batch_end {
                unsafe { func(i); }
            }
            counter.fetch_add(1, std::sync::atomic::Ordering::Release);
        });
        s = batch_end;
    }

    // Spin wait for completion (but yield occasionally)
    while counter.load(std::sync::atomic::Ordering::Acquire) < total {
        std::thread::yield_now();
    }
    0
}
