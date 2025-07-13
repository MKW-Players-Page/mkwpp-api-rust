use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct StandardLevels {
    code: String,
    value: i32,
    is_legacy: bool,
}

impl super::OldFixtureJson for StandardLevels {
    const FILENAME: &str = "standardlevels.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::standard_levels::StandardLevels {
            id: key,
            code: self.code,
            value: self.value,
            is_legacy: self.is_legacy,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}

impl crate::sql::tables::standard_levels::StandardLevels {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO standard_levels (id, code, value, is_legacy) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET code = $2, value = $3, is_legacy = $4 WHERE standard_levels.id = $1;").bind(self.id).bind(&self.code).bind(self.value).bind(self.is_legacy).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
