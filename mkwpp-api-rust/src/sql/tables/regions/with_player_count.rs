use crate::sql::tables::BasicTableQueries;

pub use super::RegionsWithPlayerCount;

impl BasicTableQueries for RegionsWithPlayerCount {
    const TABLE_NAME: &'static str = super::Regions::TABLE_NAME;

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(const_format::formatc!(
            r#"
            SELECT
                {table_name}.*,
                COALESCE(player_count, 0) AS player_count
            FROM {table_name}
            LEFT JOIN (
                SELECT region_id, COUNT(region_id) AS player_count
                FROM {players_table}
                GROUP BY region_id
            ) AS z ON
                z.region_id = {table_name}.id
            ORDER BY player_count DESC;
            "#,
            table_name = RegionsWithPlayerCount::TABLE_NAME,
            players_table = crate::sql::tables::players::Players::TABLE_NAME
        ))
        .fetch_all(executor)
        .await;
    }
}
