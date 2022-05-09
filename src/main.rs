use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4}
};

use actix_cors::Cors;
use actix_web::{
    App,
    get,
    HttpResponse,
    HttpServer,
    Responder,
};
use once_cell::sync::OnceCell;
use openssl::ssl::{ SslAcceptor, SslMethod, SslFiletype };
use r2d2_sqlite::SqliteConnectionManager;

pub mod data;
pub mod error;
pub mod web;

use prelude::*;
mod prelude {
    pub use crate::{
        data::{self, auth::{By, Signed}},
        error::{Error, Result},
    };
    pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
}


#[derive(Debug, config::Config)]
struct Config {
    address: SocketAddr,
    key: String,
    cert: String,
    database: String,
    root_redirection: Option<String>,
    allowed_origins: Vec<String>
}
impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 443)),
            key: "key.pem".into(),
            cert: "cert.pem".into(),
            database: "お単語.db".into(),
            root_redirection: None,
            allowed_origins: vec![]
        }
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = CONFIG.get_or_init(|| <Config as config::Config>::load("otango"));

    let mut ssl = SslAcceptor::mozilla_modern_v5(SslMethod::tls())?;
    ssl.set_private_key_file(&config.key, SslFiletype::PEM)?;
    ssl.set_certificate_chain_file(&config.cert)?;

    let manager = SqliteConnectionManager::file(&config.database);
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allow_any_method()
            .allowed_header("Content-Type");
        if config.allowed_origins.is_empty() {
            cors = cors.allow_any_origin();
        } else {
            for origin in &config.allowed_origins {
                cors = cors.allowed_origin(&origin);
            }
        }
        App::new()
    //        .app_data(web::Data::new(auth::ChallengeStore::default()))
            .app_data(actix_web::web::Data::new(pool.clone()))
            .wrap(cors)

            .service(index)

            .service(web::auth::register)
            .service(web::auth::challenge)

            .service(web::dictionary::get_word)
            .service(web::dictionary::set_word)
            .service(web::dictionary::get_kanji)
            //.service(all_words)
            //.service(backup)
    }).bind_openssl(config.address, ssl)?
        .run()
        .await
}

#[get("/")]
async fn index() -> impl Responder {
    if let Some(location) = CONFIG.get().and_then(|c| c.root_redirection.as_ref()) {
        HttpResponse::PermanentRedirect()
            .append_header(("Location", location.as_str()))
            .finish()
    } else {
        HttpResponse::NotFound()
            .finish()
    }
}

/*
#[get("/単語")]
async fn all_words(db: web::Data<Pool>) -> Result<impl Responder, Error> {
    let c = web::block(move || db.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;
    let words: Vec<data::Word> = web::block(move || data::Word::get(&c))
        .await?
        .map_err(error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&words)?)
    )
}*/

