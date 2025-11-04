use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

pub trait AppEmitter: Send + Clone + 'static {
    fn emit<S: Serialize + Clone + 'static>(&self, event: &str, payload: S);
}

#[cfg(test)]
mock!{
    pub AppEmitter {}
    impl AppEmitter for AppEmitter {
        fn emit<S: Serialize + Clone + 'static>(&self, event: &str, payload: S);
    }
    impl Clone for AppEmitter {
        fn clone(&self) -> Self;
    }
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
