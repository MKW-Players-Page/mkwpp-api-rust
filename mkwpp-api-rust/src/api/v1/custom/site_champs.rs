use crate::{
    api::errors::FinalErrorResponse,
    sql::tables::{Category, champs::Champs},
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn site_champs() -> impl HttpServiceFactory {
    web::scope("/site_champs")
        .service(web::scope("/category/{track_id}").default_service(web::get().to(by_category)))
        .default_service(web::get().to(default))
}
default_paths_fn!("/category/:categoryId");

async fn by_category(path: web::Path<u8>) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    return crate::api::v1::basic_get::<Champs>(async |x| {
        return Champs::filter_by_category(
            Category::try_from(path.into_inner()).unwrap_or(Category::NonSc),
            x,
        )
        .await;
    })
    .await;
}
