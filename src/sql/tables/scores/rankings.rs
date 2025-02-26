use crate::sql::tables::players::players_basic::PlayersBasic;
use crate::sql::tables::BasicTableQueries;
use sqlx::{FromRow, Row};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct Rankings {
    pub rank: i32,
    pub value: RankingType,
    pub player: PlayersBasic,
}

impl BasicTableQueries for Rankings {
    fn table_name() -> &'static str {
        return super::Scores::table_name();
    }
}

impl<'a> FromRow<'a, sqlx::postgres::PgRow> for Rankings {
    fn from_row(row: &'a sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let data = match row.try_get("personal_record_world_record") {
            Ok(v) => RankingType::PersonalRecordWorldRecord(v),
            Err(_) => match row.try_get("total_time") {
                Ok(v) => RankingType::TotalTime(v),
                Err(_) => match row.try_get("average_finish") {
                    Ok(v) => RankingType::AverageFinish(v),
                    Err(_) => match row.try_get("tally_points") {
                        Ok(v) => RankingType::TallyPoints(v),
                        Err(_) => {
                            RankingType::AverageRankRating(row.try_get("average_rank_rating")?)
                        }
                    },
                },
            },
        };

        return Ok(Rankings {
            rank: row.try_get("rank")?,
            player: PlayersBasic {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                alias: row.try_get("alias").ok(),
                region_id: row.try_get("region_id")?,
            },
            value: data,
        });
    }
}

impl Rankings {
    pub async fn get(
        executor: &mut sqlx::PgConnection,
        ranking_type: RankingType,
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

        match ranking_type {
            RankingType::AverageFinish(_) => {
                Self::get_average_finish(executor, category, is_lap, max_date, region_ids).await
            }
            RankingType::TotalTime(_) => {
                Self::get_total_time(executor, category, is_lap, max_date, region_ids).await
            }
            RankingType::PersonalRecordWorldRecord(_) => {
                Self::get_personal_record_world_record(
                    executor, category, is_lap, max_date, region_ids,
                )
                .await
            }
            RankingType::TallyPoints(_) => {
                Self::get_tally_points(executor, category, is_lap, max_date, region_ids).await
            }
            RankingType::AverageRankRating(_) => {
                Self::get_average_rank_rating(executor, category, is_lap, max_date, region_ids)
                    .await
            }
        }
    }

    async fn get_total_time(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            r#"
            WITH
                slowest_times AS (
                    SELECT
                        track_id,
                        category,
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
                        FROM {0}
                    )
                    WHERE row_n = 1 AND category <= $1 {3}
                    GROUP BY track_id, category, is_lap
                ),
                track_ids AS (
                    SELECT id AS track_id, t.is_lap FROM {2}
                    CROSS JOIN (VALUES (TRUE),(FALSE)) AS t (is_lap)
                )
            SELECT *,
                (RANK() OVER(ORDER BY total_time ASC))::INTEGER AS rank
            FROM (
                SELECT
                    id,
                    MAX(name) AS name,
                    MAX(alias) AS alias,
                    MAX(region_id) AS region_id,
                    SUM(value)::INTEGER AS total_time
                FROM (
                    SELECT * FROM (
                        SELECT *,
                            ROW_NUMBER() OVER (
                                PARTITION BY
                                    id,
                                    track_id,
                                    category,
                                    is_lap
                                ORDER BY value
                            ) AS row_n
                        FROM (
                            SELECT
                                COALESCE({0}.value, slowest_times.value) AS value,
                                COALESCE({0}.category, slowest_times.category) AS category,
                                COALESCE({0}.date, '2008-01-01'::DATE) AS date,
                                {1}.id,
                                {1}.name,
                                {1}.alias,
                                {1}.region_id,
                                track_ids.track_id,
                                track_ids.is_lap
                            FROM {1}
                            CROSS JOIN track_ids
                            LEFT JOIN {0} ON
                                {0}.player_id = {1}.id AND
                                {0}.is_lap = track_ids.is_lap AND
                                {0}.track_id = track_ids.track_id AND
                                {0}.category <= $1
                            LEFT JOIN slowest_times ON
                                slowest_times.track_id = track_ids.track_id AND
                                slowest_times.is_lap = track_ids.is_lap
                        )
                    )
                    WHERE 
                        row_n = 1 AND
                        category <= $1 AND
                        date <= $2 AND
                        region_id = ANY($3)
                        {3}
                )
                GROUP BY id
            ) ORDER BY total_time ASC;
            "#,
            Self::table_name(),
            PlayersBasic::table_name(),
            crate::sql::tables::tracks::Tracks::table_name(),
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }

    async fn get_personal_record_world_record(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            r#"
            WITH
                slowest_times AS (
                    SELECT
                        track_id,
                        category,
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
                        FROM {0}
                    )
                    WHERE row_n = 1 AND category <= $1 {3}
                    GROUP BY track_id, category, is_lap
                ),
                track_ids AS (
                    SELECT id AS track_id, t.is_lap FROM {2}
                    CROSS JOIN (VALUES (TRUE),(FALSE)) AS t (is_lap)
                )
            SELECT *,
                id, region_id, name, alias,
                ((FIRST_VALUE(total_time) OVER(ORDER BY total_time ASC))::FLOAT8 / total_time::FLOAT8) AS personal_record_world_record,
                (RANK() OVER(ORDER BY total_time ASC))::INTEGER AS rank
            FROM (
                SELECT
                    id,
                    MAX(name) AS name,
                    MAX(alias) AS alias,
                    MAX(region_id) AS region_id,
                    SUM(value)::INTEGER AS total_time
                FROM (
                    SELECT * FROM (
                        SELECT *,
                            ROW_NUMBER() OVER (
                                PARTITION BY
                                    id,
                                    track_id,
                                    category,
                                    is_lap
                                ORDER BY value
                            ) AS row_n
                        FROM (
                            SELECT
                                COALESCE({0}.value, slowest_times.value) AS value,
                                COALESCE({0}.category, slowest_times.category) AS category,
                                COALESCE({0}.date, '2008-01-01'::DATE) AS date,
                                {1}.id,
                                {1}.name,
                                {1}.alias,
                                {1}.region_id,
                                track_ids.track_id,
                                track_ids.is_lap
                            FROM {1}
                            CROSS JOIN track_ids
                            LEFT JOIN {0} ON
                                {0}.player_id = {1}.id AND
                                {0}.is_lap = track_ids.is_lap AND
                                {0}.track_id = track_ids.track_id AND
                                {0}.category <= $1
                            LEFT JOIN slowest_times ON
                                slowest_times.track_id = track_ids.track_id AND
                                slowest_times.is_lap = track_ids.is_lap
                        )
                    )
                    WHERE 
                        row_n = 1 AND
                        category <= $1 AND
                        date <= $2 AND
                        region_id = ANY($3)
                        {3}
                )
                GROUP BY id
            ) ORDER BY total_time ASC;
            "#,
            Self::table_name(),
            PlayersBasic::table_name(),
            crate::sql::tables::tracks::Tracks::table_name(),
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }

    async fn get_tally_points(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            r#"
            SELECT *, 
            RANK() OVER(ORDER BY tally_points DESC)::INTEGER AS rank FROM (
                SELECT
                SUM(pts)::INT2 AS tally_points,
                MAX(name) AS name, MAX(alias) AS alias,
                MAX(region_id) AS region_id, MAX(id) AS id
                FROM (
                    SELECT *,
                    GREATEST((11 - RANK() OVER(PARTITION BY track_id, is_lap ORDER BY value ASC)), 0) AS pts
                    FROM (
                        SELECT track_id, MAX({1}.id) AS id,
                        MIN(value) AS value, is_lap,
                        MAX(name) AS name, MAX(alias) AS alias,
                        MAX(region_id) AS region_id FROM {0}
                        LEFT JOIN {1} ON {0}.player_id = {1}.id
                        WHERE category <= $1 AND date <= $2 AND region_id = ANY($3) {2}
                        GROUP BY {1}.id, track_id, is_lap
                    ) AS distinct_data
                    ) AS with_points
                GROUP BY id
                ORDER BY tally_points DESC
            );
            "#,
            Self::table_name(),
            PlayersBasic::table_name(),
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }

    async fn get_average_finish(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            r#"
            WITH
                slowest_times AS (
                    SELECT
                        track_id,
                        category,
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
                        FROM {0}
                    )
                    WHERE row_n = 1 AND category <= $1 {3}
                    GROUP BY track_id, category, is_lap
                ),
                track_ids AS (
                    SELECT id AS track_id, t.is_lap FROM {2}
                    CROSS JOIN (VALUES (TRUE),(FALSE)) AS t (is_lap)
                )
            SELECT *,
                (RANK() OVER(ORDER BY average_finish ASC))::INTEGER AS rank
            FROM (
                SELECT
                    id,
                    MAX(name) AS name,
                    MAX(alias) AS alias,
                    MAX(region_id) AS region_id,
                    (SUM(rank)::FLOAT8 / {4}::FLOAT8) AS average_finish
                FROM (
                    SELECT *,
                        RANK() OVER(
                        PARTITION BY
                                track_id,
                                is_lap
                            ORDER BY value
                        ) AS rank
                    FROM (
                        SELECT *,
                            ROW_NUMBER() OVER (
                                PARTITION BY
                                    id,
                                    track_id,
                                    category,
                                    is_lap
                                ORDER BY value
                            ) AS row_n
                        FROM (
                            SELECT
                                COALESCE({0}.value, slowest_times.value) AS value,
                                COALESCE({0}.category, slowest_times.category) AS category,
                                COALESCE({0}.date, '2008-01-01'::DATE) AS date,
                                {1}.id,
                                {1}.name,
                                {1}.alias,
                                {1}.region_id,
                                track_ids.track_id,
                                track_ids.is_lap
                            FROM {1}
                            CROSS JOIN track_ids
                            LEFT JOIN {0} ON
                                {0}.player_id = {1}.id AND
                                {0}.is_lap = track_ids.is_lap AND
                                {0}.track_id = track_ids.track_id AND
                                {0}.category <= $1
                            LEFT JOIN slowest_times ON
                                slowest_times.track_id = track_ids.track_id AND
                                slowest_times.is_lap = track_ids.is_lap
                        )
                    )
                    WHERE 
                        row_n = 1 AND
                        category <= $1 AND
                        date <= $2 AND
                        region_id = ANY($3)
                        {3}
                )
                GROUP BY id
            ) ORDER BY average_finish ASC;
            "#,
            Self::table_name(),
            PlayersBasic::table_name(),
            crate::sql::tables::tracks::Tracks::table_name(),
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
            if is_lap.is_some() { 32 } else { 64 },
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }

    // TODO: Hardcoded Value 33 for Newbie
    async fn get_average_rank_rating(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            r#"
            SELECT *, 
                RANK() OVER(ORDER BY average_rank_rating ASC)::INTEGER AS rank
            FROM (
                SELECT
                    (((({3} - COUNT(*)) * 33) + COALESCE(SUM(value), 0))::FLOAT8 / {3}::FLOAT8) AS average_rank_rating,
                    MAX(name) AS name,
                    MAX(alias) AS alias,
                    MAX(region_id) AS region_id,
                    MAX(id) AS id
                FROM (
                    SELECT *
                    FROM (
                        SELECT
                            ROW_NUMBER() OVER(
                                PARTITION BY
                                    {1}.id,
                                    {0}.track_id,
                                    {0}.category,
                                    {0}.is_lap
                                ORDER BY {0}.value ASC, {5}.value ASC
                            ) AS row_n,
                            {1}.id,
                            {5}.value,
                            name,
                            alias,
                            region_id
                        FROM {0}
                        LEFT JOIN {1} ON
                            {0}.player_id = {1}.id
                        LEFT JOIN {4} ON
                            {0}.track_id = {4}.track_id AND
                            {0}.value <= {4}.value AND
                            {4}.category <= {0}.category AND
                            {4}.is_lap = {0}.is_lap
                        LEFT JOIN {5} ON
                            {5}.id = {4}.standard_level_id
                        WHERE
                            {0}.category <= $1 AND
                            {0}.date <= $2 AND
                            {1}.region_id = ANY($3)
                            {2}
                        ORDER BY value ASC, {5}.value ASC
                    )
                    WHERE row_n = 1
                ) AS distinct_data
                GROUP BY id
                ORDER BY average_rank_rating ASC
            );
            "#,
            Self::table_name(),
            PlayersBasic::table_name(),
            if is_lap.is_some() {
                "AND {0}.is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
            if is_lap.is_some() { 32 } else { 64 },
            crate::sql::tables::standards::Standards::table_name(),
            crate::sql::tables::standard_levels::StandardLevels::table_name(),
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
#[serde(untagged)]
pub enum RankingType {
    AverageFinish(f64),
    TotalTime(i32),
    TallyPoints(i16),
    AverageRankRating(f64),
    PersonalRecordWorldRecord(f64),
}

impl TryInto<f64> for RankingType {
    type Error = ();
    fn try_into(self) -> Result<f64, Self::Error> {
        return match self {
            Self::AverageFinish(x) => Ok(x),
            Self::AverageRankRating(x) => Ok(x),
            Self::PersonalRecordWorldRecord(x) => Ok(x),
            _ => Err(()),
        };
    }
}

impl TryInto<i32> for RankingType {
    type Error = ();
    fn try_into(self) -> Result<i32, Self::Error> {
        return match self {
            Self::TotalTime(x) => Ok(x),
            _ => Err(()),
        };
    }
}

impl TryInto<i16> for RankingType {
    type Error = ();
    fn try_into(self) -> Result<i16, Self::Error> {
        return match self {
            Self::TallyPoints(x) => Ok(x),
            _ => Err(()),
        };
    }
}
