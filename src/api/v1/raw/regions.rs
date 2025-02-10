use crate::sql::tables::regions;
use crate::sql::tables::BasicTableQueries;
use actix_web::{web, HttpResponse, Responder};
use sqlx::FromRow;

pub async fn get(data: web::Data<crate::AppState>) -> impl Responder {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let region_rows = match regions::Regions::select_star_query(&mut connection).await {
        Ok(rows) => {
            connection.close().await.unwrap();
            rows
        }
        Err(e) => {
            connection.close().await.unwrap();
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(crate::api::v1::generate_error_json_string(
                    "Couldn't get rows from database",
                    e.to_string().as_str(),
                ));
        }
    };

    let regions = region_rows
        .into_iter()
        .map(|r| regions::Regions::from_row(&r).unwrap())
        .collect::<Vec<regions::Regions>>();

    match serde_json::to_string(&regions) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string())
        }
    }
}
