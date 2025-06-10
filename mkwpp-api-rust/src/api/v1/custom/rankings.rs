use crate::api::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::api::v1::{close_connection, send_serialized_data};
use crate::app_state::AppState;
use crate::sql::tables::scores::rankings::{RankingType, Rankings};
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

macro_rules! ranking {
    ($fn_name:ident, $enum_variant:ident, $default_val:expr) => {
        async fn $fn_name(req: HttpRequest) -> HttpResponse {
            return get(RankingType::$enum_variant($default_val), req).await;
        }
    };
}

pub fn rankings() -> impl HttpServiceFactory {
    web::scope("/rankings")
        .route("/totaltime", web::get().to(total_time))
        .route("/prwr", web::get().to(prwr))
        .route("/tally", web::get().to(tally))
        .route("/af", web::get().to(af))
        .route("/arr", web::get().to(arr))
        .default_service(web::get().to(default))
}
default_paths_fn!("/af", "/arr", "/tally", "/prwr", "/totaltime");

ranking!(af, AverageFinish, 0.0);
ranking!(arr, AverageRankRating, 0.0);
ranking!(prwr, PersonalRecordWorldRecord, 0.0);
ranking!(tally, TallyPoints, 0);
ranking!(total_time, TotalTime, 0);

async fn get(ranking_type: RankingType, req: HttpRequest) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let data = data.read().unwrap();

    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return AppState::pg_conn_http_error(e),
    };

    std::mem::drop(data);

    let data = match Rankings::get(
        &mut connection,
        ranking_type,
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
                String::from("Error getting data from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    if let Err(e) = close_connection(connection).await {
        return e;
    }
    send_serialized_data(data)
}
