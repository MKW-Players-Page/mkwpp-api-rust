use sqlx::FromRow;

use crate::sql::tables::{
    BasicTableQueries,
    scores::{TimesheetTimesetData, rankings::ValidTimesetItem, timesets::Timeset},
};

pub use super::Times;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Timesheet {
    pub times: Vec<Times>,
    pub af: f64,
    pub total_time: i32,
    pub tally: i16,
    pub arr: f64,
    pub prwr: f64,
}

impl ValidTimesetItem for TimesheetTimesetData {
    fn get_date(&self) -> Option<chrono::NaiveDate> {
        self.date
    }
    fn get_time(&self) -> i32 {
        self.value
    }
    fn set_prwr(&mut self, prwr: f64) {}
    fn set_rank(&mut self, rank: i32) {}
    fn get_is_lap(&self) -> bool {
        self.is_lap
    }
    fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }
    fn get_time_id(&self) -> i32 {
        self.id
    }
    fn get_category(&self) -> crate::sql::tables::Category {
        self.category
    }
    fn get_track_id(&self) -> i32 {
        self.track_id
    }
    fn get_player_id(&self) -> i32 {
        self.player_id
    }
    fn get_ghost_link(&self) -> Option<String> {
        self.ghost_link.clone()
    }
    fn get_video_link(&self) -> Option<String> {
        self.video_link.clone()
    }
    fn get_initial_rank(&self) -> Option<i32> {
        self.initial_rank
    }
    fn get_player_region_id(&self) -> i32 {
        0
    }
}

impl BasicTableQueries for Timesheet {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

impl Timesheet {
    // TODO: Hardcoded values for Newbie Standard
    pub async fn timesheet(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Self, anyhow::Error> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        let timeset = sqlx::query(&format!(
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
        .await?
        .into_iter()
        .map(|score_row| TimesheetTimesetData::from_row(&score_row))
        .collect::<Result<Vec<TimesheetTimesetData>, sqlx::Error>>()?;

        let mut timeset_encoder = Timeset::default();
        timeset_encoder.timeset = timeset;
        timeset_encoder.filters.category = category;
        timeset_encoder.filters.is_lap = is_lap;
        timeset_encoder.filters.max_date = max_date;

        timeset_encoder.timesheet(player_id).await
    }
}
