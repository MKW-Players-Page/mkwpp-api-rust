use actix_web::{HttpResponse, dev::HttpServiceFactory, web};
use sqlx::postgres::PgRow;

use crate::{
    api::{
        FinalErrorResponse,
        v1::{close_connection, decode_row_to_table, decode_rows_to_table, send_serialized_data},
    },
    app_state::{AppState, access_app_state},
    auth::{BareMinimumValidationData, get_user_data, is_user_admin, is_valid_token},
    sql::tables::{
        Category,
        players::Players,
        submissions::{Submissions, edit_submissions::EditSubmissions},
    },
};

pub fn submissions() -> impl HttpServiceFactory {
    web::scope("/submissions")
        .route(
            "/create_submission",
            web::post().to(create_or_edit_submission),
        )
        .route("/delete_submission", web::post().to(get_submissions))
        .route("/get_submissions", web::post().to(get_submissions))
        .route(
            "/create_edit_submission",
            web::post().to(get_edit_submissions),
        )
        .route(
            "/delete_edit_submission",
            web::post().to(get_edit_submissions),
        )
        .route(
            "/get_edit_submissions",
            web::post().to(get_edit_submissions),
        )
        .default_service(web::get().to(default))
}

default_paths_fn!("/get_submissions", "/get_edit_submissions");

async fn get_submissions(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    get::<Submissions>(data, Submissions::get_user_submissions).await
}
async fn get_edit_submissions(data: web::Json<BareMinimumValidationData>) -> HttpResponse {
    get::<EditSubmissions>(data, EditSubmissions::get_user_edit_submissions).await
}

async fn get<T: serde::Serialize + for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>>(
    data: web::Json<BareMinimumValidationData>,
    callback: impl AsyncFn(i32, i32, &mut sqlx::PgConnection) -> Result<Vec<PgRow>, sqlx::Error>,
) -> HttpResponse {
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

    let data = match callback(data.user_id, player_id, &mut executor).await {
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

    let data = match decode_rows_to_table::<T>(data) {
        Err(e) => return e,
        Ok(v) => v,
    };

    send_serialized_data(data)
}

#[derive(serde::Deserialize)]
pub struct SubmissionCreation {
    pub submission_id: Option<i32>,
    pub value: i32,
    pub category: Category,
    pub is_lap: bool,
    pub player_id: i32,
    pub track_id: i32,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
}

#[derive(serde::Deserialize)]
struct Data<T> {
    data: T,
    validation_data: BareMinimumValidationData,
}

async fn create_or_edit_submission(data: web::Json<Data<SubmissionCreation>>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    if let Ok(false) | Err(_) = is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await
    {
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "Error validating session token",
        )])
        .generate_response(HttpResponse::BadRequest);
    }

    if data.data.submitter_id != data.validation_data.user_id {
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "The data is invalid submitter_id != user_id!",
        )])
        .generate_response(HttpResponse::BadRequest);
    }

    let can_submit = match (
        is_user_admin(data.validation_data.user_id, &mut executor).await,
        get_user_data(&data.validation_data.session_token, &mut executor).await,
        Players::get_player_submitters(&mut executor, data.data.player_id).await,
    ) {
        (Ok(true), _, _) => true,
        (_, Ok(Some(Ok(user_data))), _) if user_data.player_id == data.data.player_id => true,
        (_, _, Ok(v)) if v.contains(&data.data.submitter_id) => true,
        _ => false,
    };

    if !can_submit {
        return FinalErrorResponse::new_no_fields(vec![String::from("Player can't submit!")])
            .generate_response(HttpResponse::InternalServerError);
    }

    if let Err(e) = Submissions::create_or_edit_submission(data.data, &mut executor).await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Error creating submission"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

async fn delete_submission(data: web::Json<Data<i32>>) -> HttpResponse {
    let data = data.0;

    let app_state = access_app_state().await;
    let mut executor = {
        let app_state = app_state.read().await;
        match app_state.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    if let Ok(false) | Err(_) = is_valid_token(
        &data.validation_data.session_token,
        data.validation_data.user_id,
        &mut executor,
    )
    .await
    {
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "Error validating session token",
        )])
        .generate_response(HttpResponse::BadRequest);
    }

    let submission = match Submissions::get_submission_by_id(data.data, &mut executor).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting submission from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let submission = match decode_row_to_table::<Submissions>(submission) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let can_delete = match (
        is_user_admin(data.validation_data.user_id, &mut executor).await,
        get_user_data(&data.validation_data.session_token, &mut executor).await,
        Players::get_player_submitters(&mut executor, submission.player_id).await,
    ) {
        (Ok(true), _, _) => true,
        (_, Ok(Some(Ok(user_data))), _) if user_data.player_id == submission.player_id => true,
        (_, _, Ok(v))
            if submission.submitter_id == data.validation_data.user_id
                && v.contains(&data.validation_data.user_id) =>
        {
            true
        }
        _ => false,
    };

    if !can_delete {
        return FinalErrorResponse::new_no_fields(vec![String::from("Insufficient permissions")])
            .generate_response(HttpResponse::Forbidden);
    }

    if let Err(e) = Submissions::delete_submission_by_id(data.data, &mut executor).await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Error deleting submission"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}
