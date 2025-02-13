#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct PlayersBasic {
    pub id: i32,
    pub name: String,
    pub alias: Option<String>,
    pub region_id: i32,
}

impl crate::sql::tables::BasicTableQueries for PlayersBasic {
    fn table_name() -> &'static str {
        return super::Players::table_name();
    }
    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            "SELECT id, name, alias, region_id from {};",
            super::Players::table_name(),
        ))
        .fetch_all(executor)
        .await;
    }
}
