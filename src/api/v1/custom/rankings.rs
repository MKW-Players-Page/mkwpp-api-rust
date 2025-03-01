use crate::api::v1::custom::params::{Params, ParamsDestructured};
use crate::sql::tables::scores::rankings::{RankingType, Rankings};
use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

macro_rules! ranking {
    ($fn_name:ident, $enum_variant:ident, $default_val:expr) => {
        async fn $fn_name(req: HttpRequest, data: web::Data<crate::AppState>) -> HttpResponse {
            return get(RankingType::$enum_variant($default_val), req, data).await;
        }
    };
}

pub fn rankings() -> impl HttpServiceFactory {
    return web::scope("/rankings")
        .route("/totaltime", web::get().to(total_time))
        .route("/prwr", web::get().to(prwr))
        .route("/tally", web::get().to(tally))
        .route("/af", web::get().to(af))
        .route("/arr", web::get().to(arr))
        .default_service(web::get().to(default));
}
default_paths_fn!("/af", "/arr", "/tally", "/prwr", "/totaltime");

ranking!(af, AverageFinish, 0.0);
ranking!(arr, AverageRankRating, 0.0);
ranking!(prwr, PersonalRecordWorldRecord, 0.0);
ranking!(tally, TallyPoints, 0);
ranking!(total_time, TotalTime, 0);

async fn get(
    ranking_type: RankingType,
    req: HttpRequest,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    return crate::api::v1::basic_get::<Rankings>(data, async |x| {
        Rankings::get(
            x,
            ranking_type,
            params.category,
            params.lap_mode,
            params.date,
            params.region_id,
        )
        .await
    })
    .await;
}
