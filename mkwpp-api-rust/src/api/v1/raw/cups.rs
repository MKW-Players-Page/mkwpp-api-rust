use crate::sql::tables::cups;
use actix_web::Responder;

pub async fn get() -> impl Responder {
    let cups = cups::Cups::reg_track_cups();
    crate::api::v1::send_serialized_data(cups)
}
