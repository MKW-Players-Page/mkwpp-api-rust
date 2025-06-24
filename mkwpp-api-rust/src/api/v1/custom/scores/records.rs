use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn records() -> impl HttpServiceFactory {
    web::scope("/records").default_service(web::get().to(get))
}

pub async fn get(req: HttpRequest) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    return crate::api::v1::basic_get::<ScoresWithPlayer>(async |x| {
        return ScoresWithPlayer::get_records(
            x,
            params.category,
            params.lap_mode,
            params.date,
            params.region_id,
        )
        .await;
    })
    .await;
}
