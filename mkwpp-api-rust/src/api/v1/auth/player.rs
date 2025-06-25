use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::EveryReturnedError,
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
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    if let Ok(false) | Err(_) = is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await
    {
        EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let player_id = match get_user_data(&data.validation_data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
            },
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let data = match callback(&mut executor, player_id, &data.data).await {
        Ok(v) => v.rows_affected().to_string(),
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body(data)
}

async fn update_player_bio(data: web::Json<UpdateData>) -> HttpResponse {
    update_data(data, Players::update_player_bio).await
}
async fn update_player_alias(data: web::Json<UpdateData>) -> HttpResponse {
    update_data(data, Players::update_player_alias).await
}

async fn get_submitters(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    if let Ok(false) | Err(_) =
        is_valid_token(&data.session_token, data.user_id, &mut executor).await
    {
        return EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let player_id = match get_user_data(&data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
            },
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let data = match Players::get_player_submitters(&mut executor, player_id).await {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let data = match Players::get_player_ids_from_user_ids(&mut executor, &data).await {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::UserIdToPlayerId.http_response(e),
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    send_serialized_data(data)
}

async fn get_submittees(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    if let Ok(false) | Err(_) =
        is_valid_token(&data.session_token, data.user_id, &mut executor).await
    {
        return EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let data = match match is_user_admin(data.user_id, &mut executor).await {
        Ok(true) => Players::get_ids_but_list(&mut executor, &[]).await,
        _ => Players::get_submittees(&mut executor, data.user_id).await,
    } {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    send_serialized_data(data)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitterAddRemove {
    player_id: i32,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

async fn add_submitter(data: web::Json<SubmitterAddRemove>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    if let Ok(false) | Err(_) = is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await
    {
        return EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let player_id = match get_user_data(&data.validation_data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
            },
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let associated_user_id = match get_user_id_from_player_id(data.player_id, &mut executor).await {
        Ok(v) => match v {
            Some(v) => v,
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
    };

    let mut submitters_list = match Players::get_player_submitters(&mut executor, player_id).await {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    if submitters_list.contains(&associated_user_id) {
        return HttpResponse::Ok().content_type("application/json").body("");
    }

    submitters_list.push(associated_user_id);

    if let Err(e) =
        Players::update_player_submitters(&mut executor, player_id, &submitters_list).await
    {
        return EveryReturnedError::GettingFromDatabase.http_response(e);
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    send_serialized_data(submitters_list)
}

async fn remove_submitter(data: web::Json<SubmitterAddRemove>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    if let Ok(false) | Err(_) = is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await
    {
        return EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let player_id = match get_user_data(&data.validation_data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
            },
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let associated_user_id = match get_user_id_from_player_id(data.player_id, &mut executor).await {
        Ok(v) => match v {
            Some(v) => v,
            None => return EveryReturnedError::UserHasNoAssociatedPlayer.http_response(""),
        },
        Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
    };

    let mut submitters_list = match Players::get_player_submitters(&mut executor, player_id).await {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    if !submitters_list.contains(&associated_user_id) {
        return HttpResponse::Ok().content_type("application/json").body("");
    }

    submitters_list.retain(|x| *x != associated_user_id);

    if let Err(e) =
        Players::update_player_submitters(&mut executor, player_id, &submitters_list).await
    {
        return EveryReturnedError::GettingFromDatabase.http_response(e);
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    send_serialized_data(submitters_list)
}
