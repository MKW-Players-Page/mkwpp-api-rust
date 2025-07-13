use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Champs {
    player: i32,
    date_instated: String,
    category: u8,
}

impl super::OldFixtureJson for Champs {
    const FILENAME: &str = "sitechampions.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::champs::Champs {
            id: key,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            date_instated: chrono::NaiveDate::parse_from_str(
                self.date_instated.split('T').next().unwrap(),
                "%F",
            )
            .unwrap(),
            player_id: self.player,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::champs::Champs {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO site_champs (id, player_id, category, date_instated) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET player_id = $2, category = $3, date_instated = $4 WHERE site_champs.id = $1;").bind(self.id).bind(self.player_id).bind(self.category).bind(self.date_instated).execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
