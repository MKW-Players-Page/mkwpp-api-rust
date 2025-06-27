use crate::{
    api::errors::FinalErrorResponse,
    sql::tables::players::{FilterPlayers, Players, players_basic::PlayersBasic},
};
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
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + FilterPlayers,
>(
    body: web::Json<Vec<i32>>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let player_ids = body.into_inner();
    return crate::api::v1::basic_get::<Table>(async |x| {
        return Table::get_select_players(x, player_ids).await;
    })
    .await;
}
