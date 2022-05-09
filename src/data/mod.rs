mod prelude {
    pub use crate::prelude::*;
    pub use std::fmt;
    
    pub use chrono::{ DateTime, Utc };
    pub use r2d2_sqlite::SqliteConnectionManager;
    pub use rusqlite::OptionalExtension;
    pub use serde::{ Serialize, Deserialize };

    pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

    pub mod serde_base64 {
        use serde::{Serializer, Deserializer, Deserialize};
        pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&base64::encode(data))
        }
        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
            let raw = <&str>::deserialize(deserializer)?;
            base64::decode(raw).map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(raw), &"base64 encoded data"))
        }
    }
}

pub mod auth;
pub use auth::*;
pub mod dictionary;
pub use dictionary::*;
pub mod tag;
pub use tag::*;