use crate::sql::tables::regions::Regions;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

pub fn regions() -> impl HttpServiceFactory {
    return web::scope("/regions")
        .service(web::scope("/ancestors/{player_id}").default_service(web::get().to(get_ancestors)))
        .service(
            web::scope("/descendants/{player_id}").default_service(web::get().to(get_descendants)),
        )
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/ancestors/:regionId","/descendants/:regionId"]}"#);
}

pub async fn get_ancestors(path: web::Path<i32>, data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = Regions::get_ancestors(&mut connection, path.into_inner()).await;

    return handle_basic_get_i32(rows_request, connection).await;
}

pub async fn get_descendants(
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = Regions::get_descendants(&mut connection, path.into_inner()).await;

    return handle_basic_get_i32(rows_request, connection).await;
}

pub async fn handle_basic_get_i32(
    rows_request: Result<Vec<i32>, sqlx::Error>,
    connection: sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> HttpResponse {
    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    let rows = match rows_request {
        Ok(rows) => rows,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Couldn't get rows from database",
                    e.to_string().as_str(),
                ))
        }
    };

    return crate::api::v1::send_serialized_data(rows);
}
