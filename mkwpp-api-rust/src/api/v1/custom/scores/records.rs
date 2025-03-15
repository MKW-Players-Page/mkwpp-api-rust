use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn records() -> impl HttpServiceFactory {
    return web::scope("/records").default_service(web::get().to(get));
}

pub async fn get(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    return crate::api::v1::basic_get::<ScoresWithPlayer>(data, async |x| {
        ScoresWithPlayer::get_records(
            x,
            params.category,
            params.lap_mode,
            params.date,
            params.region_id,
        )
        .await
    })
    .await;
}
