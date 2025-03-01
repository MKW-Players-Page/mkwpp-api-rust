#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow)]
pub struct StandardLevels {
    pub id: i32,
    pub code: String,
    pub value: i32,
    pub is_legacy: bool,
}

impl super::BasicTableQueries for StandardLevels {
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
