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

impl Standards {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO standards (id, standard_level_id, track_id, category, is_lap, value) VALUES($1, $2, $3, $4, $5, $6);").bind(self.id).bind(self.standard_level_id).bind(self.track_id).bind(&self.category).bind(self.is_lap).bind(self.value).execute(executor).await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data_old")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO standards (id, standard_level_id, track_id, category, is_lap, value) VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET standard_level_id = $2, track_id = $3, category = $4, is_lap = $5, value = $6 WHERE standards.id = $1;").bind(self.id).bind(self.standard_level_id).bind(self.track_id).bind(self.category).bind(self.is_lap).bind(self.value).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
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
