use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::rankings::{RankingType, Rankings};
use actix_web::{dev::HttpServiceFactory, web, HttpRequest, HttpResponse};

pub fn rankings() -> impl HttpServiceFactory {
    return web::scope("/rankings")
        .route("/totaltime", web::get().to(total_time))
        .route("/prwr", web::get().to(personal_record_world_record))
        .route("/tally", web::get().to(tally))
        .route("/af", web::get().to(af))
        .route("/arr", web::get().to(arr))
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/af","/arr","/tally","/prwr","/totaltime"]}"#);
}

async fn af(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    return get(RankingType::AverageFinish(0.0), req, data).await;
}

async fn arr(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    return get(RankingType::AverageRankRating(0.0), req, data).await;
}

async fn tally(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    return get(RankingType::TallyPoints(0), req, data).await;
}

async fn total_time(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    return get(RankingType::TotalTime(0), req, data).await;
}

async fn personal_record_world_record(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    return get(RankingType::PersonalRecordWorldRecord(0.0), req, data).await;
}

async fn get(ranking_type: RankingType, req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let rows_request = Rankings::get(
        &mut connection,
        ranking_type,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;
    
    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    let rows = match crate::api::v1::match_rows(rows_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let data = match crate::api::v1::decode_rows_to_table::<Rankings>(rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    return crate::api::v1::send_serialized_data(data);
}
