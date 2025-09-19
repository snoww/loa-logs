use std::io::Read;

use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use flate2::read::GzDecoder;
use serde::de::DeserializeOwned;

pub struct CompressedJson<T>(pub T);

impl<T> FromSql for CompressedJson<T>
where
    T: DeserializeOwned,
{
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(bytes) => {
                let mut decompress = GzDecoder::new(bytes);
                let mut buffer = Vec::new();
                decompress.read_to_end(&mut buffer).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;

                let parsed: T = serde_json::from_slice(&buffer).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;
                Ok(CompressedJson(parsed))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

pub struct JsonColumn<T>(pub T);

impl<T> FromSql for JsonColumn<T>
where
    T: DeserializeOwned,
{
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let parsed = serde_json::from_slice(text).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;
                Ok(JsonColumn(parsed))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}