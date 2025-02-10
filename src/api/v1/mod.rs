use actix_web::{dev::HttpServiceFactory, web};

mod raw;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1")
        .service(raw::raw())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/raw"]}"#);
}
