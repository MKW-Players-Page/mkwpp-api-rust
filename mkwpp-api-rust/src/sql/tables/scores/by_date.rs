use crate::api::errors::{EveryReturnedError, FinalErrorResponse};
pub use crate::sql::tables::BasicTableQueries;

pub use super::ScoresByDate;

impl BasicTableQueries for ScoresByDate {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

#[derive(PartialEq)]
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
        return sqlx::query(include_str!("../../../../../db/queries/by_date.sql"))
            .bind(limit)
            .bind(order_type == OrderType::Records)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
