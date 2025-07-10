use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::close_connection,
    },
    auth::is_user_admin,
    sql::tables::players::Players,
};

pub fn players() -> impl HttpServiceFactory {
    web::scope("/players")
        .route("/list", web::post().to(list))
        .route("/insert", web::put().to(insert_or_edit))
        .route("/edit", web::patch().to(insert_or_edit))
        .route(
            "/delete",
            web::delete().to(crate::api::v1::delete_by_id::<Players>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/insert", "/edit", "/delete");

async fn list(body: web::Json<super::UserDataBody>) -> Result<HttpResponse, FinalErrorResponse> {
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

    crate::api::v1::get_star_query::<Players>().await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertOrEditBody {
    id: Option<i32>,
    name: String,
    alias: Option<String>,
    bio: Option<String>,
    pronouns: Option<String>,
    region_id: i32,
    joined_date: chrono::NaiveDate,
    last_activity: chrono::NaiveDate,
    submitters: Vec<i32>,
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

    Players::insert_or_edit(
        &mut connection,
        body.id,
        body.name,
        body.alias,
        body.bio,
        body.pronouns,
        body.region_id,
        body.joined_date,
        body.last_activity,
        body.submitters,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}
