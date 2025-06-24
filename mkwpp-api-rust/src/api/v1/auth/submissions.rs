use actix_web::{HttpResponse, dev::HttpServiceFactory, web};
use sqlx::{
    FromRow,
    postgres::{PgQueryResult, PgRow},
};

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
        scores::Scores,
        submissions::{Submissions, edit_submissions::EditSubmissions},
    },
};

pub fn submissions() -> impl HttpServiceFactory {
    web::scope("/submissions")
        .route(
            "/create_submission",
            web::post().to(create_or_edit_submission),
        )
        .route("/delete_submission", web::post().to(delete_submission))
        .route("/get_submissions", web::post().to(get_submissions))
        .route(
            "/create_edit_submission",
            web::post().to(create_or_edit_edit_submission),
        )
        .route(
            "/delete_edit_submission",
            web::post().to(delete_edit_submission),
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

trait Convertible {
    fn set_submitter_id(&mut self, new_id: i32);
    fn set_reviewer_id(&mut self, new_id: i32);
    fn get_submitter_id(&self) -> i32;
    fn get_reviewer_id(&self) -> Option<i32>;
}

impl Convertible for Submissions {
    fn get_reviewer_id(&self) -> Option<i32> {
        self.reviewer_id
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
    fn set_reviewer_id(&mut self, new_id: i32) {
        self.reviewer_id = Some(new_id);
    }
    fn set_submitter_id(&mut self, new_id: i32) {
        self.submitter_id = new_id;
    }
}

impl Convertible for EditSubmissions {
    fn get_reviewer_id(&self) -> Option<i32> {
        self.reviewer_id
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
    fn set_reviewer_id(&mut self, new_id: i32) {
        self.reviewer_id = Some(new_id);
    }
    fn set_submitter_id(&mut self, new_id: i32) {
        self.submitter_id = new_id;
    }
}

async fn get<
    T: serde::Serialize + for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> + Convertible,
>(
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

    let mut data = match decode_rows_to_table::<T>(data) {
        Err(e) => return e,
        Ok(v) => v,
    };

    for item in data.iter_mut() {
        match Players::get_player_ids_from_user_ids(&mut executor, &[item.get_submitter_id()]).await
        {
            Ok(v) => item.set_submitter_id(v[0]),
            Err(e) => {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Error converting Submitter IDs to Player IDs"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
            }
        }
        if let Some(reviewer_id) = item.get_reviewer_id() {
            match Players::get_player_ids_from_user_ids(&mut executor, &[reviewer_id]).await {
                Ok(v) => item.set_reviewer_id(v[0]),
                Err(e) => {
                    return FinalErrorResponse::new_no_fields(vec![
                        String::from("Error converting Reviewer IDs to Player IDs"),
                        e.to_string(),
                    ])
                    .generate_response(HttpResponse::InternalServerError);
                }
            }
        }
    }

    if let Err(e) = close_connection(executor).await {
        return e;
    }

    send_serialized_data(data)
}

#[derive(serde::Deserialize)]
struct DeletionData {
    data: i32,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

#[derive(serde::Deserialize)]
struct Data<T> {
    #[serde(flatten)]
    data: T,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

trait Deletable {
    async fn get_player_id(&self) -> Result<i32, HttpResponse>;
    fn get_submitter_id(&self) -> i32;
}

impl Deletable for Submissions {
    async fn get_player_id(&self) -> Result<i32, HttpResponse> {
        Ok(self.player_id)
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
}

impl Deletable for EditSubmissions {
    async fn get_player_id(&self) -> Result<i32, HttpResponse> {
        let app_state = access_app_state().await;
        let mut executor = {
            let app_state = app_state.read().await;
            app_state
                .acquire_pg_connection()
                .await
                .map_err(AppState::pg_conn_http_error)?
        };

        let row = Scores::get_from_id(self.score_id, &mut executor)
            .await
            .map_err(|e| {
                FinalErrorResponse::new_no_fields(vec![
                    String::from("Error validating session token"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::BadRequest)
            })?;

        decode_row_to_table::<Scores>(row).map(|s| s.player_id)
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
}

async fn delete_submission(data: web::Json<DeletionData>) -> HttpResponse {
    delete::<Submissions>(
        data,
        Submissions::get_submission_by_id,
        Submissions::delete_submission_by_id,
    )
    .await
}

async fn delete_edit_submission(data: web::Json<DeletionData>) -> HttpResponse {
    delete::<EditSubmissions>(
        data,
        EditSubmissions::get_edit_submission_by_id,
        EditSubmissions::delete_edit_submission_by_id,
    )
    .await
}

async fn delete<X: Deletable + for<'a> FromRow<'a, PgRow>>(
    data: web::Json<DeletionData>,
    get_callback: impl AsyncFn(i32, &mut sqlx::PgConnection) -> Result<PgRow, sqlx::Error>,
    delete_callback: impl AsyncFn(i32, &mut sqlx::PgConnection) -> Result<PgQueryResult, sqlx::Error>,
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

    let submission = match get_callback(data.data, &mut executor).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting data from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let submission = match decode_row_to_table::<X>(submission) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let player_id = match submission.get_player_id().await {
        Ok(v) => v,
        Err(e) => return e,
    };

    let can_delete = match (
        is_user_admin(data.validation_data.user_id, &mut executor).await,
        get_user_data(&data.validation_data.session_token, &mut executor).await,
        Players::get_player_submitters(&mut executor, player_id).await,
    ) {
        (Ok(true), _, _) => true,
        (_, Ok(Some(Ok(user_data))), _) if user_data.player_id == player_id => true,
        (_, _, Ok(v))
            if submission.get_submitter_id() == data.validation_data.user_id
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

    if let Err(e) = delete_callback(data.data, &mut executor).await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Error deleting"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSubmissionCreation {
    pub edit_submission_id: Option<i32>,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub date_edited: bool,
    pub video_link_edited: bool,
    pub ghost_link_edited: bool,
    pub comment_edited: bool,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
    pub score_id: i32,
}

async fn create_or_edit_edit_submission(
    data: web::Json<Data<EditSubmissionCreation>>,
) -> HttpResponse {
    let mut data = data.0;

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

    let score = match Scores::get_from_id(data.data.score_id, &mut executor).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Couldn't get score"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let score = match decode_row_to_table::<Scores>(score) {
        Ok(v) => v,
        Err(e) => return e,
    };

    data.data.comment_edited = data.data.comment != score.comment;
    data.data.video_link_edited = data.data.video_link != score.video_link;
    data.data.ghost_link_edited = data.data.ghost_link != score.ghost_link;
    data.data.date_edited = data.data.date != score.date;

    if data.data.comment_edited == false
        && data.data.video_link_edited == false
        && data.data.ghost_link_edited == false
        && data.data.date_edited == false
        && data.data.edit_submission_id.is_none()
    {
        return FinalErrorResponse::new_no_fields(vec![String::from("No data to modify!")])
            .generate_response(HttpResponse::BadRequest);
    }

    let can_submit = match (
        is_user_admin(data.validation_data.user_id, &mut executor).await,
        get_user_data(&data.validation_data.session_token, &mut executor).await,
        Players::get_player_submitters(&mut executor, score.player_id).await,
    ) {
        (Ok(true), _, _) => true,
        (_, Ok(Some(Ok(user_data))), _) if user_data.player_id == score.player_id => true,
        (_, _, Ok(v)) if v.contains(&data.data.submitter_id) => true,
        _ => false,
    };

    if !can_submit {
        return FinalErrorResponse::new_no_fields(vec![String::from("Player can't submit!")])
            .generate_response(HttpResponse::Forbidden);
    }

    if let Err(e) = EditSubmissions::create_or_edit_submission(data.data, &mut executor).await {
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
