use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracks {
    pub id: i32,
    pub abbr: String,
    pub cup_id: i32,
    pub categories: Vec<super::Category>,
}

impl super::BasicTableQueries for Tracks {
    const TABLE_NAME: &'static str = "tracks";
}

impl Tracks {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO tracks (id, abbr, cup_id, categories) VALUES($1, $2, $3, $4);")
    //         .bind(self.id)
    //         .bind(&self.abbr)
    //         .bind(self.cup_id)
    //         .bind(&self.categories)
    //         .execute(executor)
    //         .await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO tracks (id, abbr, cup_id, categories) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET abbr = $2, cup_id = $3, categories = $4 WHERE tracks.id = $1;").bind(self.id).bind(&self.abbr).bind(self.cup_id).bind(&self.categories).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }
}
