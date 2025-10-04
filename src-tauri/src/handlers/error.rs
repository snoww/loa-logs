use log::error;
use serde::Serialize;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("UI error: {0}")]
    Vibrancy(#[from] window_vibrancy::Error),

    #[error("UI error: {0}")]
    Ui(#[from] tauri::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command error: {0}")]
    Command(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error = self.source().unwrap();
        error!("{}", error);

        if let Some(inner) = error.source() {
            error!("caused by: {}", inner);
        }

        serializer.serialize_str(&self.to_string())
    }
}
