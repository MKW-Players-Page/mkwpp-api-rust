use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Players {
    name: String,
    alias: Option<String>,
    bio: Option<String>,
    region: Option<i32>,
    joined_date: Option<String>,
    last_activity: Option<String>,
}

impl super::OldFixtureJson for Players {
    const FILENAME: &str = "players.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        let joined_date = self
            .joined_date
            .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap())
            .unwrap_or_default();

        return crate::sql::tables::players::Players {
            id: key,
            name: self.name,
            alias: self.alias,
            bio: self.bio,
            region_id: self.region.unwrap_or(1),
            pronouns: None,
            joined_date,
            last_activity: self
                .last_activity
                .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap())
                .unwrap_or(joined_date),
            submitters: Vec::with_capacity(0),
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::players::Players {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity, submitters) VALUES($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET name = $2, alias = $3, bio = $4, region_id = $5, joined_date = $6, last_activity = $7, submitters = $8 WHERE players.id = $1;").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(self.region_id).bind(self.joined_date).bind(self.last_activity).bind(&self.submitters).execute(executor).await.map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
