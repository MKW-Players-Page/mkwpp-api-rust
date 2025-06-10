pub use super::SlowestTimes;
use crate::app_state::cache::CacheItem;
use crate::sql::tables::{BasicTableQueries, players::players_basic::PlayersBasic};
use anyhow::anyhow;
use sqlx::FromRow;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct SlowestTimesInputs {
    pub category: crate::sql::tables::Category,
    pub max_date: chrono::NaiveDate,
    pub region_id: i32,
}

impl BasicTableQueries for SlowestTimes {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

impl SlowestTimes {
    // TODO: What happens when there is no
    // slowest time with the current filters?
    // Come up with defaults.
    pub async fn get(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        return sqlx::query(&format!(
            r#"
                SELECT
                    track_id,
                    is_lap,
                    (MAX(value) + 1) AS value
                FROM (
                    SELECT *,
                        ROW_NUMBER() OVER(
                            PARTITION BY
                                player_id,
                                track_id,
                                category,
                                is_lap
                            ORDER BY value ASC
                        ) AS row_n
                    FROM {this_table}
                    LEFT JOIN {players_table} ON
                        {this_table}.player_id = {players_table}.id
                    WHERE
                        category <= $1 AND
                        date <= $2 AND
                        region_id = ANY($3)
                )
                WHERE row_n = 1
                GROUP BY track_id, category, is_lap
            "#,
            this_table = Self::TABLE_NAME,
            players_table = PlayersBasic::TABLE_NAME
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .fetch_all(executor)
        .await;
    }
}

impl CacheItem for SlowestTimes {
    type Input = SlowestTimesInputs;

    async fn load(
        executor: &mut sqlx::PgConnection,
        input: Self::Input,
    ) -> Result<Vec<Self>, anyhow::Error> {
        match SlowestTimes::get(executor, input.category, input.max_date, input.region_id).await {
            Err(e) => return Err(anyhow!("Error loading slowest times. {e}")),
            Ok(v) => v
                .into_iter()
                .map(|r| {
                    SlowestTimes::from_row(&r)
                        .map_err(|e| anyhow!("Error loading slowest times. {e}"))
                })
                .collect::<Result<Vec<SlowestTimes>, anyhow::Error>>(),
        }
    }
}
