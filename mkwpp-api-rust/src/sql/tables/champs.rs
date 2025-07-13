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
