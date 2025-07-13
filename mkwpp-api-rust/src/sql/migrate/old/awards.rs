use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Awards {
    player: i32,
    date: String,
    description: String,
    #[serde(rename = "type")]
    award_type: String,
}

impl super::OldFixtureJson for Awards {
    const FILENAME: &str = "playerawards.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::awards::Awards {
            id: key,
            player_award_type: crate::sql::tables::awards::AwardType::try_from(
                self.award_type.as_str(),
            )
            .unwrap(),
            date: chrono::NaiveDate::parse_from_str(&self.date, "%F").unwrap(),
            description: self.description,
            player_id: self.player,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::awards::Awards {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO player_awards (id, player_id, date, description, player_award_type) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET player_id = $2, date = $3, description = $4, player_award_type = $5 WHERE player_awards.id = $1;").bind(self.id).bind(self.player_id).bind(self.date).bind(&self.description).bind(&self.player_award_type).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
