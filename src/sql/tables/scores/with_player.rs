use crate::sql::tables::players::players_basic::PlayersBasic;
use crate::sql::tables::BasicTableQueries;
use sqlx::{FromRow, Row};

#[serde_with::skip_serializing_none]
#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ScoresWithPlayer {
    pub rank: Option<i32>,
    pub prwr: Option<f64>,
    pub std_lvl_code: String,
    pub id: i32,
    pub value: i32,
    pub category: crate::sql::tables::Category,
    pub is_lap: bool,
    pub player: PlayersBasic,
    pub track_id: i32,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub initial_rank: Option<i32>,
}

impl<'a> FromRow<'a, sqlx::postgres::PgRow> for ScoresWithPlayer {
    fn from_row(row: &'a sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            rank: row.try_get("rank").unwrap_or(None),
            prwr: row.try_get("prwr").unwrap_or(None),
            id: row.try_get("s_id")?,
            value: row.try_get("value")?,
            category: row.try_get("category")?,
            std_lvl_code: row.try_get("code")?,
            is_lap: row.try_get("is_lap")?,
            player: PlayersBasic {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                alias: row.try_get("alias")?,
                region_id: row.try_get("region_id")?,
            },
            track_id: row.try_get("track_id")?,
            date: row.try_get("date")?,
            video_link: row.try_get("video_link")?,
            ghost_link: row.try_get("ghost_link")?,
            comment: row.try_get("comment")?,
            initial_rank: row.try_get("initial_rank")?,
        })
    }
}

impl BasicTableQueries for ScoresWithPlayer {
    fn table_name() -> &'static str {
        return super::Scores::table_name();
    }

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
            "SELECT {0}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {1}.id, name, alias, region_id FROM {0} LEFT JOIN {1} ON {0}.player_id = {1}.id;",
            super::Scores::table_name(),
            PlayersBasic::table_name(),
        ))
        .fetch_all(executor)
        .await;
    }
}

impl ScoresWithPlayer {
    pub async fn order_by_date(
        executor: &mut sqlx::PgConnection,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
                "SELECT {0}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {1}.id, name, alias, region_id FROM {0} LEFT JOIN {1} ON {0}.player_id = {1}.id WHERE date IS NOT NULL ORDER BY date DESC LIMIT $1;",
                super::Scores::table_name(),
                PlayersBasic::table_name(),
            )).bind(limit)
            .fetch_all(executor)
            .await;
    }

    pub async fn order_records_by_date(
        executor: &mut sqlx::PgConnection,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
                    "SELECT {0}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {1}.id, name, alias, region_id FROM {0} LEFT JOIN {1} ON {0}.player_id = {1}.id WHERE date IS NOT NULL AND initial_rank = 1 ORDER BY date DESC LIMIT $1;",
                    super::Scores::table_name(),
                    PlayersBasic::table_name(),
                )).bind(limit)
                .fetch_all(executor)
                .await;
    }

    // TODO: Hardcoded value for Newbie Code
    pub async fn filter_charts(
        executor: &mut sqlx::PgConnection,
        track_id: i32,
        category: crate::sql::tables::Category,
        is_lap: bool,
        max_date: chrono::NaiveDate,
        region_id: i32,
        limit: i32,
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
                        (RANK() OVER(ORDER BY value ASC))::INTEGER AS rank,
                        ((FIRST_VALUE(value) OVER(ORDER BY value ASC))::FLOAT8 / value::FLOAT8) AS prwr
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
                                    PARTITION BY {1}.id
                                    ORDER BY {0}.value ASC, {3}.value ASC
                                ) AS row_n,
                                date,
                                video_link,
                                ghost_link,
                                comment,
                                initial_rank,
                                {1}.id,
                                COALESCE({3}.code, 'NW') AS code,
                                name,
                                alias,
                                region_id FROM {0}
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
                                {0}.track_id = $1 AND
                                {0}.category <= $2 AND
                                {0}.is_lap = $3 AND
                                {0}.date <= $4 AND
                                {1}.region_id = ANY($5) 
                            ORDER BY value ASC, {3}.value ASC
                        ) WHERE row_n = 1
                    ) ORDER BY value ASC, date DESC
                ) WHERE rank <= $6;
                "#,
            super::Scores::table_name(),
            PlayersBasic::table_name(),
            crate::sql::tables::standards::Standards::table_name(),
            crate::sql::tables::standard_levels::StandardLevels::table_name(),
        ))
        .bind(track_id)
        .bind(category)
        .bind(is_lap)
        .bind(max_date)
        .bind(region_ids)
        .bind(limit)
        .fetch_all(executor)
        .await;
    }

    // TODO: Hardcoded value for Newbie Code
    pub async fn get_records(
        executor: &mut sqlx::PgConnection,
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
                SELECT *,
                    1::INTEGER AS rank,
                    1::FLOAT8 AS prwr
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
                                PARTITION BY {0}.track_id, {0}.is_lap
                                ORDER BY {0}.value ASC, {3}.value ASC
                            ) AS row_n,
                            date,
                            video_link,
                            ghost_link,
                            comment,
                            initial_rank,
                            {1}.id,
                            COALESCE({3}.code, 'NW') AS code,
                            name,
                            alias,
                            region_id FROM {0}
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
                            {0}.category <= $1 AND
                            {0}.date <= $2 AND
                            {1}.region_id = ANY($3) 
                            {4}
                        ORDER BY value ASC, {3}.value ASC
                    ) WHERE row_n = 1
                ) ORDER BY track_id ASC, is_lap ASC;
                "#,
            super::Scores::table_name(),
            PlayersBasic::table_name(),
            crate::sql::tables::standards::Standards::table_name(),
            crate::sql::tables::standard_levels::StandardLevels::table_name(),
            if is_lap.is_some() {
                "AND is_lap = $4"
            } else {
                ""
            }
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await;
    }
}
