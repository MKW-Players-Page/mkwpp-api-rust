use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn records() -> impl HttpServiceFactory {
    return web::scope("/records").default_service(web::get().to(get));
}

pub async fn get(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let rows_request = ScoresWithPlayer::get_records(
        &mut connection,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    let rows = match crate::api::v1::match_rows(rows_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let data = match crate::api::v1::decode_rows_to_table::<ScoresWithPlayer>(rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    return crate::api::v1::send_serialized_data(data);
}
