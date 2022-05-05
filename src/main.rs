use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4}
};

use actix_cors::Cors;
use actix_web::{
    App,
    get,
    http::header::ContentType,
    HttpResponse,
    HttpServer,
    post,
    Responder,
    web
};
use once_cell::sync::OnceCell;
use openssl::ssl::{ SslAcceptor, SslMethod, SslFiletype };
use r2d2_sqlite::SqliteConnectionManager;

mod data;
mod error;

use prelude::*;
mod prelude {
    pub use crate::error::{Error, Result};
}

type Pool = r2d2::Pool<SqliteConnectionManager>;

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
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)

            .service(register)
            .service(challenge)
            .service(verify)

            .service(index)
            .service(word)
            .service(kanji)
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

#[get("/word/{word}")]
async fn word(db: web::Data<Pool>, path: web::Path<String>) -> Result<impl Responder> {
    let word = path.into_inner();
    let c = web::block(move || db.get())
        .await??;
    let word = web::block(move || data::Word::get(&c, &word))
        .await??;

    if let Some(word) = word {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&word)?)
        )
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[get("/kanji/{kanji}")]
async fn kanji(db: web::Data<Pool>, path: web::Path<char>) -> Result<impl Responder> {
    let kanji = path.into_inner();
    let c = web::block(move || db.get())
        .await??;
    let kanji = web::block(move || data::Kanji::get(&c, kanji))
        .await??;

    if let Some(kanji) = kanji {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&kanji)?)
        )
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[post("/auth/register")]
async fn register(db: web::Data<Pool>, signed: web::Json<data::Signed<data::Certificate>>) -> Result<impl Responder> {
    let cert = signed.into_inner().verify()?;

    let user = data::User {
        name: cert.name,
        contact: cert.contact,
        image: None,
        privilege: data::Privilege::None,
        pubkey: cert.pubkey
    };

    let c = web::block(move || db.get())
        .await??;
    web::block(move || user.insert(&c))
        .await??;

    Ok(HttpResponse::Created())
}
#[post("/auth/challenge")]
async fn challenge(db: web::Data<Pool>, signed: web::Json<data::Signed<String>>) -> Result<impl Responder> {
    let c = web::block(move || db.get())
        .await??;
    return Ok(serde_json::to_string(&data::Challenge::generate(&c, &signed.into_inner().verify_user(&c)?)?)?);
}

#[post("/auth/unregister")]
async fn verify(db: web::Data<Pool>, signed: web::Json<data::Signed<data::By<()>>>) -> Result<impl Responder> {
    let signed = signed.into_inner();
    let c = web::block(move || db.get())
        .await??;
    web::block(move || signed.verify(&c))
        .await??;
    Ok(HttpResponse::Ok().body("Deleted"))
}