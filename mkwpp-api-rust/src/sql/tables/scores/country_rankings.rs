use sqlx::FromRow;

use crate::sql::tables::{
    BasicTableQueries, Category,
    players::Players,
    regions::RegionType,
    scores::{CountryRankingsTimesetData, rankings::ValidTimesetItem, timesets::Timeset},
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountryRankings {
    pub region_id: i32,
    pub rank: i32,
    pub value: f64,
}

impl CountryRankings {
    pub async fn get_country_af(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_type: RegionType,
        player_numbers: i32,
    ) -> Result<Vec<CountryRankings>, anyhow::Error> {
        let timeset = sqlx::query(&format!(
            r#"
            SELECT
                value, category, is_lap, track_id, player_id, region_id
            FROM (
                SELECT
                    value,
                    category,
                    is_lap,
                    track_id,
                    player_id,
                    date,
                    region_id,
                    ROW_NUMBER() OVER(
                        PARTITION BY player_id, track_id, is_lap
                        ORDER BY value ASC, date DESC
                    ) AS row_n
                FROM {this_table}
                LEFT JOIN {players_table} ON
                    {this_table}.player_id = {players_table}.id
                WHERE
                    category <= $1 AND
                    date <= $3
                    {is_lap}
                ORDER BY value ASC
            )
            WHERE row_n = 1
            ORDER BY track_id ASC, is_lap ASC, value ASC, date DESC;
            "#,
            this_table = super::Scores::TABLE_NAME,
            players_table = Players::TABLE_NAME,
            is_lap = if is_lap.is_some() {
                "AND is_lap = $2".to_string()
            } else {
                String::new()
            }
        ))
        .bind(category)
        .bind(is_lap)
        .bind(max_date)
        .fetch_all(executor)
        .await?
        .into_iter()
        .map(|score_row| CountryRankingsTimesetData::from_row(&score_row))
        .collect::<Result<Vec<CountryRankingsTimesetData>, sqlx::Error>>()?;

        let mut timeset_encoder = Timeset::default();
        timeset_encoder.timeset = timeset;
        timeset_encoder.filters.category = category;
        timeset_encoder.filters.is_lap = is_lap;
        timeset_encoder.filters.max_date = max_date;
        timeset_encoder
            .calculate_country_rankings(region_type, player_numbers)
            .await
    }
}

impl ValidTimesetItem for CountryRankingsTimesetData {
    fn get_time(&self) -> i32 {
        self.value
    }
    fn get_track_id(&self) -> i32 {
        self.track_id
    }
    fn get_is_lap(&self) -> bool {
        self.is_lap
    }
    fn get_player_id(&self) -> i32 {
        self.player_id
    }
    fn set_rank(&mut self, _rank: i32) {}
    fn set_prwr(&mut self, _prwr: f64) {}
    fn get_date(&self) -> Option<chrono::NaiveDate> {
        None
    }
    fn get_comment(&self) -> Option<String> {
        None
    }
    fn get_time_id(&self) -> i32 {
        0
    }
    fn get_category(&self) -> crate::sql::tables::Category {
        Category::NonSc
    }
    fn get_ghost_link(&self) -> Option<String> {
        None
    }
    fn get_video_link(&self) -> Option<String> {
        None
    }
    fn get_initial_rank(&self) -> Option<i32> {
        None
    }
    fn get_player_region_id(&self) -> i32 {
        self.region_id
    }
}
