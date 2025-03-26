use crate::sql::tables::BasicTableQueries;

pub use super::Times;

impl Times {
    // TODO: Hardcoded value for Newbie Code
    async fn get_times(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        let region_ids = match crate::sql::tables::regions::Regions::get_descendants(
            executor, region_id,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        return sqlx::query(&format!(
            r#"
                SELECT *
                FROM (
                    SELECT *,
                        (RANK() OVER(PARTITION BY track_id ORDER BY value ASC))::INTEGER AS rank,
                        ((FIRST_VALUE(value) OVER(PARTITION BY track_id ORDER BY value ASC))::FLOAT8 / value::FLOAT8) AS prwr
                    FROM (
                        SELECT *
                        FROM (
                            SELECT
                                {0}.id AS s_id,
                                {0}.value,
                                {0}.category,
                                {0}.is_lap,
                                {0}.track_id,
                                ROW_NUMBER() OVER(
                                    PARTITION BY {1}.id, {0}.track_id
                                    ORDER BY {0}.value ASC, {3}.value ASC
                                ) AS row_n,
                                date,
                                video_link,
                                ghost_link,
                                comment,
                                initial_rank,
                                {1}.id,
                                COALESCE({3}.code, 'NW') AS std_lvl_code
                            FROM {0}
                            LEFT JOIN {1} ON
                                {0}.player_id = {1}.id
                            LEFT JOIN {2} ON
                                {0}.track_id = {2}.track_id AND
                                {0}.value <= {2}.value AND
                                {2}.category <= {0}.category AND
                                {2}.is_lap = {0}.is_lap
                            LEFT JOIN {3} ON
                                {3}.id = {2}.standard_level_id
                            WHERE
                                {0}.category <= $2 AND
                                {0}.date <= $4 AND
                                {1}.region_id = ANY($5) 
                                {4}
                            ORDER BY value ASC, {3}.value ASC
                        ) WHERE row_n = 1
                    ) ORDER BY value ASC, date DESC
                )
                WHERE id = $1
                ORDER BY track_id;
            "#,
            super::Scores::TABLE_NAME,
            crate::sql::tables::players::Players::TABLE_NAME,
            crate::sql::tables::standards::Standards::TABLE_NAME,
            crate::sql::tables::standard_levels::StandardLevels::TABLE_NAME,
            if is_lap.is_some() {
                "AND is_lap = $3"
            } else {
                ""
            }
        ))
        .bind(player_id)
        .bind(category)
        .bind(is_lap)
        .bind(max_date)
        .bind(region_ids)
        .fetch_all(executor)
        .await;
    }
}

#[serde_with::skip_serializing_none]
#[derive(serde::Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timesheet {
    pub times: Vec<Times>,
    pub af: Option<f64>,
    pub total_time: Option<i32>,
    pub tally: Option<i16>,
    pub arr: Option<f64>,
    pub prwr: Option<f64>,
}

impl BasicTableQueries for Timesheet {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
}

impl Timesheet {
    pub async fn get_times(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return Times::get_times(executor, player_id, category, is_lap, max_date, region_id).await;
    }

    pub fn new(
        times: Vec<Times>,
        af: Option<f64>,
        arr: Option<f64>,
        total_time: Option<i32>,
        prwr: Option<f64>,
        tally: Option<i16>,
    ) -> Self {
        Timesheet {
            times,
            af,
            arr,
            total_time,
            prwr,
            tally,
        }
    }
}
