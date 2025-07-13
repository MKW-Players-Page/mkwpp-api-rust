use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::decode_rows_to_table,
    },
    app_state::cache::CacheItem,
    sql::tables::BasicTableQueries,
};
use sqlx::FromRow;

#[derive(serde::Deserialize, Debug, serde::Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Standards {
    pub id: i32,
    pub standard_level_id: i32,
    pub track_id: i32,
    pub category: super::Category,
    pub is_lap: bool,
    pub value: Option<i32>,
}

impl super::BasicTableQueries for Standards {
    const TABLE_NAME: &'static str = "standards";
}

impl CacheItem for Standards {
    type Input = ();

    async fn load(
        executor: &mut sqlx::PgConnection,
        _input: Self::Input,
    ) -> Result<Vec<Self>, FinalErrorResponse>
    where
        Self: Sized,
    {
        decode_rows_to_table::<Self>(sqlx::query(
            format!(
                "SELECT * FROM {this_table} ORDER BY track_id ASC, is_lap ASC, category DESC, value ASC;",
                this_table = Self::TABLE_NAME
            )
            .as_str(),
        )
        .fetch_all(executor)
        .await.map_err(|e|EveryReturnedError::GettingFromDatabase.into_final_error(e))?)
    }
}
