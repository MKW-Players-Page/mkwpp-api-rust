use actix_web::{dev::HttpServiceFactory, web};

pub mod cups;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1")
        .route("/cups", web::get().to(cups::get));
}
