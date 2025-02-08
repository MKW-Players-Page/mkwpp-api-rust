use actix_web::{dev::HttpServiceFactory, middleware, web};

pub mod cups;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1")
        .wrap(middleware::NormalizePath::default())
        .route("/cups", web::get().to(cups::get));
}
