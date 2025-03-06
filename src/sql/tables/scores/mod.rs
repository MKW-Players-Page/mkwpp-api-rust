pub mod rankings;
pub mod timesheet;
pub mod with_player;

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Scores {
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
    pub initial_rank: Option<i32>,
}

impl super::BasicTableQueries for Scores {
    const TABLE_NAME: &'static str = "scores";
}

impl Scores {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO scores (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);").bind(self.id).bind(self.value).bind(&self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(self.initial_rank).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO scores (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (id) DO UPDATE SET value = $2, category = $3, is_lap = $4, player_id = $5, track_id = $6, date = $7, video_link = $8, ghost_link = $9, comment = $10, admin_note = $11, initial_rank = $12 WHERE scores.id = $1;").bind(self.id).bind(self.value).bind(self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(self.initial_rank).execute(executor).await;
    }
}
