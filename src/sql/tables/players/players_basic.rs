use crate::sql::tables::BasicTableQueries;

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PlayersBasic {
    pub id: i32,
    pub name: String,
    pub alias: Option<String>,
    pub region_id: i32,
}

impl BasicTableQueries for PlayersBasic {
    fn table_name() -> &'static str {
        return super::Players::table_name();
    }
    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            "SELECT id, name, alias, region_id from {};",
            Self::table_name(),
        ))
        .fetch_all(executor)
        .await;
    }
}

impl PlayersBasic {
    pub async fn get_select_players(
        executor: &mut sqlx::PgConnection,
        player_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            "SELECT id, name, alias, region_id from {} WHERE id = ANY($1);",
            Self::table_name(),
        ))
        .bind(player_ids)
        .fetch_all(executor)
        .await;
    }
}
