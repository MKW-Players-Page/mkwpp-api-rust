use sqlx::postgres::PgQueryResult;

use crate::api::errors::{EveryReturnedError, FinalErrorResponse};
use crate::custom_serde::DateAsTimestampNumber;
use crate::{api::v1::auth::submissions::EditSubmissionCreation, sql::tables::BasicTableQueries};

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow, Default)]
#[serde(rename_all = "camelCase")]
pub struct EditSubmissions {
    pub id: i32,
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub date_edited: bool,
    pub video_link_edited: bool,
    pub ghost_link_edited: bool,
    pub comment_edited: bool,
    pub admin_note: Option<String>,
    pub status: super::SubmissionStatus,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub reviewer_id: Option<i32>,
    pub reviewer_note: Option<String>,
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub score_id: i32,
}

impl BasicTableQueries for EditSubmissions {
    const TABLE_NAME: &'static str = "edit_submissions";
}

impl EditSubmissions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO edit_submissions (id, video_link_edited, ghost_link_edited, comment_edited, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16);").bind(self.id).bind(self.video_link_edited).bind(&self.ghost_link_edited).bind(self.comment_edited).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        // TODO fix missing columns
        return sqlx::query("INSERT INTO edit_submissions (id, video_link_edited, ghost_link_edited, comment_edited, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) ON CONFLICT (id) DO UPDATE SET video_link_edited = $2, ghost_link_edited = $3, comment_edited = $4, video_link = $5, ghost_link = $6, comment = $7, admin_note = $8, status = $9, submitter_id = $10, submitter_note = $11, submitted_at = $12, reviewer_id = $13, reviewer_note = $14, reviewed_at = $15, score_id = $16 WHERE edit_submissions.id = $1;").bind(self.id).bind(self.video_link_edited).bind(self.ghost_link_edited).bind(self.comment_edited).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn create_or_edit_submission(
        data: EditSubmissionCreation,
        add_admin_note: bool,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        match (data.edit_submission_id, add_admin_note, data.reviewer_id) {
            (_, false, Some(_)) => {
                return Err(EveryReturnedError::InsufficientPermissions
                    .into_final_error("reviewer_id cannot be set if you're not a moderator"));
            }
            (None, _, Some(_)) => {
                return Err(EveryReturnedError::InvalidInput
                    .into_final_error("reviewer_id cannot be set on first submission"));
            }
            (None, false, None) => sqlx::query(
                r#"
                    INSERT INTO
                        edit_submissions
                    (
                        submitter_note,
                        submitter_id,
                        score_id,
                        video_link,
                        video_link_edited,
                        ghost_link,
                        ghost_link_edited,
                        comment_edited,
                        comment,
                        date_edited,
                        date
                    ) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);
                "#,
            ),
            (Some(id), false, None) => sqlx::query(
                r#"
                    UPDATE
                        edit_submissions
                    SET
                        submitter_note = $2,
                        submitter_id = $3,
                        score_id = $4,
                        video_link = $5,
                        video_link_edited = $6,
                        ghost_link = $7,
                        ghost_link_edited = $8,
                        comment_edited = $9,
                        comment = $10,
                        date_edited = $11,
                        date = $12,
                    WHERE id = $1
                "#,
            )
            .bind(id),
            (None, true, None) => sqlx::query(
                r#"
                    INSERT INTO
                        edit_submissions
                    (
                        submitter_note,
                        submitter_id,
                        score_id,
                        video_link,
                        video_link_edited,
                        ghost_link,
                        ghost_link_edited,
                        comment_edited,
                        comment,
                        date_edited,
                        date,
                        admin_note
                    ) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);
                "#,
            ),
            (Some(id), true, None) => {
                if data.status.is_none()
                    || data.reviewer_note.is_none()
                    || data.admin_note.is_none()
                {
                    return Err(
                        EveryReturnedError::InvalidInput.into_final_error("Partially missing data")
                    );
                }
                sqlx::query(
                    r#"
                        UPDATE
                            edit_submissions
                        SET
                            submitter_note = $2,
                            submitter_id = $3,
                            score_id = $4,
                            video_link = $5,
                            video_link_edited = $6,
                            ghost_link = $7,
                            ghost_link_edited = $8,
                            comment_edited = $9,
                            comment = $10,
                            date_edited = $11,
                            date = $12,
                            admin_note = $13,
                            reviewer_note = $14,
                            status = $15
                        WHERE id = $1
                    "#,
                )
                .bind(id)
            }
            (Some(id), true, Some(reviewer_id)) => {
                if data.status.is_none()
                    || data.reviewer_note.is_none()
                    || data.admin_note.is_none()
                {
                    return Err(
                        EveryReturnedError::InvalidInput.into_final_error("Partially missing data")
                    );
                }
                sqlx::query(
                    r#"
                        UPDATE
                            edit_submissions
                        SET
                            submitter_note = $3,
                            submitter_id = $4,
                            score_id = $5,
                            video_link = $6,
                            video_link_edited = $7,
                            ghost_link = $8,
                            ghost_link_edited = $9,
                            comment_edited = $10,
                            comment = $11,
                            date_edited = $12,
                            date = $13,
                            admin_note = $14,
                            reviewer_note = $15,
                            status = $16,
                            reviewer_id = $2,
                            reviewed_at = NOW()
                        WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(reviewer_id)
            }
        }
        .bind(data.submitter_note)
        .bind(data.submitter_id)
        .bind(data.score_id)
        .bind(data.video_link)
        .bind(data.video_link_edited)
        .bind(data.ghost_link)
        .bind(data.ghost_link_edited)
        .bind(data.comment_edited)
        .bind(data.comment)
        .bind(data.date_edited)
        .bind(data.date)
        .bind(data.admin_note)
        .bind(data.reviewer_note)
        .bind(data.status)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))
    }

    pub async fn get_edit_submission_by_id(
        id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgRow, FinalErrorResponse> {
        return sqlx::query(const_format::formatc!(
            "SELECT * FROM {} WHERE id = $1",
            EditSubmissions::TABLE_NAME
        ))
        .bind(id)
        .fetch_one(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn get_user_edit_submissions(
        user_id: i32,
        player_id: i32, // Associated Player ID
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(const_format::formatc!(
            "SELECT {0}.* FROM {0} LEFT JOIN scores ON score_id = scores.id WHERE submitter_id = $1 OR player_id = $2",
            EditSubmissions::TABLE_NAME,
        ))
        .bind(user_id)
        .bind(player_id)
        .fetch_all(executor)
        .await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn delete_edit_submission_by_id(
        submission_id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<PgQueryResult, FinalErrorResponse> {
        sqlx::query(const_format::formatc!(
            "DELETE FROM {} WHERE id = $1;",
            EditSubmissions::TABLE_NAME
        ))
        .bind(submission_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))
    }
}
