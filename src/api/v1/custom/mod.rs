use actix_web::{dev::HttpServiceFactory, web};

pub mod params;
mod rankings;
mod scores;

pub fn custom() -> impl HttpServiceFactory {
    return web::scope("/custom")
        .service(scores::scores())
        .service(rankings::rankings())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/scores","/rankings"]}"#);
}
