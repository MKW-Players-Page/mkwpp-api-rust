use actix_web::{dev::HttpServiceFactory, web};

mod chart;
mod recent;
mod records;
mod timesheet;

pub fn scores() -> impl HttpServiceFactory {
    return web::scope("/scores")
        .service(recent::recent())
        .service(chart::chart())
        .service(records::records())
        .service(timesheet::timesheet())
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/recent","/chart/:trackId","/timesheet/:playerId","/records"]}"#);
}
