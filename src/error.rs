use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    Blocking(actix_web::error::BlockingError),
    ConnectionPool(r2d2::Error),
    Exists(String),
    Json(serde_json::Error),
    NoUser(String),
    Other(Box<dyn 'static + std::error::Error + Send + Sync>),
    Signature,
    Sql(rusqlite::Error),
    Ssl(openssl::error::ErrorStack),
    Utf8(std::str::Utf8Error),
}
impl actix_web::ResponseError for Error {

}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocking(e) => write!(f, "[Blocking Error] {}", e),
            Self::ConnectionPool(e) => write!(f, "[Connection pool Error] {}", e),
            Self::Exists(item) => write!(f, "[Request Error] {:?} already exists", item),
            Self::Json(e) => write!(f, "[JSON Error] {}", e),
            Self::NoUser(user) => write!(f, "[Authentication Error] No user {:?}", user),
            Self::Other(e) => write!(f, "[Other Error] {}", e),
            Self::Signature => write!(f, "[Authentication Error] Payload signature is invalid"),
            Self::Sql(e) => write!(f, "[SQL Error] {}", e),
            Self::Ssl(e) => write!(f, "[OpenSSL Error] {}", e),
            Self::Utf8(e) => write!(f, "[UTF-8 Encoding Error] {}", e)
        }
    }
}
impl From<openssl::error::ErrorStack> for Error {
    fn from(e: openssl::error::ErrorStack) -> Self {
        Self::Ssl(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}
impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Self::ConnectionPool(e)
    }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8(e.utf8_error())
    }
}
impl From<actix_web::error::BlockingError> for Error {
    fn from(e: actix_web::error::BlockingError) -> Self {
        Self::Blocking(e)
    }
}