use super::prelude::*;

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
    /// Get the public key and privilege of the user
    pub fn credentials(c: &Connection, name: &str) -> Result<Option<(String, Privilege)>> {
        c.prepare("SELECT `pubkey`, `privilege` FROM `user` WHERE `name` = ?1")?
            .query_row([name], |row| {
                let pubkey = row.get(0)?;
                let privilege = row.get(1)?;
                Ok((pubkey, privilege))
            })
            .optional()
            .map_err(|e| e.into())
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Privilege {
    None,
    Admin
}
impl std::cmp::PartialOrd for Privilege {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this: u8 = self.into();
        let other: u8 = other.into();
        this.partial_cmp(&other)
    }
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Signed<T> {
    data: T,
    #[serde(with = "serde_base64")]
    signature: Vec<u8>
}
impl Signed<String> {
    /// Verify the signature, assuming that the string refers to a user
    /// This is not suitable for verifying a request.
    /// Instead, this allows a user to get a challenge that can be used to correctly validate a request. 
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
    /// Verify that a certificate is valid.
    /// A valid certificate ensures that the other end owns the private key for a given public key.
    /// It does not, however, guarantee anything about the user that presented the certificate, such as ownership of an account.
    /// Certificates should only be used to establish an account, otherwise they must be signed by the old key pair when rotating keys.
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
    pub fn privileged(self, c: &Connection) -> Result<T> {
        self.verify(c, data::Privilege::Admin)
    }
    /// Verify that a request made by a user is valid. This includes challenege verification to prevent replay attacks.
    pub fn verify(self, c: &Connection, requires_privilege: data::Privilege) -> Result<T> {
        let (pubkey, privilege) = User::credentials(c, &self.data.user)?
            .ok_or(Error::NoUser(self.data.user.clone()))?;
        let pubkey = openssl::pkey::PKey::public_key_from_pem(pubkey.as_bytes())?;
        let mut verifier = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &pubkey)?;
        let unsigned = serde_json::to_vec(&self.data)?;
        if verifier.verify_oneshot(&self.signature, &unsigned)? {
            if Challenge::verify(c, &self.data.challenge, &self.data.user)? {
                if privilege >= requires_privilege {
                    Ok(self.data.data)
                } else {
                    Err(Error::Privilege)
                }
            } else {
                Err(Error::Challenge)
            }
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
pub struct Certificate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    pub pubkey: String,
    pub created: DateTime<Utc>
}

#[derive(Clone, Serialize, Deserialize)]
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
    /// Checks a challenge, removing it if it exists
    /// 
    /// Returns true if the challenge was valid
    pub fn verify(c: &Connection, challenge: &str, user: &str) -> Result<bool> {
        // Ensure we are not including old challenges
        Self::prune_old(c)?;
        let mut s = c.prepare("DELETE FROM `challenge` WHERE `challenge`.`challenge` = ?1 AND `challenge`.`user` = (SELECT `user`.`rowid` FROM `user` WHERE `user`.`name` = ?2)")?;
        Ok(s.execute(rusqlite::params!(challenge, user))? == 1)
    }
}