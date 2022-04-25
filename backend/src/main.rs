use actix_web::{error, Error, get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, http::header::ContentType, error::ErrorInternalServerError};
use actix_cors::Cors;
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype};

type Pool = r2d2::Pool<SqliteConnectionManager>;
use r2d2_sqlite::SqliteConnectionManager;

use common::data;

const FRONTEND_LOCATION: Option<&'static str> = option_env!("FRONTEND_LOCATION");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut ssl = SslAcceptor::mozilla_modern_v5(SslMethod::tls())?;
    ssl.set_private_key_file("key.pem", SslFiletype::PEM)?;
    ssl.set_certificate_chain_file("cert.pem")?;

    let manager = SqliteConnectionManager::file("お単語.db");
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        let cors = Cors::default();
        let cors = if let Some(location) = FRONTEND_LOCATION {
            cors.allowed_origin(location)
        } else {
            cors.allow_any_origin()
        };
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(index)
            .service(word)
            .service(kanji)
            //.service(all_words)
            //.service(backup)
    }).bind_openssl(("0.0.0.0", 8000), ssl)?
        .run()
        .await
}

#[get("/")]
async fn index() -> impl Responder {
    if let Some(location) = FRONTEND_LOCATION {
        HttpResponse::PermanentRedirect()
            .append_header(("Location", location))
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

#[get("/単語/{word}")]
async fn word(db: web::Data<Pool>, path: web::Path<String>) -> Result<impl Responder, Error> {
    let word = path.into_inner();
    let c = web::block(move || db.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;
    let word = web::block(move || data::Word::get(&c, &word))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    if let Some(word) = word {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&word)?)
        )
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[get("/漢字/{kanji}")]
async fn kanji(db: web::Data<Pool>, path: web::Path<char>) -> Result<impl Responder, Error> {
    let kanji = path.into_inner();
    let c = web::block(move || db.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;
    let kanji = web::block(move || data::Kanji::get(&c, kanji))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    if let Some(kanji) = kanji {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&kanji)?)
        )
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}