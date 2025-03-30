use crate::api::FinalErrorResponse;
use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::timesheet::Timesheet;
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

pub fn timesheet() -> impl HttpServiceFactory {
    web::scope("/timesheet/{player_id}").default_service(web::get().to(get))
}

// TODO: Incredibly, incredibly unoptimized
pub async fn get(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );
    let player_id = path.into_inner();

    let data = match Timesheet::timesheet(
        &mut connection,
        player_id,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Could not generate timesheet"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    crate::api::v1::send_serialized_data(data)
}
