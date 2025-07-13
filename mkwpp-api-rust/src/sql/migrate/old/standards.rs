use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Standards {
    level: i32,
    track: i32,
    category: u8,
    is_lap: bool,
    value: Option<i32>,
}

impl super::OldFixtureJson for Standards {
    const FILENAME: &str = "standards.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::standards::Standards {
            id: key,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            standard_level_id: self.level,
            track_id: self.track,
            is_lap: self.is_lap,
            value: self.value,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::standards::Standards {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO standards (id, standard_level_id, track_id, category, is_lap, value) VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET standard_level_id = $2, track_id = $3, category = $4, is_lap = $5, value = $6 WHERE standards.id = $1;").bind(self.id).bind(self.standard_level_id).bind(self.track_id).bind(self.category).bind(self.is_lap).bind(self.value).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
