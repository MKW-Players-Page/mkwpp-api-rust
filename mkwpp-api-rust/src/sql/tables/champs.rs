use crate::{
    api::errors::{EveryReturnedError, FinalErrorResponse},
    custom_serde::DateAsTimestampNumber,
};
use sqlx::postgres::PgRow;

use super::Category;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Champs {
    pub id: i32,
    pub player_id: i32,
    pub category: super::Category,
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub date_instated: chrono::NaiveDate,
}

impl super::BasicTableQueries for Champs {
    const TABLE_NAME: &'static str = "site_champs";
}

impl Champs {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO site_champs (id, player_id, category, date_instated) VALUES($1, $2, $3, $4);").bind(self.id).bind(self.player_id).bind(&self.category).bind(self.date_instated).execute(executor).await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO site_champs (id, player_id, category, date_instated) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET player_id = $2, category = $3, date_instated = $4 WHERE site_champs.id = $1;").bind(self.id).bind(self.player_id).bind(self.category).bind(self.date_instated).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn filter_by_category(
        category: Category,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<PgRow>, FinalErrorResponse> {
        return sqlx::query(
            "SELECT * FROM site_champs WHERE category = $1 ORDER BY date_instated ASC;",
        )
        .bind(category)
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
