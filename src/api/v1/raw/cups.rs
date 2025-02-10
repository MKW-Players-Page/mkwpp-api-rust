use crate::sql::tables::cups;
use actix_web::{HttpResponse, Responder};

pub async fn get() -> impl Responder {
    let cups = cups::Cups::reg_track_cups();
    match serde_json::to_string(&cups) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string())
        }
    }
}
