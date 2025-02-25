use actix_web::{dev::HttpServiceFactory, web};

pub mod params;
mod rankings;
mod regions;
mod scores;

pub fn custom() -> impl HttpServiceFactory {
    return web::scope("/custom")
        .service(scores::scores())
        .service(rankings::rankings())
        .service(regions::regions())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/scores","/rankings","/regions"]}"#);
}
