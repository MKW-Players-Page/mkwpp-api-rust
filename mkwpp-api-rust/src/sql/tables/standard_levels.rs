use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::decode_rows_to_table,
    },
    app_state::cache::CacheItem,
    sql::tables::BasicTableQueries,
};

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardLevels {
    pub id: i32,
    pub code: String,
    pub value: i32,
    pub is_legacy: bool,
}

impl BasicTableQueries for StandardLevels {
    const TABLE_NAME: &'static str = "standard_levels";
}

impl CacheItem for StandardLevels {
    type Input = ();

    async fn load(
        executor: &mut sqlx::PgConnection,
        _input: Self::Input,
    ) -> Result<Vec<Self>, FinalErrorResponse>
    where
        Self: Sized,
    {
        decode_rows_to_table::<Self>(
            sqlx::query(
                format!(
                    "SELECT * FROM {this_table} WHERE is_legacy = TRUE;",
                    this_table = Self::TABLE_NAME
                )
                .as_str(),
            )
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )
    }
}
