pub use crate::sql::tables::BasicTableQueries;
use crate::{
    api::errors::{EveryReturnedError, FinalErrorResponse},
    sql::tables::players::players_basic::PlayersBasic,
};

pub use super::ScoresByDate;

impl BasicTableQueries for ScoresByDate {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

enum OrderType {
    All,
    Records,
}

impl ScoresByDate {
    pub async fn order_by_date(
        executor: &mut sqlx::PgConnection,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return Self::order(executor, OrderType::All, limit).await;
    }

    pub async fn order_records_by_date(
        executor: &mut sqlx::PgConnection,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return Self::order(executor, OrderType::Records, limit).await;
    }

    async fn order(
        executor: &mut sqlx::PgConnection,
        order_type: OrderType,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(&format!(
            r#"
            SELECT
                {scores_table}.id AS s_id,
                value, category,
                is_lap, track_id,
                date, {players_basic_table}.id, name,
                alias, region_id
            FROM {scores_table}
            LEFT JOIN {players_basic_table} ON {scores_table}.player_id = {players_basic_table}.id
            WHERE
                date IS NOT NULL
                {order_type}
            ORDER BY date DESC
            LIMIT $1;
            "#,
            scores_table = super::Scores::TABLE_NAME,
            players_basic_table = PlayersBasic::TABLE_NAME,
            order_type = match order_type {
                OrderType::All => "",
                OrderType::Records => "AND initial_rank = 1",
            }
        ))
        .bind(limit)
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }
}
