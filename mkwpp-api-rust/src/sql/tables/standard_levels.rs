use anyhow::anyhow;
use sqlx::FromRow;

use crate::{app_state::cache::CacheItem, sql::tables::BasicTableQueries};

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow)]
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

impl StandardLevels {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query(
    //         "INSERT INTO standard_levels (id, code, value, is_legacy) VALUES($1, $2, $3, $4);",
    //     )
    //     .bind(self.id)
    //     .bind(&self.code)
    //     .bind(self.value)
    //     .bind(self.is_legacy)
    //     .execute(executor)
    //     .await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO standard_levels (id, code, value, is_legacy) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET code = $2, value = $3, is_legacy = $4 WHERE standard_levels.id = $1;").bind(self.id).bind(&self.code).bind(self.value).bind(self.is_legacy).execute(executor).await;
    }
}

impl CacheItem for StandardLevels {
    type Input = ();

    async fn load(
        executor: &mut sqlx::PgConnection,
        _input: Self::Input,
    ) -> Result<Vec<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        match sqlx::query(
            format!(
                "SELECT * FROM {this_table} WHERE is_legacy = TRUE;",
                this_table = Self::TABLE_NAME
            )
            .as_str(),
        )
        .fetch_all(executor)
        .await
        {
            Ok(v) => v
                .into_iter()
                .map(|r| {
                    StandardLevels::from_row(&r)
                        .map_err(|e| anyhow!("Error in loading Legacy Standard Levels. {e}"))
                })
                .collect::<Result<Vec<StandardLevels>, anyhow::Error>>(),

            Err(e) => Err(anyhow!("Error in loading Legacy Standard Levels. {e}")),
        }
    }
}
