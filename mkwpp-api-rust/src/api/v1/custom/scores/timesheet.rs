use crate::api::errors::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::api::v1::send_serialized_data;
use crate::sql::tables::scores::matchup::MatchupData;
use crate::sql::tables::scores::timesheet::Timesheet;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn timesheet() -> impl HttpServiceFactory {
    web::scope("/timesheet/{player_id}")
        .route("/dates", web::get().to(get_dates))
        .default_service(web::get().to(get))
}

pub fn matchup() -> impl HttpServiceFactory {
    web::scope("/matchup").default_service(web::post().to(get_matchup))
}

pub async fn get(
    req: HttpRequest,
    path: web::Path<i32>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );
    let player_id = path.into_inner();

    let data = Timesheet::timesheet(
        &mut connection,
        player_id,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await?;

    crate::api::v1::close_connection(connection).await?;

    crate::api::v1::send_serialized_data(data)
}

pub async fn get_matchup(
    req: HttpRequest,
    body: web::Json<Vec<i32>>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = MatchupData::get(
        &mut connection,
        body.into_inner(),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await?;

    crate::api::v1::close_connection(connection).await?;

    crate::api::v1::send_serialized_data(data)
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

    let dates = Timesheet::filter_player_dates(
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
