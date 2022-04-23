use std::fmt;
use serde::{Serialize, Deserialize};

#[cfg(feature = "db")]
use rusqlite::OptionalExtension;
#[cfg(feature = "db")]
type Pool = r2d2::Pool<SqliteConnectionManager>;
#[cfg(feature = "db")]
type Connection = r2d2::PooledConnection<SqliteConnectionManager>;
#[cfg(feature = "db")]
type SqlError = rusqlite::Error;
#[cfg(feature = "db")]
use r2d2_sqlite::SqliteConnectionManager;

#[cfg(feature = "actix")]
use actix_web::{body, error::ResponseError, HttpResponse, http::StatusCode};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    JsonError(serde_json::Error),
    #[cfg(feature = "db")]
    SqlError(rusqlite::Error)
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError(e) => write!(f, "[JSON Error] {}", e),
            #[cfg(feature = "db")]
            Self::SqlError(e) => write!(f, "[SQL Error] {}", e)
        }
    }
}
#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        HttpResponse::InternalServerError()
            .finish()
    }
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonError(e)
    }
}
#[cfg(feature = "db")]
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::SqlError(e)
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Tag {
    pub tag: String,
}
#[cfg(feature = "db")]
impl Tag {
    const QUERY_WORD: &'static str = "SELECT `tag`.`tag` FROM `word_tag` INNER JOIN `tag` ON `word_tag`.`tag` = `tag`.`rowid` WHERE `word_tag`.`word` = ?1";
    pub fn for_word(c: &Connection, word: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY_WORD)?;
        let rows = s.query_map([word], |row| {
                Ok(Self {
                    tag: row.get(0)?
                })
            })?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag?)
        }
        Ok(tags)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub readings: Vec<WordReading>,
    pub tags: Vec<Tag>
}
#[cfg(feature = "db")]
impl Word {
    const QUERY: &'static str = "SELECT `word`, `rowid` FROM `word` WHERE `word` = ?1";
    pub fn get(c: &Connection, word: &str) -> Result<Option<Self>> {
        Ok(c.prepare(Self::QUERY)?
            .query_row([word], |row| {
                let word_id = row.get(1)?;
                Ok(Self {
                    word: row.get(0)?,
                    readings: WordReading::for_word(c, word_id)?,
                    tags: Tag::for_word(c, word_id)?
                })
            })
            .optional()?
        )
    }
}
#[derive(Serialize, Deserialize)]
pub struct WordReading {
    pub full: String,
    pub accent: String,
    pub definitions: Vec<Definition>
}
#[cfg(feature = "db")]
impl WordReading {
    const QUERY: &'static str = "SELECT `full`, `accent`, `rowid` FROM `word_reading` WHERE `word` = ?1";
    pub fn for_word(c: &Connection, word: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([word], |row| {
                Ok(Self {
                    full: row.get(0)?,
                    accent: row.get(1)?,
                    definitions: Definition::for_reading(c, row.get(2)?)?
                })
            })?;
        let mut readings = Vec::new();
        for reading in rows {
            readings.push(reading?)
        }
        Ok(readings)
    }
}
#[derive(Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
}
impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.definition)
    }
}
#[cfg(feature = "db")]
impl Definition {
    const QUERY: &'static str = "SELECT `definition` FROM `definition` WHERE `definition`.`word_reading` = ?1";
    pub fn for_reading(c: &Connection, reading: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([reading], |row| {
                Ok(Self {
                    definition: row.get(0)?
                })
            })?;
        let mut definitions = Vec::new();
        for definition in rows {
            definitions.push(definition?)
        }
        Ok(definitions)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Kanji {
    pub kanji: char,
    pub memonic: String
}
#[cfg(feature = "db")]
impl Kanji {
    const QUERY: &'static str = "SELECT `kanji`, `memonic` FROM `kanji` WHERE `kanji` = ?1";
    pub fn get(c: &Connection, kanji: char) -> rusqlite::Result<Option<Self>> {
        Ok(c.prepare(Self::QUERY)?.query_row([kanji.to_string()], |row| {
            Ok(Self {
                kanji: row.get::<_, String>(0)?.chars().next().ok_or(rusqlite::Error::QueryReturnedNoRows)?,
                memonic: row.get(1)?
            })
        }).optional()?)
    }
    /*pub fn for_reading(c: &Connection, reading: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([reading], |row| {
                Ok(Self {
                    definition: row.get(0)?
                })
            })?;
        let mut definitions = Vec::new();
        for definition in rows {
            definitions.push(definition?)
        }
        Ok(definitions)
    }*/
}
