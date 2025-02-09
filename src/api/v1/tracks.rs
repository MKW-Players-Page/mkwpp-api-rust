use crate::sql::tables::tracks;
use actix_web::{web, HttpResponse, Responder};
use sqlx::FromRow;

pub async fn get(data: web::Data<crate::AppState>) -> impl Responder {
    let mut connection = match data.pg_pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::Ok().content_type("application/json").body(
                super::generate_error_json_string(
                    "Couldn't get connection from data pool",
                    e.to_string().as_str(),
                ),
            )
        }
    };

    let track_rows = match tracks::Tracks::select_star_query(&mut connection).await {
        Ok(rows) => {
            connection.close().await.unwrap();
            rows
        }
        Err(e) => {
            connection.close().await.unwrap();
            return HttpResponse::Ok().content_type("application/json").body(
                super::generate_error_json_string(
                    "Couldn't get rows from database",
                    e.to_string().as_str(),
                ),
            );
        }
    };

    let tracks = track_rows
        .into_iter()
        .map(|r| tracks::Tracks::from_row(&r).unwrap())
        .collect::<Vec<tracks::Tracks>>();

    match serde_json::to_string(&tracks) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string())
        }
    }
}
