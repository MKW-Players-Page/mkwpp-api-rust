use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::{DeleteBody, close_connection, decode_row_to_table, send_serialized_data},
    },
    auth::is_user_admin,
    custom_serde::DateAsTimestampNumber,
    sql::tables::{Category, scores::Scores},
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn scores() -> impl HttpServiceFactory {
    web::scope("/scores")
        .route("/list", web::post().to(list))
        .route("/id", web::post().to(get_by_id))
        .route("/insert", web::put().to(insert_or_edit))
        .route("/edit", web::patch().to(insert_or_edit))
        .route(
            "/delete",
            web::delete().to(crate::api::v1::delete_by_id::<Scores>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/id", "/insert", "/edit", "/delete");

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListBody {
    pub session_token: String,
    pub track_id: i32,
}

async fn list(body: web::Json<ListBody>) -> Result<HttpResponse, FinalErrorResponse> {
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

    crate::api::v1::basic_get::<Scores>(async |x| Scores::filter_by_track(body.track_id, x).await)
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertOrEditBody {
    id: Option<i32>,
    value: i32,
    category: Category,
    is_lap: bool,
    player_id: i32,
    track_id: i32,
    #[serde(deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp")]
    date: Option<chrono::NaiveDate>,
    video_link: Option<String>,
    ghost_link: Option<String>,
    comment: Option<String>,
    admin_note: Option<String>,
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

    Scores::insert_or_edit(
        body.id,
        body.value,
        body.category,
        body.is_lap,
        body.player_id,
        body.track_id,
        body.date,
        body.video_link,
        body.ghost_link,
        body.comment,
        body.admin_note,
        &mut connection,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}

async fn get_by_id(body: web::Json<DeleteBody>) -> Result<HttpResponse, FinalErrorResponse> {
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

    let data = Scores::get_from_id(body.id, &mut connection).await?;

    close_connection(connection).await?;
    send_serialized_data(decode_row_to_table::<Scores>(data)?)
}
