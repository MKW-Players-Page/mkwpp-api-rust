pub mod by_date;
pub mod country_rankings;
pub mod matchup;
pub mod rankings;
pub mod timesets;
pub mod timesheet;
pub mod with_player;

use crate::{
    api::errors::{EveryReturnedError, FinalErrorResponse},
    custom_serde::DateAsTimestampNumber,
    sql::tables::{BasicTableQueries, players::players_basic::PlayersBasic},
};

#[either_field::make_template(
    GenStructs: true,
    DeleteTemplate: true,
    OmitEmptyTupleFields: true;
    pub Scores: [
        player_id: i32,
        admin_note: Option<String>
    ],
    pub ScoresWithPlayer: [
        player: PlayersBasic,
        rank: i32,
        prwr: f64,
        std_lvl_code: String
    ],
    pub Times: [
        rank: i32,
        prwr: f64,
        std_lvl_code: String
    ],
    pub TimesheetTimesetData: [
        player_id: i32
    ],
    pub ScoresByDate: [
        player: PlayersBasic,
        video_link: (),
        ghost_link: (),
        comment: (),
        initial_rank: ()
    ],
    pub RankingsTimesetData: [
        id: (),
        category: (),
        player_id: i32,
        date: (),
        initial_rank: (),
        video_link: (),
        ghost_link: (),
        comment: ()
    ],
    pub CountryRankingsTimesetData: [
        id: (),
        category: (),
        player_id: i32,
        region_id: i32,
        date: (),
        initial_rank: (),
        video_link: (),
        ghost_link: (),
        comment: ()
    ]
)]
#[serde_with::skip_serializing_none]
#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoresTemplate {
    pub id: either_field::either!(i32 | ()),
    pub value: either_field::either!(i32 | ()),
    pub rank: either_field::either!(() | i32),
    pub prwr: either_field::either!(() | f64),
    pub std_lvl_code: either_field::either!(() | String),
    pub category: either_field::either!(super::Category | ()),
    pub is_lap: either_field::either!(bool | ()),
    #[sqlx(flatten)]
    pub player: either_field::either!(() | PlayersBasic),
    pub player_id: either_field::either!(() | i32),
    pub region_id: either_field::either!(() | i32),
    pub track_id: either_field::either!(i32 | ()),
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub date: either_field::either!(Option<chrono::NaiveDate> | ()),
    pub video_link: either_field::either!(Option<String> | ()),
    pub ghost_link: either_field::either!(Option<String> | ()),
    pub comment: either_field::either!(Option<String> | ()),
    pub admin_note: either_field::either!(() | Option<String>),
    pub initial_rank: either_field::either!(Option<i32> | ()),
}

impl super::BasicTableQueries for Scores {
    const TABLE_NAME: &'static str = "scores";
}

impl Scores {
    pub async fn filter_by_track(
        track_id: i32,

        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query("SELECT * FROM scores WHERE track_id = $1")
            .bind(track_id)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn insert_or_edit(
        id: Option<i32>,
        value: i32,
        category: super::Category,
        is_lap: bool,
        player_id: i32,
        track_id: i32,
        date: Option<chrono::NaiveDate>,
        video_link: Option<String>,
        ghost_link: Option<String>,
        comment: Option<String>,
        admin_note: Option<String>,
        initial_rank: Option<i32>,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        match id {
            None => {
                sqlx::query(const_format::formatcp!("INSERT INTO {table_name} (value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);", table_name = Scores::TABLE_NAME))
            }
            Some(id) => {
                sqlx::query(const_format::formatcp!("UPDATE {table_name} SET (value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) = ($2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) WHERE id = $1;", table_name = Scores::TABLE_NAME)).bind(id)

            }
        }
        .bind(value)
        .bind(category)
        .bind(is_lap)
        .bind(player_id)
        .bind(track_id)
        .bind(date)
        .bind(video_link)
        .bind(ghost_link)
        .bind(comment)
        .bind(admin_note)
        .bind(initial_rank)
        .execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e))
    }

    pub async fn get_from_id(
        id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgRow, FinalErrorResponse> {
        return sqlx::query("SELECT * FROM scores WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
