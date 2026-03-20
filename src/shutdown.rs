//! Global state for signal handling and graceful shutdown.

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

fn set_interrupted() {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

pub fn register_handlers() {
    if let Err(e) = ctrlc::set_handler(move || {
        set_interrupted();
    }) {
        eprintln!("Warning: Could not register signal handler: {}", e);
    }
}
