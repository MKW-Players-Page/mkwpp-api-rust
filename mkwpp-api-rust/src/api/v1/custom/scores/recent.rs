use crate::sql::tables::scores::with_player::ScoresWithPlayer;
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

macro_rules! get_fn {
    ($fn_name:ident, $handle:ident) => {
        async fn $fn_name(path: web::Path<i32>, data: web::Data<crate::AppState>) -> HttpResponse {
            return crate::api::v1::basic_get::<ScoresWithPlayer>(data, async |x| {
                ScoresWithPlayer::$handle(x, path.into_inner()).await
            })
            .await;
        }
    };
}

pub fn recent() -> impl HttpServiceFactory {
    web::scope("/recent")
        .guard(actix_web::guard::Get())
        .service(
            web::scope("/{limit}")
                .route("/all", web::get().to(get_all))
                .route("/records", web::get().to(get_all_records)),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/:limit/records", "/:limit/all");

get_fn!(get_all, order_by_date);
get_fn!(get_all_records, order_records_by_date);
