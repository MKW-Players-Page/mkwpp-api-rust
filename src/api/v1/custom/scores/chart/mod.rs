use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use crate::sql::tables::Category;
use actix_web::{dev::HttpServiceFactory, web, HttpRequest, HttpResponse};

pub fn course() -> impl HttpServiceFactory {
    return web::scope("/chart/{track_id}").default_service(web::get().to(get));
}

#[derive(serde::Deserialize, Debug)]
struct Params {
    cat: Option<u8>,
    lap: Option<u8>,
    dat: Option<String>,
    reg: Option<i32>,
}

pub async fn get(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let track = path.into_inner();
    let category = params
        .cat
        .and_then(|x| Category::try_from(x).ok())
        .unwrap_or(Category::NonSc);
    let is_lap = params.lap.map(|x| x == 1).unwrap_or(false);
    let max_date = params
        .dat
        .as_ref()
        .and_then(|x| chrono::NaiveDate::parse_from_str(x, "%F").ok())
        .unwrap_or(chrono::Local::now().date_naive());

    let rows_request =
        ScoresWithPlayer::filter_charts(&mut connection, track, category, is_lap, max_date).await;
    return crate::api::v1::handle_basic_get::<ScoresWithPlayer>(rows_request, connection).await;
}
