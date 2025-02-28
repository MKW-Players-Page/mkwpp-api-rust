use crate::sql::tables::BasicTableQueries;
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

mod cups;

const PLAYER_AWARDS_PATH: &str = "/player_awards";
const SITE_CHAMPS_PATH: &str = "/site_champs";
const CUPS_PATH: &str = "/cups";
const EDIT_SUBMISSIONS_PATH: &str = "/edit_submissions";
const PLAYERS_PATH: &str = "/players";
const REGIONS_PATH: &str = "/regions";
const SCORES_PATH: &str = "/scores";
const STANDARD_LEVELS_PATH: &str = "/standard_levels";
const STANDARDS_PATH: &str = "/standards";
const SUBMISSIONS_PATH: &str = "/submissions";
const TRACKS_PATH: &str = "/tracks";

pub fn raw() -> impl HttpServiceFactory {
    return web::scope("/raw")
        .guard(actix_web::guard::Get())
        .default_service(web::get().to(default))
        .route(
            PLAYER_AWARDS_PATH,
            web::get().to(get::<crate::sql::tables::awards::Awards>),
        )
        .route(
            SITE_CHAMPS_PATH,
            web::get().to(get::<crate::sql::tables::champs::Champs>),
        )
        .route(CUPS_PATH, web::get().to(cups::get))
        .route(
            EDIT_SUBMISSIONS_PATH,
            web::get().to(get::<crate::sql::tables::edit_submissions::EditSubmissions>),
        )
        .route(
            PLAYERS_PATH,
            web::get().to(get::<crate::sql::tables::players::Players>),
        )
        .route(
            REGIONS_PATH,
            web::get().to(get::<crate::sql::tables::regions::Regions>),
        )
        .route(
            SCORES_PATH,
            web::get().to(get::<crate::sql::tables::scores::Scores>),
        )
        .route(
            STANDARD_LEVELS_PATH,
            web::get().to(get::<crate::sql::tables::standard_levels::StandardLevels>),
        )
        .route(
            STANDARDS_PATH,
            web::get().to(get::<crate::sql::tables::standards::Standards>),
        )
        .route(
            SUBMISSIONS_PATH,
            web::get().to(get::<crate::sql::tables::submissions::Submissions>),
        )
        .route(
            TRACKS_PATH,
            web::get().to(get::<crate::sql::tables::tracks::Tracks>),
        );
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok().content_type("application/json").body(format!("{{\"paths\":[\"{PLAYER_AWARDS_PATH}\",\"{SITE_CHAMPS_PATH}\",\"{CUPS_PATH}\",\"{EDIT_SUBMISSIONS_PATH}\",\"{PLAYERS_PATH}\",\"{REGIONS_PATH}\",\"{SCORES_PATH}\",\"{STANDARD_LEVELS_PATH}\",\"{STANDARDS_PATH}\",\"{SUBMISSIONS_PATH}\",\"{TRACKS_PATH}\"]}}"));
}

pub async fn get<
    Table: for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + serde::Serialize + BasicTableQueries,
>(
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = Table::select_star_query(&mut connection).await;
    return crate::api::v1::handle_basic_get::<Table>(rows_request, connection).await;
}
