use crate::sql::tables::players::{FilterByPlayerId, Players, players_basic::PlayersBasic};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn players() -> impl HttpServiceFactory {
    return web::scope("/players")
        .route(
            "/list",
            web::get().to(crate::api::v1::get_star_query::<PlayersBasic>),
        )
        .route("/select", web::get().to(get_with_decode::<Players>))
        .route(
            "/select_basic",
            web::get().to(get_with_decode::<PlayersBasic>),
        )
        .default_service(web::get().to(default));
}
default_paths_fn!("/list", "/select", "/select_basic");

pub async fn get_with_decode<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + FilterByPlayerId,
>(
    data: web::Data<crate::AppState>,
    body: web::Bytes,
) -> HttpResponse {
    let json_string = match String::from_utf8(body.to_vec()) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Couldn't turn request body bytes into utf8 string",
                    &e.to_string(),
                ));
        }
    };

    let player_ids = match serde_json::from_str::<Vec<i32>>(&json_string) {
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Couldn't turn request body into the right json",
                    &e.to_string(),
                ));
        }
        Ok(v) => v,
    };

    return crate::api::v1::basic_get::<Table>(data, async |x| {
        Table::get_select_players(x, player_ids).await
    })
    .await;
}
