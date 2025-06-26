use std::fmt::Debug;

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::decode_rows_to_table,
    },
    sql::tables::{
        BasicTableQueries,
        scores::{TimesheetTimesetData, timesets::Timeset},
    },
};

use super::timesheet::Timesheet;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchupData {
    pub player_data: Vec<Timesheet>,
    pub wins: Vec<i8>,
    pub diff_first: Vec<Vec<i32>>,
    pub diff_next: Vec<Vec<i32>>,
    pub diff_af_first: Vec<f64>,
    pub diff_af_next: Vec<f64>,
    pub diff_total_time_first: Vec<i32>,
    pub diff_total_time_next: Vec<i32>,
    pub diff_tally_first: Vec<i16>,
    pub diff_tally_next: Vec<i16>,
    pub diff_arr_first: Vec<f64>,
    pub diff_arr_next: Vec<f64>,
    pub diff_prwr_first: Vec<f64>,
    pub diff_prwr_next: Vec<f64>,
    pub diff_wins_first: Vec<i8>,
    pub diff_wins_next: Vec<i8>,
    pub rgb_diff: Vec<Vec<u8>>,
    pub rgb_diff_af: Vec<u8>,
    pub rgb_diff_total_time: Vec<u8>,
    pub rgb_diff_tally: Vec<u8>,
    pub rgb_diff_arr: Vec<u8>,
    pub rgb_diff_prwr: Vec<u8>,
    pub rgb_diff_wins: Vec<u8>,
}

impl BasicTableQueries for MatchupData {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

impl MatchupData {
    // TODO: Hardcoded values for Newbie Standard
    pub async fn get(
        executor: &mut sqlx::PgConnection,
        player_ids: Vec<i32>,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Self, FinalErrorResponse> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        let timeset = decode_rows_to_table::<TimesheetTimesetData>(
            sqlx::query(&format!(
                r#"
                SELECT
                    id, value, category, is_lap, track_id, 
                    player_id, date, video_link, ghost_link,
                    comment, initial_rank
                FROM (
                    SELECT
                        {this_table}.id, value,
                        category, is_lap, track_id,
                        {players_table}.id AS player_id,
                        date, video_link, ghost_link, comment, initial_rank,
                        ROW_NUMBER() OVER(
                            PARTITION BY player_id, track_id, is_lap
                            ORDER BY value ASC, date DESC
                        ) AS row_n
                    FROM {this_table}
                    LEFT JOIN {players_table}
                        ON {this_table}.player_id = {players_table}.id
                    WHERE
                        category <= $1 AND
                        date <= $3 AND
                        region_id = ANY($4)
                        {is_lap}
                    ORDER BY value ASC
                )
                WHERE row_n = 1
                ORDER BY track_id ASC, is_lap ASC, value ASC, date DESC;
                "#,
                this_table = super::Scores::TABLE_NAME,
                players_table = super::PlayersBasic::TABLE_NAME,
                is_lap = if is_lap.is_some() {
                    "AND is_lap = $2".to_string()
                } else {
                    String::new()
                }
            ))
            .bind(category)
            .bind(is_lap)
            .bind(max_date)
            .bind(&region_ids)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )?;

        let mut timeset_encoder = Timeset::default();
        timeset_encoder.timeset = timeset;
        timeset_encoder.filters.category = category;
        timeset_encoder.filters.is_lap = is_lap;
        timeset_encoder.filters.max_date = max_date;

        timeset_encoder.matchup(player_ids).await
    }
}
