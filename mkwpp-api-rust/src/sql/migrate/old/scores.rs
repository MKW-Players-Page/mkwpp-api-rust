use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Scores {
    value: i32,
    category: u8,
    is_lap: bool,
    player: i32,
    track: i32,
    date: Option<String>,
    video_link: Option<String>,
    ghost_link: Option<String>,
    comment: Option<String>,
    admin_note: Option<String>,
    initial_rank: Option<i32>,
}

impl super::OldFixtureJson for Scores {
    const FILENAME: &str = "scores.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::scores::Scores {
            id: key,
            value: self.value,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            is_lap: self.is_lap,
            player_id: self.player,
            track_id: self.track,
            date: self
                .date
                .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap()),
            video_link: self.video_link,
            ghost_link: self.ghost_link,
            comment: self.comment,
            admin_note: self.admin_note,
            initial_rank: self.initial_rank,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::scores::Scores {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO scores (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (id) DO UPDATE SET value = $2, category = $3, is_lap = $4, player_id = $5, track_id = $6, date = $7, video_link = $8, ghost_link = $9, comment = $10, admin_note = $11, initial_rank = $12 WHERE scores.id = $1;").bind(self.id).bind(self.value).bind(self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(self.initial_rank).execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
