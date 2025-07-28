use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::close_connection,
    },
    auth::is_user_admin,
    custom_serde::DateAsTimestampNumber,
    sql::tables::players::Players,
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

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
    #[serde(deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp")]
    joined_date: chrono::NaiveDate,
    #[serde(deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp")]
    last_activity: chrono::NaiveDate,
    submitters: Vec<i32>,
    chadsoft_ids: Vec<String>,
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

    let chadsoft_ids = body
        .chadsoft_ids
        .iter()
        .map(|number| {
            u64::from_str_radix(number, 16)
                .map(|num| num as i64)
                .map_err(|err| EveryReturnedError::InvalidChadsoftID.into_final_error(err))
        })
        .collect::<Result<Vec<i64>, FinalErrorResponse>>()?;

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
        chadsoft_ids,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}
