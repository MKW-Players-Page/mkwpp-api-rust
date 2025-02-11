use crate::sql::tables::players::Players;
use crate::sql::tables::BasicTableQueries;

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow)]
pub struct ScoresWithPlayer {
    #[sqlx(rename = "s_id")]
    pub id: i32,
    pub value: i32,
    pub category: crate::sql::tables::Category,
    pub is_lap: bool,
    #[sqlx(flatten)]
    pub player: crate::sql::tables::players::players_basic::PlayersBasic,
    pub track_id: i32,
    pub date: Option<chrono::NaiveDate>,
    pub video_link: Option<String>,
    pub ghost_link: Option<String>,
    pub comment: Option<String>,
    pub initial_rank: Option<i32>,
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
            Players::table_name(),
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
                "SELECT {0}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {1}.id, name, alias, region_id FROM {0} LEFT JOIN {1} ON {0}.player_id = {1}.id WHERE date IS NOT NULL ORDER BY date desc LIMIT $1",
                super::Scores::table_name(),
                Players::table_name(),
                
            )).bind(limit)
            .fetch_all(executor)
            .await;
    }

    pub async fn order_records_by_date(
        executor: &mut sqlx::PgConnection,
        limit: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!(
                    "SELECT {0}.id AS s_id, value, category, is_lap, track_id, date, video_link, ghost_link, comment, initial_rank, {1}.id, name, alias, region_id FROM {0} LEFT JOIN {1} ON {0}.player_id = {1}.id WHERE date IS NOT NULL AND initial_rank = 1 ORDER BY date desc LIMIT $1",
                    super::Scores::table_name(),
                    Players::table_name(),
                    
                )).bind(limit)
                .fetch_all(executor)
                .await;
    }
}
