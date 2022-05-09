use super::prelude::*;

#[post("/word/{word}")]
async fn set_word(db: Data<Pool>, path: Path<String>, signed: Json<Signed<By<data::Word>>>) -> Result<impl Responder> {
    let c = web::block(move || db.get())
        .await??;
    let word = signed.into_inner().privileged(&c)?;
    web::block(move || word.insert(&c)).await??;
    Ok(HttpResponse::NoContent())
}
#[get("/word/{word}")]
async fn get_word(db: Data<Pool>, path: Path<String>) -> Result<impl Responder> {
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
async fn get_kanji(db: Data<Pool>, path: Path<char>) -> Result<impl Responder> {
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