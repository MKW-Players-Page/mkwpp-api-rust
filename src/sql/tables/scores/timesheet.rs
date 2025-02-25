use crate::sql::tables::{BasicTableQueries, Category};
use sqlx::{FromRow, Row};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct Timesheet {
    pub rank: Option<i32>,
    pub prwr: Option<f64>,
    pub std_lvl_code: String,
    pub id: i32,
    pub value: i32,
    pub category: Category,
    pub is_lap: bool,
    pub track_id: i32,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub initial_rank: Option<i32>,
}

impl BasicTableQueries for Timesheet {
    fn table_name() -> &'static str {
        return super::Scores::table_name();
    }
}


impl<'a> FromRow<'a, sqlx::postgres::PgRow> for Timesheet {
    fn from_row(row: &'a sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            rank: row.try_get("rank").unwrap_or(None),
            prwr: row.try_get("prwr").unwrap_or(None),
            id: row.try_get("s_id")?,
            value: row.try_get("value")?,
            category: row.try_get("category")?,
            std_lvl_code: row.try_get("code")?,
            is_lap: row.try_get("is_lap")?,
            track_id: row.try_get("track_id")?,
            date: row.try_get("date")?,
            video_link: row.try_get("video_link")?,
            ghost_link: row.try_get("ghost_link")?,
            comment: row.try_get("comment")?,
            initial_rank: row.try_get("initial_rank")?,
        })
    }
}

impl Timesheet {
    // TODO: Hardcoded value for Newbie Code
    pub async fn get_timesheet(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        let region_ids =
            match crate::sql::tables::regions::Regions::get_nephews(region_id, executor).await {
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
                                COALESCE({3}.code, 'NW') AS code
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
            super::Scores::table_name(),
            crate::sql::tables::players::Players::table_name(),
            crate::sql::tables::standards::Standards::table_name(),
            crate::sql::tables::standard_levels::StandardLevels::table_name(),
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
