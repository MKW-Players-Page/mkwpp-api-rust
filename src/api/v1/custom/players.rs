use crate::sql::tables::players::players_basic::PlayersBasic;
use crate::sql::tables::BasicTableQueries;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

pub fn players() -> impl HttpServiceFactory {
    return web::scope("/players").default_service(web::get().to(get));
}

pub async fn get(data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = PlayersBasic::select_star_query(&mut connection).await;
    return crate::api::v1::handle_basic_get::<PlayersBasic>(rows_request, connection).await;
}
