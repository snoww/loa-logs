use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AppEmitter: Send + 'static {
    fn emit<S: Serialize + Clone + 'static>(&self, event: &str, payload: S);
}

#[derive(Debug, Clone)]
pub struct TauriAppEmitter(AppHandle);

impl AppEmitter for TauriAppEmitter {
    fn emit<S: Serialize + Clone + 'static>(&self, event: &str, payload: S) {
        self.0.emit(event, payload).unwrap();
    }
}

impl TauriAppEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self(app_handle)
    }
}