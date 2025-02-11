use actix_web::{dev::HttpServiceFactory, web};

mod recent;

pub fn scores() -> impl HttpServiceFactory {
    return web::scope("/scores")
        .service(recent::recent())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/recent"]}"#);
}
