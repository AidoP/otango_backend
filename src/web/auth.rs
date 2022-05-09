use super::prelude::*;

#[post("/auth/register")]
async fn register(db: Data<Pool>, signed: Json<data::Signed<data::Certificate>>) -> Result<impl Responder> {
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
async fn challenge(db: Data<Pool>, signed: Json<Signed<String>>) -> Result<impl Responder> {
    let c = web::block(move || db.get())
        .await??;
    return Ok(serde_json::to_string(&data::Challenge::generate(&c, &signed.into_inner().verify_user(&c)?)?)?);
}