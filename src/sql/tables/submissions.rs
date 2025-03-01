#[derive(serde::Deserialize, serde::Serialize, Debug, sqlx::FromRow)]
pub struct Submissions {
    pub id: i32,
    pub value: i32,
    pub category: super::Category,
    pub is_lap: bool,
    pub player_id: i32,
    pub track_id: i32,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
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
        return sqlx::query("INSERT INTO submissions (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, status, submitter_id, submitter_note, submitted_at, reviewer_id, reviewer_note, reviewed_at, score_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19) ON CONFLICT (id) DO UPDATE SET value = $2, category = $3, is_lap = $4, player_id = $5, track_id = $6, date = $7, video_link = $8, ghost_link = $9, comment = $10, admin_note = $11, status = $12, submitter_id = $13, submitter_note = $14, submitted_at = $15, reviewer_id = $16, reviewer_note = $17, reviewed_at = $18, score_id = $19 WHERE submissions.id = $1;").bind(self.id).bind(self.value).bind(self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(&self.status).bind(self.submitter_id).bind(&self.submitter_note).bind(self.submitted_at).bind(self.reviewer_id).bind(&self.reviewer_note).bind(self.reviewed_at).bind(self.score_id).execute(executor).await;
    }
}
