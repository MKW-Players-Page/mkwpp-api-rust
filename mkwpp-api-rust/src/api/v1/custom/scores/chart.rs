use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn chart() -> impl HttpServiceFactory {
    return web::scope("/chart/{track_id}").default_service(web::get().to(get));
}

pub async fn get(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    return crate::api::v1::basic_get::<ScoresWithPlayer>(data, async |x| {
        ScoresWithPlayer::filter_charts(
            x,
            path.into_inner(),
            params.category,
            params.lap_mode.unwrap_or(false),
            params.date,
            params.region_id,
            params.limit,
        )
        .await
    })
    .await;
}
