use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::close_connection,
    },
    auth::is_user_admin,
    sql::tables::regions::{RegionType, Regions},
};

pub fn regions() -> impl HttpServiceFactory {
    web::scope("/regions")
        .route("/insert", web::put().to(insert_or_edit))
        .route("/edit", web::patch().to(insert_or_edit))
        .route(
            "/delete",
            web::delete().to(crate::api::v1::delete_by_id::<Regions>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/insert", "/edit", "/delete");

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertOrEditBody {
    id: Option<i32>,
    code: String,
    region_type: RegionType,
    parent_id: Option<i32>,
    is_ranked: bool,
    session_token: String,
}

async fn insert_or_edit(
    body: web::Json<InsertOrEditBody>,
) -> Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    if !is_user_admin(
        crate::auth::get_user_data(&body.session_token, &mut connection)
            .await?
            .user_id,
        &mut connection,
    )
    .await?
    {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    Regions::insert_or_edit(
        &mut connection,
        body.id,
        &body.code,
        body.region_type,
        body.parent_id,
        body.is_ranked,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}