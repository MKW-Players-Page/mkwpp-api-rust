use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::rankings::{RankingType, Rankings};
use crate::sql::tables::scores::timesheet::{Times, Timesheet};
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

    let times_request = Timesheet::get_times(
        &mut connection,
        player_id,
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    let af_request = Rankings::get(
        &mut connection,
        RankingType::AverageFinish(0.0),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    let totals_request = Rankings::get(
        &mut connection,
        RankingType::TotalTime(0),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    let tally_request = Rankings::get(
        &mut connection,
        RankingType::TallyPoints(0),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    let arr_request = Rankings::get(
        &mut connection,
        RankingType::AverageRankRating(0.0),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    let prwr_request = Rankings::get(
        &mut connection,
        RankingType::PersonalRecordWorldRecord(0.0),
        params.category,
        params.lap_mode,
        params.date,
        params.region_id,
    )
    .await;

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    let times_rows = match crate::api::v1::match_rows(times_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let times = match crate::api::v1::decode_rows_to_table::<Times>(times_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let af_rows = match crate::api::v1::match_rows(af_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let af = match crate::api::v1::decode_rows_to_table::<Rankings>(af_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let totals_rows = match crate::api::v1::match_rows(totals_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let totals = match crate::api::v1::decode_rows_to_table::<Rankings>(totals_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let tally_rows = match crate::api::v1::match_rows(tally_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let tally = match crate::api::v1::decode_rows_to_table::<Rankings>(tally_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let arr_rows = match crate::api::v1::match_rows(arr_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let arr = match crate::api::v1::decode_rows_to_table::<Rankings>(arr_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let prwr_rows = match crate::api::v1::match_rows(prwr_request) {
        Ok(rows) => rows,
        Err(e) => return e,
    };

    let prwr = match crate::api::v1::decode_rows_to_table::<Rankings>(prwr_rows) {
        Ok(data) => data,
        Err(e) => return e,
    };

    // TODO: wtf is this bs
    crate::api::v1::send_serialized_data(Timesheet::new(
        times,
        af.iter()
            .find(|r| r.player.id == player_id)
            .map(|found| <RankingType as TryInto<f64>>::try_into(found.value.clone()).unwrap()),
        arr.iter()
            .find(|r| r.player.id == player_id)
            .map(|found| <RankingType as TryInto<f64>>::try_into(found.value.clone()).unwrap()),
        totals
            .iter()
            .find(|r| r.player.id == player_id)
            .map(|found| <RankingType as TryInto<i32>>::try_into(found.value.clone()).unwrap()),
        prwr.iter()
            .find(|r| r.player.id == player_id)
            .map(|found| <RankingType as TryInto<f64>>::try_into(found.value.clone()).unwrap()),
        tally
            .iter()
            .find(|r| r.player.id == player_id)
            .map(|found| <RankingType as TryInto<i16>>::try_into(found.value.clone()).unwrap()),
    ))
}
