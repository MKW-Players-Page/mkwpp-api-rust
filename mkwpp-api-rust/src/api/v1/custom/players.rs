use crate::sql::tables::players::{FilterByPlayerId, Players, players_basic::PlayersBasic};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn players() -> impl HttpServiceFactory {
    web::scope("/players")
        .route(
            "/list",
            web::get().to(crate::api::v1::get_star_query::<PlayersBasic>),
        )
        .route("/select", web::post().to(get_with_decode::<Players>))
        .route(
            "/select_basic",
            web::post().to(get_with_decode::<PlayersBasic>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/select", "/select_basic");

pub async fn get_with_decode<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + FilterByPlayerId,
>(
    data: web::Data<crate::AppState>,
    body: web::Json<Vec<i32>>,
) -> HttpResponse {
    let player_ids = body.0;
    return crate::api::v1::basic_get::<Table>(data, async |x| {
        return Table::get_select_players(x, player_ids).await;
    })
    .await;
}
