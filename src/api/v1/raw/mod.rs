use crate::sql::tables::BasicTableQueries;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};

mod cups;

const PLAYER_AWARDS_PATH: &'static str = "/player_awards";
const SITE_CHAMPS_PATH: &'static str = "/site_champs";
const CUPS_PATH: &'static str = "/cups";
const EDIT_SUBMISSIONS_PATH: &'static str = "/edit_submissions";
const PLAYERS_PATH: &'static str = "/players";
const REGIONS_PATH: &'static str = "/regions";
const SCORES_PATH: &'static str = "/scores";
const STANDARD_LEVELS_PATH: &'static str = "/standard_levels";
const STANDARDS_PATH: &'static str = "/standards";
const SUBMISSIONS_PATH: &'static str = "/submissions";
const TRACKS_PATH: &'static str = "/tracks";

pub fn raw() -> impl HttpServiceFactory {
    return web::scope("/raw")
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
    Table: BasicTableQueries + serde::Serialize + for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>,
>(
    data: web::Data<crate::AppState>,
) -> impl Responder {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let region_rows = match Table::select_star_query(&mut connection).await {
        Ok(rows) => {
            connection.close().await.unwrap();
            rows
        }
        Err(e) => {
            connection.close().await.unwrap();
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(crate::api::generate_error_json_string(
                    "Couldn't get rows from database",
                    e.to_string().as_str(),
                ));
        }
    };

    let regions = region_rows
        .into_iter()
        .map(|r| return Table::from_row(&r).unwrap())
        .collect::<Vec<Table>>();

    match serde_json::to_string(&regions) {
        Ok(v) => return HttpResponse::Ok().content_type("application/json").body(v),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(e.to_string())
        }
    }
}
