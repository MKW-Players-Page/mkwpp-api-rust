use crate::api::errors::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn chart() -> impl HttpServiceFactory {
    web::scope("/chart/{track_id}").default_service(web::get().to(get))
}

pub async fn get(
    req: HttpRequest,
    path: web::Path<i32>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    return crate::api::v1::basic_get::<ScoresWithPlayer>(async |x| {
        return ScoresWithPlayer::filter_charts(
            x,
            path.into_inner(),
            params.category,
            params.lap_mode.unwrap_or(false),
            params.date,
            params.region_id,
            params.limit,
        )
        .await;
    })
    .await;
}
