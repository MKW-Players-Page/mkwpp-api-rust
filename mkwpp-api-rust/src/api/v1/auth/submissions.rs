use actix_web::{HttpResponse, dev::HttpServiceFactory, web};
use sqlx::{
    FromRow,
    postgres::{PgQueryResult, PgRow},
};

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::{close_connection, decode_row_to_table, decode_rows_to_table, send_serialized_data},
    },
    app_state::access_app_state,
    auth::{BareMinimumValidationData, get_user_data, is_user_admin, is_valid_token},
    custom_serde::DateAsTimestampNumber,
    sql::tables::{
        Category,
        players::Players,
        scores::Scores,
        submissions::{SubmissionStatus, Submissions, edit_submissions::EditSubmissions},
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

async fn get_submissions(
    data: web::Json<BareMinimumValidationData>,
) -> Result<HttpResponse, FinalErrorResponse> {
    get::<Submissions>(data, Submissions::get_user_submissions).await
}
async fn get_edit_submissions(
    data: web::Json<BareMinimumValidationData>,
) -> Result<HttpResponse, FinalErrorResponse> {
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
    callback: impl AsyncFn(i32, i32, &mut sqlx::PgConnection) -> Result<Vec<PgRow>, FinalErrorResponse>,
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

    let player_id = match get_user_data(&data.session_token, &mut executor)
        .await?
        .player_id
    {
        Some(v) => v,
        None => return Err(EveryReturnedError::NoAssociatedPlayer.into_final_error("")),
    };

    let mut data =
        decode_rows_to_table::<T>(callback(data.user_id, player_id, &mut executor).await?)?;

    for item in data.iter_mut() {
        item.set_submitter_id(
            Players::get_player_id_from_user_id(&mut executor, item.get_submitter_id()).await?,
        );
        if let Some(reviewer_id) = item.get_reviewer_id() {
            item.set_reviewer_id(
                Players::get_player_id_from_user_id(&mut executor, reviewer_id).await?,
            );
        }
    }

    close_connection(executor).await?;

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
    async fn get_player_id(&self) -> Result<i32, FinalErrorResponse>;
    fn get_submitter_id(&self) -> i32;
}

impl Deletable for Submissions {
    async fn get_player_id(&self) -> Result<i32, FinalErrorResponse> {
        Ok(self.player_id)
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
}

impl Deletable for EditSubmissions {
    async fn get_player_id(&self) -> Result<i32, FinalErrorResponse> {
        let app_state = access_app_state().await;
        let mut executor = {
            let app_state = app_state.read().await;
            app_state.acquire_pg_connection().await?
        };

        decode_row_to_table::<Scores>(Scores::get_from_id(self.score_id, &mut executor).await?)
            .map(|s| s.player_id)
    }
    fn get_submitter_id(&self) -> i32 {
        self.submitter_id
    }
}

async fn delete_submission(
    data: web::Json<DeletionData>,
) -> Result<HttpResponse, FinalErrorResponse> {
    delete::<Submissions>(
        data,
        Submissions::get_submission_by_id,
        Submissions::delete_submission_by_id,
    )
    .await
}

async fn delete_edit_submission(
    data: web::Json<DeletionData>,
) -> Result<HttpResponse, FinalErrorResponse> {
    delete::<EditSubmissions>(
        data,
        EditSubmissions::get_edit_submission_by_id,
        EditSubmissions::delete_edit_submission_by_id,
    )
    .await
}

async fn delete<X: Deletable + for<'a> FromRow<'a, PgRow>>(
    data: web::Json<DeletionData>,
    get_callback: impl AsyncFn(i32, &mut sqlx::PgConnection) -> Result<PgRow, FinalErrorResponse>,
    delete_callback: impl AsyncFn(
        i32,
        &mut sqlx::PgConnection,
    ) -> Result<PgQueryResult, FinalErrorResponse>,
) -> Result<HttpResponse, FinalErrorResponse> {
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

    let submission = decode_row_to_table::<X>(get_callback(data.data, &mut executor).await?)?;
    let player_id = submission.get_player_id().await?;

    let can_delete = match (
        is_user_admin(data.validation_data.user_id, &mut executor).await?,
        get_user_data(&data.validation_data.session_token, &mut executor)
            .await?
            .player_id,
        Players::get_player_submitters(&mut executor, player_id).await?,
    ) {
        (true, _, _) => true,
        (_, Some(user_player_id), _) if user_player_id == player_id => true,
        (_, _, v)
            if submission.get_submitter_id() == data.validation_data.user_id
                && v.contains(&data.validation_data.user_id) =>
        {
            true
        }
        _ => false,
    };

    if !can_delete {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    delete_callback(data.data, &mut executor).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionCreation {
    pub submission_id: Option<i32>,
    pub value: i32,
    pub category: Category,
    pub is_lap: bool,
    pub player_id: i32,
    pub track_id: i32,
    #[serde(deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp")]
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
    pub admin_note: Option<String>,
    pub reviewer_note: Option<String>,
    pub status: Option<SubmissionStatus>,
    pub reviewer_id: Option<i32>,
}

async fn create_or_edit_submission(
    data: web::Json<Data<SubmissionCreation>>,
) -> Result<HttpResponse, FinalErrorResponse> {
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

    if data.data.submitter_id != data.validation_data.user_id {
        return Err(EveryReturnedError::MismatchedIds.into_final_error(""));
    }

    let is_admin = is_user_admin(data.validation_data.user_id, &mut executor).await?;
    let can_submit = match (
        is_admin,
        get_user_data(&data.validation_data.session_token, &mut executor)
            .await?
            .player_id,
        Players::get_player_submitters(&mut executor, data.data.player_id).await?,
    ) {
        (true, _, _) => true,
        (_, Some(user_player_id), _) if user_player_id == data.data.player_id => true,
        (_, _, v) if v.contains(&data.data.submitter_id) => true,
        _ => false,
    };

    if !can_submit {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    Submissions::create_or_edit_submission(data.data.clone(), is_admin, &mut executor).await?;

    if is_admin
        && let Some(status) = data.data.status
        && status == SubmissionStatus::Accepted
    {
        Scores::insert_or_edit(
            None,
            data.data.value,
            data.data.category,
            data.data.is_lap,
            data.data.player_id,
            data.data.track_id,
            data.data.date,
            data.data.video_link,
            data.data.ghost_link,
            data.data.comment,
            None,
            None,
            &mut executor,
        )
        .await?;
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditSubmissionCreation {
    pub edit_submission_id: Option<i32>,
    #[serde(deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp")]
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
    pub admin_note: Option<String>,
    pub reviewer_note: Option<String>,
    pub status: Option<SubmissionStatus>,
    pub reviewer_id: Option<i32>,
}

async fn create_or_edit_edit_submission(
    data: web::Json<Data<EditSubmissionCreation>>,
) -> Result<HttpResponse, FinalErrorResponse> {
    let mut data = data.0;

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

    if data.data.submitter_id != data.validation_data.user_id {
        return Err(EveryReturnedError::MismatchedIds.into_final_error(""));
    }

    let score = decode_row_to_table::<Scores>(
        Scores::get_from_id(data.data.score_id, &mut executor).await?,
    )?;

    data.data.comment_edited = data.data.comment != score.comment;
    data.data.video_link_edited = data.data.video_link != score.video_link;
    data.data.ghost_link_edited = data.data.ghost_link != score.ghost_link;
    data.data.date_edited = data.data.date != score.date;

    if !data.data.comment_edited
        && !data.data.video_link_edited
        && !data.data.ghost_link_edited
        && !data.data.date_edited
        && data.data.edit_submission_id.is_none()
    {
        return Err(EveryReturnedError::NothingChanged.into_final_error(""));
    }

    let is_admin = is_user_admin(data.validation_data.user_id, &mut executor).await?;

    let can_submit = match (
        is_admin,
        get_user_data(&data.validation_data.session_token, &mut executor)
            .await?
            .player_id,
        Players::get_player_submitters(&mut executor, score.player_id).await?,
    ) {
        (true, _, _) => true,
        (_, Some(user_player_id), _) if user_player_id == score.player_id => true,
        (_, _, v) if v.contains(&data.data.submitter_id) => true,
        _ => false,
    };

    if !can_submit {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    EditSubmissions::create_or_edit_submission(data.data.clone(), is_admin, &mut executor).await?;

    if is_admin
        && let Some(status) = data.data.status
        && status == SubmissionStatus::Accepted
    {
        let score = decode_row_to_table::<Scores>(
            Scores::get_from_id(data.data.score_id, &mut executor).await?,
        )?;
        Scores::insert_or_edit(
            Some(data.data.score_id),
            score.value,
            score.category,
            score.is_lap,
            score.player_id,
            score.track_id,
            data.data.date,
            data.data.video_link,
            data.data.ghost_link,
            data.data.comment,
            score.admin_note,
            score.initial_rank,
            &mut executor,
        )
        .await?;
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}
