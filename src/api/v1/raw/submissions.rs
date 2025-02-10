use crate::sql::tables::submissions;
use crate::sql::tables::BasicTableQueries;
use actix_web::{web, HttpResponse, Responder};
use sqlx::FromRow;

pub async fn get(data: web::Data<crate::AppState>) -> impl Responder {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let submission_rows = match submissions::Submissions::select_star_query(&mut connection).await {
        Ok(rows) => {
            connection.close().await.unwrap();
            rows
        }
        Err(e) => {
            connection.close().await.unwrap();
            return HttpResponse::Ok().content_type("application/json").body(
                crate::api::v1::generate_error_json_string(
                    "Couldn't get rows from database",
                    e.to_string().as_str(),
                ),
            );
        }
    };

    let submissions = submission_rows
        .into_iter()
        .map(|r| return submissions::Submissions::from_row(&r).unwrap())
        .collect::<Vec<submissions::Submissions>>();

    match serde_json::to_string(&submissions) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string())
        }
    }
}
