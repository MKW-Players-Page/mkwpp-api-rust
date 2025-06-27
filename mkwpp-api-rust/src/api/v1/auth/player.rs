use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::{close_connection, send_serialized_data},
    },
    app_state::access_app_state,
    auth::{
        BareMinimumValidationData, get_user_data, get_user_id_from_player_id, is_user_admin,
        is_valid_token,
    },
    sql::tables::players::Players,
};

pub fn player() -> impl HttpServiceFactory {
    web::scope("/player")
        .route("/updbio", web::post().to(update_player_bio))
        .route("/updalias", web::post().to(update_player_alias))
        .route("/remsubmitter", web::post().to(remove_submitter))
        .route("/addsubmitter", web::post().to(add_submitter))
        .route("/submitters", web::post().to(get_submitters))
        .route("/submittees", web::post().to(get_submittees))
        .default_service(web::get().to(default))
}
default_paths_fn!(
    "/updbio",
    "/updalias",
    "/remsubmitter",
    "/addsubmitter",
    "/submitters",
    "/submittees"
);

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateData {
    data: String,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

async fn update_data(
    data: web::Json<UpdateData>,
    callback: impl AsyncFn(
        &mut sqlx::PgConnection,
        i32,
        &str,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    if !is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await?
    {
        EveryReturnedError::InvalidSessionToken.into_final_error("");
    }

    let player_id = get_user_data(&data.validation_data.session_token, &mut executor)
        .await?
        .player_id;

    callback(&mut executor, player_id, &data.data).await?;

    close_connection(executor).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

async fn update_player_bio(
    data: web::Json<UpdateData>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    update_data(data, Players::update_player_bio).await
}
async fn update_player_alias(
    data: web::Json<UpdateData>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    update_data(data, Players::update_player_alias).await
}

async fn get_submitters(
    data: web::Json<BareMinimumValidationData>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    if !is_valid_token(&data.session_token, data.user_id, &mut executor).await? {
        return Err(EveryReturnedError::InvalidSessionToken.into_final_error(""));
    }

    let player_id = get_user_data(&data.session_token, &mut executor)
        .await?
        .player_id;

    let data = Players::get_player_submitters(&mut executor, player_id).await?;
    let data = Players::get_player_ids_from_user_ids(&mut executor, &data).await?;

    close_connection(executor).await?;

    send_serialized_data(data)
}

async fn get_submittees(
    data: web::Json<BareMinimumValidationData>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    if !is_valid_token(&data.session_token, data.user_id, &mut executor).await? {
        return Err(EveryReturnedError::InvalidSessionToken.into_final_error(""));
    }

    let data = match is_user_admin(data.user_id, &mut executor).await? {
        true => Players::get_ids_but_list(&mut executor, &[]).await?,
        false => Players::get_submittees(&mut executor, data.user_id).await?,
    };

    close_connection(executor).await?;

    send_serialized_data(data)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitterAddRemove {
    player_id: i32,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

enum SubmitterListAction {
    Add,
    Remove,
}

async fn add_submitter(
    data: web::Json<SubmitterAddRemove>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    update_submitter_list(data, SubmitterListAction::Add).await
}

async fn remove_submitter(
    data: web::Json<SubmitterAddRemove>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    update_submitter_list(data, SubmitterListAction::Remove).await
}

async fn update_submitter_list(
    data: web::Json<SubmitterAddRemove>,
    action: SubmitterListAction,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    if !is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await?
    {
        return Err(EveryReturnedError::InvalidSessionToken.into_final_error(""));
    }

    let player_id = get_user_data(&data.validation_data.session_token, &mut executor)
        .await?
        .player_id;
    let associated_user_id = get_user_id_from_player_id(data.player_id, &mut executor).await?;
    let mut submitters_list = Players::get_player_submitters(&mut executor, player_id).await?;

    if !submitters_list.contains(&associated_user_id) {
        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body("{}"));
    }

    match action {
        SubmitterListAction::Add => submitters_list.push(associated_user_id),
        SubmitterListAction::Remove => submitters_list.retain(|x| *x != associated_user_id),
    }

    Players::update_player_submitters(&mut executor, player_id, &submitters_list).await?;

    close_connection(executor).await?;

    send_serialized_data(submitters_list)
}
