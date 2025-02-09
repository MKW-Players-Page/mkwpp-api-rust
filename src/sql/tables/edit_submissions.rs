#[derive(serde::Deserialize, Debug)]
pub struct EditSubmissions {
    pub id: i32,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub video_link_edited: bool,
    pub ghost_link_edited: bool,
    pub comment_edited: bool,
    pub admin_note: Option<String>,
    pub status: super::SubmissionStatus,
    pub submitter_id: i32,
    pub submitter_note: Option<String>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub reviewer_id: i32,
    pub reviewer_note: Option<String>,
    pub reviewed_at: chrono::DateTime<chrono::Utc>,
    pub score_id: Option<i32>,
}

impl EditSubmissions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO edit_submissions (id, video_link_edited, ghost_link_edited, comment_edited, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16);").bind(self.id).bind(self.video_link_edited).bind(&self.ghost_link_edited).bind(self.comment_edited).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await
    // }

    pub async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("SELECT * FROM edit_submissions;")
            .execute(executor)
            .await;
    }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO edit_submissions (id, video_link_edited, ghost_link_edited, comment_edited, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) ON CONFLICT (id) DO UPDATE SET video_link_edited = $2, ghost_link_edited = $3, comment_edited = $4, video_link = $5, ghost_link = $6, comment = $7, admin_note = $8, status = $9, submitter_id = $10, submitter_note = $11, submitted_at = $12, reviewer_id = $13, reviewer_note = $14, reviewed_at = $15, score_id = $16 WHERE edit_submissions.id = $1;").bind(self.id).bind(self.video_link_edited).bind(&self.ghost_link_edited).bind(self.comment_edited).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await;
    }
}
