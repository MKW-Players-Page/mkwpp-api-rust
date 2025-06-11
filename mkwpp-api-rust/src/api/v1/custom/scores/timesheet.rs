use crate::api::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::app_state::AppState;
use crate::sql::tables::scores::matchup::MatchupData;
use crate::sql::tables::scores::timesheet::Timesheet;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn timesheet() -> impl HttpServiceFactory {
    web::scope("/timesheet/{player_id}").default_service(web::get().to(get))
}

pub fn matchup() -> impl HttpServiceFactory {
    web::scope("/matchup").default_service(web::post().to(get_matchup))
}

pub async fn get(req: HttpRequest, path: web::Path<i32>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let data = data.read().unwrap();
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return AppState::pg_conn_http_error(e),
    };
    std::mem::drop(data);

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );
    let player_id = path.into_inner();

    let data = match Timesheet::timesheet(
        &mut connection,
        player_id,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Could not generate timesheet"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    crate::api::v1::send_serialized_data(data)
}

pub async fn get_matchup(req: HttpRequest, body: web::Json<Vec<i32>>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let data = data.read().unwrap();

    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return AppState::pg_conn_http_error(e),
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = match MatchupData::get_data(
        &mut connection,
        body.into_inner(),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
        params.calculate_rgb,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Could not generate timesheets"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    crate::api::v1::send_serialized_data(data)
}
