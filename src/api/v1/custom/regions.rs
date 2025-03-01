use crate::sql::tables::regions::Regions;
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

macro_rules! region_fn {
    ($fn_name:ident, $handle:expr) => {
        async fn $fn_name(path: web::Path<i32>, data: web::Data<crate::AppState>) -> HttpResponse {
            return basic_get_i32(path, data, $handle).await;
        }
    };
}

pub fn regions() -> impl HttpServiceFactory {
    return web::scope("/regions")
        .service(web::scope("/ancestors/{player_id}").default_service(web::get().to(get_ancestors)))
        .service(
            web::scope("/descendants/{player_id}").default_service(web::get().to(get_descendants)),
        )
        .default_service(web::get().to(default));
}

default_paths_fn!("/ancestors/:regionId", "/descendants/:regionId");

region_fn!(get_ancestors, Regions::get_ancestors);
region_fn!(get_descendants, Regions::get_descendants);

pub async fn basic_get_i32(
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
    rows_function: impl AsyncFnOnce(&mut sqlx::PgConnection, i32) -> Result<Vec<i32>, sqlx::Error>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = rows_function(&mut connection, path.into_inner()).await;

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
                ));
        }
    };

    return crate::api::v1::send_serialized_data(rows);
}
