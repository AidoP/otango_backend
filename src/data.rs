use crate::prelude::*;

use std::fmt;

use chrono::{ DateTime, Utc };
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OptionalExtension;
use serde::{ Serialize, Deserialize };

type Pool = r2d2::Pool<SqliteConnectionManager>;
type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

mod serde_base64 {
    use serde::{Serializer, Deserializer, Deserialize};
    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&base64::encode(data))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let raw = <&str>::deserialize(deserializer)?;
        base64::decode(raw).map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(raw), &"base64 encoded data"))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Signed<T> {
    data: T,
    #[serde(with = "serde_base64")]
    signature: Vec<u8>
}
impl Signed<String> {
    /// Verify the signature, assuming that the string refers to a user
    pub fn verify_user(self, c: &Connection) -> Result<String> {
        let pubkey = User::pubkey(c, &self.data)?
            .ok_or(Error::NoUser(self.data.clone()))?;
        let pubkey = openssl::pkey::PKey::public_key_from_pem(pubkey.as_bytes())?;
        let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &pubkey)?;
        let unsigned = serde_json::to_vec(&self.data)?;
        if verifier.verify_oneshot(&self.signature, &unsigned)? {
            Ok(self.data)
        } else {
            Err(Error::Signature)
        }
    }
}
impl Signed<Certificate> {
    pub fn verify(self) -> Result<Certificate> {
        let pubkey = openssl::pkey::PKey::public_key_from_pem(self.data.pubkey.as_bytes())?;
        let unsigned = serde_json::to_vec(&self.data)?;
        let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &pubkey)?;
        if verifier.verify_oneshot(&self.signature, &unsigned)? {
            Ok(self.data)
        } else {
            Err(Error::Signature)
        }
    }
}
impl<T: serde::Serialize> Signed<By<T>> {
    pub fn verify(self, c: &Connection) -> Result<T> {
        let pubkey = User::pubkey(c, &self.data.user)?
            .ok_or(Error::NoUser(self.data.user.clone()))?;
        let pubkey = openssl::pkey::PKey::public_key_from_pem(pubkey.as_bytes())?;
        let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &pubkey)?;
        let unsigned = serde_json::to_vec(&self.data)?;
        if verifier.verify_oneshot(&self.signature, &unsigned)? && Challenge::remove(c, &self.data.challenge, &self.data.user)? {
            Ok(self.data.data)
        } else {
            Err(Error::Signature)
        }
    }
}

/// An identity for pubkey lookup
#[derive(Serialize, Clone, Deserialize)]
pub struct By<T> {
    user: String,
    challenge: String,
    data: T,
}

#[derive(Serialize, Clone, Deserialize)]
pub enum Privilege {
    None,
    Admin
}
impl Into<u8> for &Privilege {
    fn into(self) -> u8 {
        match self {
            &Privilege::None => 0,
            &Privilege::Admin => 1
        }
    }
}
impl Privilege {
    pub fn from(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Admin),
            _ => None
        }
    }
}
impl rusqlite::ToSql for Privilege {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::{ToSqlOutput, Value::Integer};
        let v: u8 = self.into();
        Ok(ToSqlOutput::Owned(Integer(v as i64)))
    }
}
impl rusqlite::types::FromSql for Privilege {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_i64()?;
        Self::from(value as u8).ok_or(rusqlite::types::FromSqlError::OutOfRange(value))
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Certificate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    pub pubkey: String,
    pub created: DateTime<Utc>
}
impl Certificate {
    pub fn new(name: String, contact: Option<String>, pubkey: String) -> Self {
        Self {
            name,
            contact,
            pubkey,
            created: Utc::now()
        }
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Challenge {
    pub user: String,
    pub challenge: String,
    pub expires: DateTime<Utc>
}
impl Challenge {
    /// Create and register a new base64 encoded challenge for the given user
    pub fn generate(c: &Connection, user: &str) -> Result<String> {
        let mut challenge = [0; 32];
        openssl::rand::rand_bytes(&mut challenge)?;
        let challenge = base64::encode(&challenge);
        let mut s = c.prepare("INSERT INTO `challenge` (`user`, `challenge`, `expires`) SELECT `user`.`rowid`, ?1 AS `challenge`, ?2 AS `expires` FROM `user` WHERE `user`.`name` = ?3")?;
        let expires = Utc::now() + chrono::Duration::minutes(5);
        s.execute(rusqlite::params!(&challenge, expires, user))?;
        Ok(challenge)
    }
    fn prune_old(c: &Connection) -> Result<()> {
        let mut s = c.prepare("DELETE FROM `challenge` WHERE `challenge`.`expires` < ?1")?;
        s.execute([Utc::now()])?;
        Ok(())
    }
    pub fn remove(c: &Connection, challenge: &str, user: &str) -> Result<bool> {
        // Ensure we are not including old challenges
        Self::prune_old(c)?;
        let mut s = c.prepare("DELETE FROM `challenge` INNER JOIN `user` ON `challenge`.`user`=`user`.`rowid` WHERE `challenge`.`challenge` = ?1 AND `user`.`name` = ?2")?;
        Ok(s.execute(rusqlite::params!(challenge, user))? == 1)
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct User {
    pub name: String,
    pub contact: Option<String>,
    pub image: Option<Vec<u8>>,
    pub privilege: Privilege,
    pub pubkey: String
}
impl User {
    pub fn pubkey(c: &Connection, name: &str) -> Result<Option<String>> {
        let mut s = c.prepare("SELECT `pubkey` FROM `user` WHERE `name` = ?1")?;
        Ok(s.query_row([name], |row| row.get(0)).optional()?)
    }
    pub fn named(c: &Connection, name: &str) -> Result<Option<Self>> {
        let mut s = c.prepare("SELECT `name`, `contact`, `image`, `privilege`, `pubkey` FROM `user` WHERE `name` = ?1")?;
        Ok(s.query_row([name], |row| Ok(Self {
            name: row.get(0)?,
            contact: row.get(1)?,
            image: row.get(2)?,
            privilege: row.get(3)?,
            pubkey: row.get(4)?,
        })).optional()?)
    }
    pub fn insert(&self, c: &Connection) -> Result<()> {
        let mut s = c.prepare("INSERT OR IGNORE INTO `user` (`name`, `contact`, `image`, `privilege`, `pubkey`) VALUES (?1, ?2, ?3, ?4, ?5)")?;
        if s.execute(rusqlite::params!(
            &self.name,
            &self.contact,
            &self.image,
            &self.privilege,
            &self.pubkey
        ))? != 1 {
            Err(Error::Exists(self.name.clone()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag: String,
}
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub readings: Vec<WordReading>,
    pub tags: Vec<Tag>
}
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
#[derive(Debug, Serialize, Deserialize)]
pub struct WordReading {
    pub full: String,
    pub accent: String,
    pub definitions: Vec<Definition>
}
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
}
impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.definition)
    }
}
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