use crate::{
    api::errors::{EveryReturnedError, FinalErrorResponse},
    auth::is_user_admin,
    sql::tables::BasicTableQueries,
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

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

mod admin;
pub mod auth;
mod custom;
mod raw;

pub fn v1() -> impl HttpServiceFactory {
    web::scope("/v1")
        .service(raw::raw())
        .service(custom::custom())
        .service(auth::auth())
        .service(admin::admin())
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
) -> Result<(), FinalErrorResponse> {
    connection
        .close()
        .await
        .map_err(|e| EveryReturnedError::ClosingConnectionFromPGPool.into_final_error(e))
}

pub fn decode_rows_to_table<Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>>(
    rows: impl IntoIterator<Item = sqlx::postgres::PgRow>,
) -> Result<Vec<Table>, FinalErrorResponse> {
    rows.into_iter()
        .map(|r| Table::from_row(&r))
        .collect::<Result<Vec<Table>, sqlx::Error>>()
        .map_err(|e| EveryReturnedError::DecodingDatabaseRows.into_final_error(e))
}

pub fn decode_row_to_table<Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>>(
    row: sqlx::postgres::PgRow,
) -> Result<Table, FinalErrorResponse> {
    Table::from_row(&row).map_err(|e| EveryReturnedError::DecodingDatabaseRows.into_final_error(e))
}

pub fn send_serialized_data<T: serde::Serialize>(
    data: T,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    serde_json::to_string(&data)
        .map(|v| HttpResponse::Ok().content_type("application/json").body(v))
        .map_err(|e| EveryReturnedError::SerializingDataToJSON.into_final_error(e))
}

pub async fn handle_basic_get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize,
>(
    rows: Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse>,
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    close_connection(connection).await?;
    let data = decode_rows_to_table::<Table>(rows?)?;
    send_serialized_data(data)
}

pub async fn basic_get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize,
>(
    rows_function: impl AsyncFnOnce(
        &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
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
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse>,
    modifier_function: impl AsyncFnOnce(&[Table]) -> T,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let rows = rows_function(&mut connection).await?;
    close_connection(connection).await?;
    let data = decode_rows_to_table::<Table>(rows)?;
    let data = modifier_function(&data).await;

    send_serialized_data(data)
}

pub async fn get_star_query<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + BasicTableQueries,
>() -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    basic_get::<Table>(Table::select_star_query).await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteBody {
    id: i32,
    session_token: String,
}

async fn delete_by_id<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + BasicTableQueries,
>(
    body: web::Json<DeleteBody>,
) -> Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    if !is_user_admin(
        crate::auth::get_user_data(&body.session_token, &mut connection)
            .await?
            .user_id,
        &mut connection,
    )
    .await?
    {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    Table::delete_by_id(body.id, &mut connection).await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}
