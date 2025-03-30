use sqlx::{FromRow, Row};

use crate::sql::tables::BasicTableQueries;

pub use super::Times;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timesheet {
    pub times: Vec<Times>,
    pub af: f64,
    pub total_time: i32,
    pub tally: i16,
    pub arr: f64,
    pub prwr: f64,
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
    ) -> Result<Self, sqlx::Error> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        let raw_data = sqlx::query(&format!(
                    r#"
                    SELECT *
                    FROM (
                        SELECT *,
                            (RANK() OVER(PARTITION BY track_id, is_lap ORDER BY value ASC))::INTEGER AS rank,
                            ((FIRST_VALUE(value) OVER(PARTITION BY track_id, is_lap ORDER BY value ASC))::FLOAT8 / value::FLOAT8) AS prwr
                        FROM (
                            SELECT *
                            FROM (
                                SELECT
                                    {this_table}.id AS id,
                                    {this_table}.value,
                                    {this_table}.category,
                                    {this_table}.is_lap,
                                    {this_table}.track_id,
                                    ROW_NUMBER() OVER(
                                        PARTITION BY {players_table}.id, {this_table}.track_id, {this_table}.is_lap
                                        ORDER BY {this_table}.value ASC, {standard_levels_table}.value ASC
                                    ) AS row_n,
                                    date,
                                    video_link,
                                    ghost_link,
                                    comment,
                                    initial_rank,
                                    {players_table}.id as p_id,
                                    COALESCE({standard_levels_table}.code, 'NW') AS std_lvl_code,
                                    COALESCE({standards_table}.value, 36) AS arr_value
                                FROM {this_table}
                                LEFT JOIN {players_table} ON
                                    {this_table}.player_id = {players_table}.id
                                LEFT JOIN {standards_table} ON
                                    {this_table}.track_id = {standards_table}.track_id AND
                                    {this_table}.value <= {standards_table}.value AND
                                    {standards_table}.category <= {this_table}.category AND
                                    {standards_table}.is_lap = {this_table}.is_lap
                                LEFT JOIN {standard_levels_table} ON
                                    {standard_levels_table}.id = {standards_table}.standard_level_id
                                WHERE
                                    {this_table}.category <= $1 AND
                                    {this_table}.date <= $3 AND
                                    {players_table}.region_id = ANY($4) 
                                    {is_lap}
                                ORDER BY value ASC, {standard_levels_table}.value ASC
                            ) WHERE row_n = 1
                        )
                    ) ORDER BY track_id, is_lap, value ASC, date DESC;
                    "#,
                    this_table = super::Scores::TABLE_NAME,
                    players_table = crate::sql::tables::players::Players::TABLE_NAME,
                    standards_table = crate::sql::tables::standards::Standards::TABLE_NAME,
                    standard_levels_table = crate::sql::tables::standard_levels::StandardLevels::TABLE_NAME,
                    is_lap = if is_lap.is_some() {
                        format!("AND {this_table}.is_lap = $2", this_table = super::Scores::TABLE_NAME)
                    } else {
                        String::new()
                    }
                ))
                .bind(category)
                .bind(is_lap)
                .bind(max_date)
                .bind(region_ids)
                .fetch_all(executor)
                .await?;

        let divvie_value = match is_lap {
            Some(_) => 32.0,
            None => 64.0,
        };

        let mut last_track = 0;
        let mut last_lap_type = false;
        let mut last_time = 0;
        let mut last_rank = 0;
        let mut has_found_track_time = false;

        let mut total_time = 0;
        let mut total_rank = 0;
        let mut total_rank_rating = 0;
        let mut tally_points = 0;
        let mut total_prwr = 0.0;
        let mut times = vec![];

        for row in raw_data {
            let time_data = Times::from_row(&row)?;

            if has_found_track_time
                && last_track == time_data.track_id
                && last_lap_type == time_data.is_lap
            {
                continue;
            }

            if last_track < time_data.track_id || last_lap_type != time_data.is_lap {
                if !has_found_track_time {
                    total_time += last_time + 1;
                    total_rank += last_rank + 1;
                    total_rank_rating += 36;
                }

                has_found_track_time = false;
            }

            last_track = time_data.track_id;
            last_time = time_data.value;
            last_lap_type = time_data.is_lap;
            last_rank = time_data.rank;

            let row_player_id: i32 = row.try_get("p_id")?;
            if row_player_id != player_id {
                continue;
            }

            has_found_track_time = true;
            total_time += time_data.value;
            total_rank += time_data.rank;
            total_prwr += time_data.prwr;
            tally_points += std::cmp::max(11 - time_data.rank, 0);
            total_rank_rating += row.try_get::<i32, &str>("arr_value")?;
            times.push(time_data);
        }

        Ok(Self {
            times,
            af: (total_rank as f64) / divvie_value,
            total_time,
            tally: tally_points as i16,
            arr: (total_rank_rating as f64) / divvie_value,
            prwr: total_prwr / divvie_value,
        })
    }
}
