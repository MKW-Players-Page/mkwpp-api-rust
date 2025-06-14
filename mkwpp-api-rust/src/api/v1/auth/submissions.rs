use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        FinalErrorResponse,
        v1::{close_connection, decode_rows_to_table, send_serialized_data},
    },
    app_state::{AppState, access_app_state},
    auth::{BareMinimumValidationData, get_user_data, is_valid_token},
    sql::tables::{edit_submissions::EditSubmissions, submissions::Submissions},
};

pub fn submissions() -> impl HttpServiceFactory {
    web::scope("/submissions")
        .route("/get_submissions", web::post().to(get_submissions))
        .route(
            "/get_edit_submissions",
            web::post().to(get_edit_submissions),
        )
        .default_service(web::get().to(default))
}

default_paths_fn!("/get_submissions", "/get_edit_submissions");

async fn get_submissions(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    if let Ok(false) | Err(_) =
        is_valid_token(&data.session_token, data.user_id, &mut executor).await
    {
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "Error validating session token",
        )])
        .generate_response(HttpResponse::BadRequest);
    }

    let player_id = match get_user_data(&data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => {
                    return FinalErrorResponse::new_no_fields(vec![
                        String::from("Error decoding Database Data"),
                        e.to_string(),
                    ])
                    .generate_response(HttpResponse::InternalServerError);
                }
            },
            None => {
                return FinalErrorResponse::new_no_fields(vec![String::from(
                    "User has no associated Player",
                )])
                .generate_response(HttpResponse::InternalServerError);
            }
        },
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Database Error"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let data = match Submissions::get_user_submissions(data.user_id, player_id, &mut executor).await
    {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting user submissions"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    let data = match decode_rows_to_table::<Submissions>(data) {
        Err(e) => return e,
        Ok(v) => v,
    };

    send_serialized_data(data)
}

async fn get_edit_submissions(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    if let Ok(false) | Err(_) =
        is_valid_token(&data.session_token, data.user_id, &mut executor).await
    {
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "Error validating session token",
        )])
        .generate_response(HttpResponse::BadRequest);
    }

    let player_id = match get_user_data(&data.session_token, &mut executor).await {
        Ok(v) => match v {
            Some(v) => match v {
                Ok(v) => v.player_id,
                Err(e) => {
                    return FinalErrorResponse::new_no_fields(vec![
                        String::from("Error decoding Database Data"),
                        e.to_string(),
                    ])
                    .generate_response(HttpResponse::InternalServerError);
                }
            },
            None => {
                return FinalErrorResponse::new_no_fields(vec![String::from(
                    "User has no associated Player",
                )])
                .generate_response(HttpResponse::InternalServerError);
            }
        },
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Database Error"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let data =
        match EditSubmissions::get_user_edit_submissions(data.user_id, player_id, &mut executor)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Error getting user edit submissions"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
            }
        };

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    let data = match decode_rows_to_table::<EditSubmissions>(data) {
        Err(e) => return e,
        Ok(v) => v,
    };

    send_serialized_data(data)
}
