use prelude::*;
mod prelude {
    pub use crate::prelude::*;
    pub use actix_web::{
        App,
        get,
        http::header::ContentType,
        HttpResponse,
        HttpServer,
        post,
        Responder,
        web::{
            self,
            Data,
            Json,
            Path
        }
    };
}

pub mod auth;
pub mod dictionary;