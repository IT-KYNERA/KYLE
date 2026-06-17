// klc_runtime::panic — Panic handler

pub fn kl_panic(message: &str) -> ! {
    eprintln!("KL PANIC: {}", message);
    std::process::abort();
}
