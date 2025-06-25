pub mod edit_submissions;

use sqlx::postgres::PgQueryResult;

use crate::api::v1::auth::submissions::SubmissionCreation;
use crate::custom_serde::DateAsTimestampNumber;
use crate::sql::tables::BasicTableQueries;

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "submission_status", rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Accepted,
    Rejected,
    OnHold,
}

impl TryFrom<u8> for SubmissionStatus {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Accepted),
            2 => Ok(Self::Rejected),
            3 => Ok(Self::OnHold),
            _ => Err(()),
        }
    }
}

impl From<SubmissionStatus> for u8 {
    fn from(value: SubmissionStatus) -> Self {
        match value {
            SubmissionStatus::Pending => 0,
            SubmissionStatus::Accepted => 1,
            SubmissionStatus::Rejected => 2,
            SubmissionStatus::OnHold => 3,
        }
    }
}

impl From<&SubmissionStatus> for u8 {
    fn from(value: &SubmissionStatus) -> Self {
        match value {
            SubmissionStatus::Pending => 0,
            SubmissionStatus::Accepted => 1,
            SubmissionStatus::Rejected => 2,
            SubmissionStatus::OnHold => 3,
        }
    }
}

impl Default for SubmissionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl serde::Serialize for SubmissionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.into())
    }
}

impl<'de> serde::Deserialize<'de> for SubmissionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde_json::Number::deserialize(deserializer).and_then(|x| {
            x.as_u64()
                .ok_or_else(|| {
                    serde::de::Error::invalid_type(
                        if x.is_f64() {
                            serde::de::Unexpected::Float(x.as_f64().unwrap())
                        } else if x.is_i64() {
                            serde::de::Unexpected::Signed(x.as_i64().unwrap())
                        } else {
                            serde::de::Unexpected::Other("integer")
                        },
                        &"u8 < 3",
                    )
                })
                .and_then(|x| {
                    SubmissionStatus::try_from(x as u8).map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(x),
                            &"u8 < 3",
                        )
                    })
                })
        })
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Submissions {
    pub id: i32,
    pub value: i32,
    pub category: super::Category,
    pub is_lap: bool,
    pub player_id: i32,
    pub track_id: i32,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub admin_note: Option<String>,
    pub status: SubmissionStatus,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub reviewer_id: Option<i32>,
    pub reviewer_note: Option<String>,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub score_id: Option<i32>,
}

impl super::BasicTableQueries for Submissions {
    const TABLE_NAME: &'static str = "submissions";
}

impl Submissions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO submissions (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19);").bind(self.id).bind(self.value).bind(&self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query(r#"INSERT INTO submissions (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19) ON CONFLICT (id) DO UPDATE SET value = $2, category = $3, is_lap = $4, player_id = $5, track_id = $6, date = $7, video_link = $8, ghost_link = $9, comment = $10, admin_note = $11, status = $12, submitter_id = $13, submitter_note = $14, submitted_at = $15, reviewer_id = $16, reviewer_note = $17, reviewed_at = $18, score_id = $19 WHERE submissions.id = $1;"#).bind(self.id).bind(self.value).bind(self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await;
    }

    pub async fn create_or_edit_submission(
        data: SubmissionCreation,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        match data.submission_id {
            None => sqlx::query(
                r#"
                INSERT INTO
                    submissions 
                (
                    value, category, is_lap,
                    player_id, track_id, date,
                    video_link, ghost_link, comment,
                    submitter_id, submitter_note
                ) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);
                "#,
            ),
            Some(id) => sqlx::query(
                r#"
                UPDATE
                    submissions 
                SET
                    value = $2, category = $3,
                    is_lap = $4, player_id = $5,
                    track_id = $6, date = $7,
                    video_link = $8, ghost_link = $9,
                    comment = $10, submitter_note = $12
                WHERE id = $1
                "#,
            )
            .bind(id),
        }
        .bind(data.value)
        .bind(data.category)
        .bind(data.is_lap)
        .bind(data.player_id)
        .bind(data.track_id)
        .bind(data.date)
        .bind(&data.video_link)
        .bind(&data.ghost_link)
        .bind(&data.comment)
        .bind(data.submitter_id)
        .bind(&data.submitter_note)
        .execute(executor)
        .await
    }

    pub async fn get_submission_by_id(
        id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgRow, sqlx::Error> {
        return sqlx::query(const_format::formatc!(
            "SELECT * FROM {} WHERE id = $1",
            Submissions::TABLE_NAME
        ))
        .bind(id)
        .fetch_one(executor)
        .await;
    }

    pub async fn get_user_submissions(
        user_id: i32,
        player_id: i32, // Associated Player ID
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(const_format::formatc!(
            "SELECT * FROM {} WHERE submitter_id = $1 OR player_id = $2",
            Submissions::TABLE_NAME
        ))
        .bind(user_id)
        .bind(player_id)
        .fetch_all(executor)
        .await;
    }

    pub async fn delete_submission_by_id(
        submission_id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query(const_format::formatc!(
            "DELETE FROM {} WHERE id = $1;",
            Submissions::TABLE_NAME
        ))
        .bind(submission_id)
        .execute(executor)
        .await
    }
}
