use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn recent() -> impl HttpServiceFactory {
    return web::scope("/recent")
        .guard(actix_web::guard::Get())
        .service(
            web::scope("/{limit}")
                .route("/all", web::get().to(get_all))
                .route("/records", web::get().to(get_all_records)),
        )
        .default_service(web::get().to(default));
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"paths":["/:limit/records","/:limit/all"]}"#);
}

pub async fn get_all(path: web::Path<i32>, data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = ScoresWithPlayer::order_by_date(&mut connection, path.into_inner()).await;
    return crate::api::v1::handle_basic_get::<ScoresWithPlayer>(rows_request, connection).await;
}

pub async fn get_all_records(
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request =
        ScoresWithPlayer::order_records_by_date(&mut connection, path.into_inner()).await;
    return crate::api::v1::handle_basic_get::<ScoresWithPlayer>(rows_request, connection).await;
}
