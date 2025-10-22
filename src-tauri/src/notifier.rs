use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::Notify;

#[derive(Clone)]
pub struct SetupEndedNotifier {
    notify: Arc<Notify>,
    loaded: Arc<AtomicBool>,
}

impl SetupEndedNotifier {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            loaded: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn notify_loaded(&self) {
        if !self.loaded.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }

    pub async fn wait_loaded(&self) {
        if self.loaded.load(Ordering::SeqCst) {
            return;
        }

        loop {
            self.notify.notified().await;
            if self.loaded.load(Ordering::SeqCst) {
                return;
            }
        }
    }
}