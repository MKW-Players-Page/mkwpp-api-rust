use crate::{app_state::AppState, sql::tables::BasicTableQueries};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use super::FinalErrorResponse;

macro_rules! default_paths_fn {
    ($($y:literal),+) => {
            async fn default() -> impl actix_web::Responder {
                return actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .body(
                        const_format::str_replace!(
                            const_format::concatc!(
                                r#"{"paths":["#, $('"',$y,'"',','),+ ,"]}"),
                            ",]",
                            "]"
                        )
                    );
            }
        };
}

mod auth;
mod custom;
mod raw;

pub fn v1() -> impl HttpServiceFactory {
    web::scope("/v1")
        .service(raw::raw())
        .service(custom::custom())
        .service(auth::auth())
        .service(
            web::scope("/doc")
                .route("/style.css", web::get().to(doc_css))
                .default_service(web::get().to(doc)),
        )
        .service(web::redirect("", "/v1/doc"))
}

async fn doc() -> HttpResponse {
    crate::api::read_file(
        "frontend/doc/v1/index.html",
        "text/html",
        &mut HttpResponse::Ok(),
    )
}

async fn doc_css() -> HttpResponse {
    crate::api::read_file(
        "frontend/doc/v1/style.css",
        "text/css",
        &mut HttpResponse::Ok(),
    )
}

pub async fn close_connection(
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> Result<(), HttpResponse> {
    connection.close().await.map_err(|e| {
        FinalErrorResponse::new_no_fields(vec![
            String::from("Error closing Database connection"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError)
    })
}

pub fn match_rows(
    rows_request: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
) -> Result<Vec<sqlx::postgres::PgRow>, HttpResponse> {
    rows_request.map_err(|e| {
        FinalErrorResponse::new_no_fields(vec![
            String::from("Couldn't get rows from database"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError)
    })
}

pub fn decode_rows_to_table<Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>>(
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<Vec<Table>, HttpResponse> {
    rows.into_iter()
        .map(|r| Table::from_row(&r))
        .collect::<Result<Vec<Table>, sqlx::Error>>()
        .map_err(|e| {
            FinalErrorResponse::new_no_fields(vec![
                String::from("Error decoding rows from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError)
        })
}

pub fn send_serialized_data<T: serde::Serialize>(data: T) -> HttpResponse {
    match serde_json::to_string(&data) {
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => FinalErrorResponse::new_no_fields(vec![
            String::from("Error serializing database data"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError),
    }
}

pub async fn handle_basic_get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize,
>(
    rows_request: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> HttpResponse {
    if let Err(e) = close_connection(connection).await {
        return e;
    }

    let rows = match match_rows(rows_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let data = match decode_rows_to_table::<Table>(rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    send_serialized_data(data)
}

pub async fn basic_get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize,
>(
    rows_function: impl AsyncFnOnce(
        &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        match data.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    let rows_request = rows_function(&mut connection).await;
    handle_basic_get::<Table>(rows_request, connection).await
}

pub async fn basic_get_with_data_mod<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>,
    T: serde::Serialize,
>(
    rows_function: impl AsyncFnOnce(
        &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>,
    modifier_function: impl AsyncFnOnce(Vec<Table>) -> T,
) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        match data.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    let rows_request = rows_function(&mut connection).await;

    if let Err(e) = close_connection(connection).await {
        return e;
    }

    let rows = match match_rows(rows_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let data = match decode_rows_to_table::<Table>(rows) {
        Ok(data) => modifier_function(data).await,
        Err(e) => return e,
    };

    send_serialized_data(data)
}

pub async fn get_star_query<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + BasicTableQueries,
>() -> HttpResponse {
    basic_get::<Table>(Table::select_star_query).await
}
