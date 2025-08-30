use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Vibrancy(window_vibrancy::Error),
    Ui(tauri::Error),
    Io(std::io::Error),
    Db(rusqlite::Error),
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Vibrancy(e) => write!(f, "UI error: {}", e),
            AppError::Ui(e) => write!(f, "UI error: {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Db(e) => write!(f, "Database error: {}", e),
            AppError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}