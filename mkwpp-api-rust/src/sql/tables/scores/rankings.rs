pub use super::{RankingsScoresData, timesets::ValidTimesetItem};
use crate::sql::tables::{
    BasicTableQueries,
    players::{FilterPlayers, players_basic::PlayersBasic},
    scores::timesets::Timeset,
};
use sqlx::{FromRow, Row};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rankings {
    pub rank: i32,
    pub value: RankingType,
    pub player: PlayersBasic,
}

impl BasicTableQueries for Rankings {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;
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

        Ok(Rankings {
            rank: row.try_get("rank")?,
            player: PlayersBasic {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                alias: row.try_get("alias").ok(),
                region_id: row.try_get("region_id")?,
            },
            value: data,
        })
    }
}

impl ValidTimesetItem for RankingsScoresData {
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
}

impl Rankings {
    pub async fn get(
        executor: &mut sqlx::PgConnection,
        ranking_type: RankingType,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<Rankings>, anyhow::Error> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        match ranking_type {
            RankingType::AverageFinish(_) => {
                Self::get_average_finish(executor, category, is_lap, max_date, region_ids).await
            }
            _ => todo!(),
        }
    }

    pub async fn get_old(
        executor: &mut sqlx::PgConnection,
        ranking_type: RankingType,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        match ranking_type {
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
            _ => todo!(),
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
            Self::TABLE_NAME,
            PlayersBasic::TABLE_NAME,
            crate::sql::tables::tracks::Tracks::TABLE_NAME,
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
                (RANK() OVER(ORDER BY personal_record_world_record DESC))::INTEGER AS rank
            FROM (
                SELECT
                    id,
                    MAX(name) AS name,
                    MAX(alias) AS alias,
                    MAX(region_id) AS region_id,
                    (SUM(prwr)::FLOAT8 / {4}::FLOAT8) AS personal_record_world_record
                FROM (
                    SELECT *,
                        ((FIRST_VALUE(value) OVER(
                            PARTITION BY track_id, is_lap ORDER BY value ASC
                        ))::FLOAT8 / value::FLOAT8) AS prwr
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
            ) ORDER BY personal_record_world_record DESC;
            "#,
            Self::TABLE_NAME,
            PlayersBasic::TABLE_NAME,
            crate::sql::tables::tracks::Tracks::TABLE_NAME,
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }, // TODO: this is shit
            if is_lap.is_some() { 32 } else { 64 }
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
            Self::TABLE_NAME,
            PlayersBasic::TABLE_NAME,
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
    ) -> Result<Vec<Rankings>, anyhow::Error> {
        let mut players = PlayersBasic::get_players_by_region_ids(executor, region_ids)
            .await?
            .into_iter()
            .map(|player_row| PlayersBasic::from_row(&player_row))
            .collect::<Result<Vec<PlayersBasic>, sqlx::Error>>()?;

        let player_ids = players.iter().map(|x| x.id).collect::<Vec<i32>>();

        let timeset = sqlx::query(&format!(
            r#"
            SELECT
                value, category, is_lap, track_id, player_id
            FROM (
                SELECT
                    value,
                    category,
                    is_lap,
                    track_id,
                    player_id,
                    date,
                    ROW_NUMBER() OVER(
                        PARTITION BY player_id, track_id, is_lap
                        ORDER BY value ASC, date DESC
                    ) AS row_n
                FROM {this_table}
                WHERE
                    category <= $1 AND
                    date <= $3 AND
                    player_id = ANY($4)
                    {is_lap}
                ORDER BY value ASC
            )
            WHERE row_n = 1
            ORDER BY track_id ASC, is_lap ASC, value ASC, date DESC;
            "#,
            this_table = super::Scores::TABLE_NAME,
            is_lap = if is_lap.is_some() {
                format!("AND is_lap = $2")
            } else {
                String::new()
            }
        ))
        .bind(category)
        .bind(is_lap)
        .bind(max_date)
        .bind(&player_ids)
        .fetch_all(executor)
        .await?
        .into_iter()
        .map(|score_row| RankingsScoresData::from_row(&score_row))
        .collect::<Result<Vec<RankingsScoresData>, sqlx::Error>>()?;

        let mut timeset_encoder = Timeset::default();
        timeset_encoder.timeset = timeset;
        timeset_encoder.filters.category = category;
        timeset_encoder.filters.is_lap = is_lap;
        timeset_encoder.filters.max_date = max_date;
        timeset_encoder.filters.player_ids = player_ids;
        timeset_encoder
            .calculate_average_finish_charts()
            .await
            .map(|value| {
                let mut value = value
                    .into_iter()
                    .map(|(rank, player_id, af)| Rankings {
                        player: {
                            let player_ref = players
                                .iter_mut()
                                .find(|player| player.id == player_id)
                                .unwrap();
                            std::mem::take(player_ref)
                        },
                        rank,
                        value: RankingType::AverageFinish(af),
                    })
                    .collect::<Vec<Rankings>>();
                value.sort_by(|x, y| x.rank.cmp(&y.rank));
                value
            })
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
                                    {0}.is_lap
                                ORDER BY {0}.value ASC, {0}.date DESC, {5}.value ASC
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
            Self::TABLE_NAME,
            PlayersBasic::TABLE_NAME,
            if is_lap.is_some() {
                format!("AND {0}.is_lap = $4", Self::TABLE_NAME)
            } else {
                String::new()
            }, // TODO: this is shit
            if is_lap.is_some() { 32 } else { 64 },
            crate::sql::tables::standards::Standards::TABLE_NAME,
            crate::sql::tables::standard_levels::StandardLevels::TABLE_NAME,
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
        match self {
            Self::AverageFinish(x) => Ok(x),
            Self::AverageRankRating(x) => Ok(x),
            Self::PersonalRecordWorldRecord(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl TryInto<i32> for RankingType {
    type Error = ();
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Self::TotalTime(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl TryInto<i16> for RankingType {
    type Error = ();
    fn try_into(self) -> Result<i16, Self::Error> {
        match self {
            Self::TallyPoints(x) => Ok(x),
            _ => Err(()),
        }
    }
}
