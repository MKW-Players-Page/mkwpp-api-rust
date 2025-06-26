use crate::api::errors::{EveryReturnedError, FinalErrorResponse};
use crate::sql::tables::BasicTableQueries;
use crate::sql::tables::players::players_basic::PlayersBasic;

pub use super::ScoresWithPlayer;

impl BasicTableQueries for ScoresWithPlayer {
    const TABLE_NAME: &'static str = super::Scores::TABLE_NAME;

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(const_format::formatc!(
            "SELECT {scores_table}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {players_table}.id, name, alias, region_id FROM {scores_table} LEFT JOIN {players_table} ON {scores_table}.player_id = {players_table}.id;",
            scores_table = super::Scores::TABLE_NAME,
            players_table = PlayersBasic::TABLE_NAME,
        ))
        .fetch_all(executor)
        .await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}

impl ScoresWithPlayer {
    // TODO: Hardcoded value for Newbie Code
    pub async fn filter_charts(
        executor: &mut sqlx::PgConnection,
        track_id: i32,
        category: crate::sql::tables::Category,
        is_lap: bool,
        max_date: chrono::NaiveDate,
        region_id: i32,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

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
                                COALESCE({3}.code, 'NW') AS std_lvl_code,
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
            super::Scores::TABLE_NAME,
            PlayersBasic::TABLE_NAME,
            crate::sql::tables::standards::Standards::TABLE_NAME,
            crate::sql::tables::standard_levels::StandardLevels::TABLE_NAME,
        ))
        .bind(track_id)
        .bind(category)
        .bind(is_lap)
        .bind(max_date)
        .bind(region_ids)
        .bind(limit)
        .fetch_all(executor)
        .await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    // TODO: Hardcoded value for Newbie Code
    pub async fn get_records(
        executor: &mut sqlx::PgConnection,
        category: crate::sql::tables::Category,
        is_lap: Option<bool>,
        max_date: chrono::NaiveDate,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        return sqlx::query(&format!(
            r#"
                SELECT *,
                    1::INTEGER AS rank,
                    1::FLOAT8 AS prwr
                FROM (
                    SELECT *
                    FROM (
                        SELECT
                            {scores_table}.id AS s_id,
                            {scores_table}.value,
                            {scores_table}.category,
                            {scores_table}.is_lap,
                            {scores_table}.track_id,
                            ROW_NUMBER() OVER(
                                PARTITION BY {scores_table}.track_id, {scores_table}.is_lap
                                ORDER BY {scores_table}.value ASC, {standard_level_table}.value ASC
                            ) AS row_n,
                            date,
                            video_link,
                            ghost_link,
                            comment,
                            initial_rank,
                            {players_table}.id,
                            COALESCE({standard_level_table}.code, 'NW') AS std_lvl_code,
                            name,
                            alias,
                            region_id FROM {scores_table}
                        LEFT JOIN {players_table} ON
                        {scores_table}.player_id = {players_table}.id
                        LEFT JOIN {standards_table} ON
                            {scores_table}.track_id = {standards_table}.track_id AND
                            {scores_table}.value <= {standards_table}.value AND
                            {standards_table}.category <= {scores_table}.category AND
                            {standards_table}.is_lap = {scores_table}.is_lap
                        LEFT JOIN {standard_level_table} ON
                            {standard_level_table}.id = {standards_table}.standard_level_id
                        WHERE
                            {scores_table}.category <= $1 AND
                            {scores_table}.date <= $2 AND
                            {players_table}.region_id = ANY($3) 
                            {is_lap_where}
                        ORDER BY value ASC, {standard_level_table}.value ASC
                    ) WHERE row_n = 1
                ) ORDER BY track_id ASC, is_lap ASC;
                "#,
            scores_table = super::Scores::TABLE_NAME,
            players_table = PlayersBasic::TABLE_NAME,
            standards_table = crate::sql::tables::standards::Standards::TABLE_NAME,
            standard_level_table = crate::sql::tables::standard_levels::StandardLevels::TABLE_NAME,
            is_lap_where = if is_lap.is_some() {
                const_format::formatc!(
                    "AND {scores_table}.is_lap = $4",
                    scores_table = super::Scores::TABLE_NAME,
                )
            } else {
                ""
            }
        ))
        .bind(category)
        .bind(max_date)
        .bind(region_ids)
        .bind(is_lap)
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
