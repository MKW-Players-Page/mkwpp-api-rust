use crate::api::errors::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::api::v1::send_serialized_data;
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn chart() -> impl HttpServiceFactory {
    web::scope("/chart/{track_id}")
        .route("/dates", web::get().to(get_dates))
        .default_service(web::get().to(get))
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

pub async fn get_dates(
    req: HttpRequest,
    path: web::Path<i32>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let dates = ScoresWithPlayer::filter_charts_dates(
        &mut connection,
        path.into_inner(),
        params.category,
        params.lap_mode.unwrap_or(false),
        params.region_id,
    )
    .await?;

    let dates = dates
        .iter()
        .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
        .collect();

    send_serialized_data::<Vec<i64>>(dates)
}
