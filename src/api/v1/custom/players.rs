use crate::sql::tables::BasicTableQueries;
use crate::sql::tables::players::Players;
use crate::sql::tables::players::players_basic::PlayersBasic;
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn players() -> impl HttpServiceFactory {
    return web::scope("/players")
        .route("/list", web::get().to(list))
        .route("/select", web::get().to(select))
        .route("/select_basic", web::get().to(select_basic))
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/list","/select","/select_basic"]}"#);
}

pub async fn list(data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = PlayersBasic::select_star_query(&mut connection).await;
    return crate::api::v1::handle_basic_get::<PlayersBasic>(rows_request, connection).await;
}

pub async fn select(data: web::Data<crate::AppState>, body: web::Bytes) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

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

    let rows_request = Players::get_select_players(&mut connection, player_ids).await;
    return crate::api::v1::handle_basic_get::<Players>(rows_request, connection).await;
}

pub async fn select_basic(data: web::Data<crate::AppState>, body: web::Bytes) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

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

    let rows_request = PlayersBasic::get_select_players(&mut connection, player_ids).await;
    return crate::api::v1::handle_basic_get::<PlayersBasic>(rows_request, connection).await;
}
