use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

type TaskFn = Box<dyn FnOnce() + Send>;

pub struct Executor {
    running: Arc<AtomicBool>,
    worker_count: usize,
    task_sender: mpsc::Sender<TaskFn>,
    workers: Vec<JoinHandle<()>>,
}

impl Executor {
    pub fn new(worker_count: usize) -> Self {
        let (tx, rx) = mpsc::channel::<TaskFn>();
        let rx = Arc::new(std::sync::Mutex::new(rx));
        let running = Arc::new(AtomicBool::new(true));
        let mut workers = Vec::with_capacity(worker_count);

        for _id in 0..worker_count {
            let rx = Arc::clone(&rx);
            let running = Arc::clone(&running);
            let handle = thread::spawn(move || {
                loop {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    let task = {
                        let rx = rx.lock().expect("executor rx lock");
                        rx.recv()
                    };
                    match task {
                        Ok(f) => f(),
                        Err(_) => break,
                    }
                }
            });
            workers.push(handle);
        }

        Self {
            running,
            worker_count,
            task_sender: tx,
            workers,
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.task_sender.send(Box::new(f));
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        drop(self.task_sender.clone());
    }

    pub fn wait_for_completion(&mut self) {
        self.stop();
        while let Some(handle) = self.workers.pop() {
            let _ = handle.join();
        }
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.stop();
    }
}
