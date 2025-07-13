use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct Tracks {
    abbr: String,
    cup: i32,
    categories: String,
}

impl super::OldFixtureJson for Tracks {
    const FILENAME: &str = "tracks.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::tracks::Tracks {
            id: key,
            categories: self
                .categories
                .split(',')
                .map(|v| crate::sql::tables::Category::try_from(v.parse::<u8>().unwrap()).unwrap())
                .collect(),
            abbr: self.abbr,
            cup_id: self.cup,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::tracks::Tracks {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO tracks (id, abbr, cup_id, categories) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET abbr = $2, cup_id = $3, categories = $4 WHERE tracks.id = $1;").bind(self.id).bind(&self.abbr).bind(self.cup_id).bind(&self.categories).execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
