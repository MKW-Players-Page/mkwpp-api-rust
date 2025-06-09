use crate::sql::tables::BasicTableQueries;
use crate::sql::tables::players::FilterByPlayerId;

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayersBasic {
    pub id: i32,
    pub name: String,
    pub alias: Option<String>,
    pub region_id: i32,
}

impl BasicTableQueries for PlayersBasic {
    const TABLE_NAME: &'static str = super::Players::TABLE_NAME;

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            "SELECT id, name, alias, region_id from {};",
            Self::TABLE_NAME,
        ))
        .fetch_all(executor)
        .await;
    }
}

impl FilterByPlayerId for PlayersBasic {
    const GET_SELECT_PLAYERS_QUERY_STR: &'static str = const_format::formatc!(
        "SELECT * FROM {} WHERE id = ANY($1);",
        PlayersBasic::TABLE_NAME
    );
}
