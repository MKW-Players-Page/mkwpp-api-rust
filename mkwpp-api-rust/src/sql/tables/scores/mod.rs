pub mod by_date;
pub mod matchup;
pub mod rankings;
pub mod slowest_times;
pub mod timesets;
pub mod timesheet;
pub mod with_player;

use crate::sql::tables::players::players_basic::PlayersBasic;

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
    pub ScoresByDate: [
        player: PlayersBasic,
        video_link: (),
        ghost_link: (),
        comment: (),
        initial_rank: ()
    ],
    pub SlowestTimes: [
        id: (),
        date: (),
        category: (),
        video_link: (),
        ghost_link: (),
        comment: (),
        initial_rank: ()
    ],
    pub RankingsScoresData: [
        id: (),
        category: (),
        player_id: i32,
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
    pub track_id: either_field::either!(i32 | ()),
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
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO scores (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);").bind(self.id).bind(self.value).bind(&self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(self.initial_rank).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO scores (id, value, category, is_lap, player_id, track_id, date, video_link, ghost_link, comment, admin_note, initial_rank) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (id) DO UPDATE SET value = $2, category = $3, is_lap = $4, player_id = $5, track_id = $6, date = $7, video_link = $8, ghost_link = $9, comment = $10, admin_note = $11, initial_rank = $12 WHERE scores.id = $1;").bind(self.id).bind(self.value).bind(self.category).bind(self.is_lap).bind(self.player_id).bind(self.track_id).bind(self.date).bind(&self.video_link).bind(&self.ghost_link).bind(&self.comment).bind(&self.admin_note).bind(self.initial_rank).execute(executor).await;
    }
}
