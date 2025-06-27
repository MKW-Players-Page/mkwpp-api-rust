pub use super::{RankingsTimesetData, timesets::ValidTimesetItem};
use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::decode_rows_to_table,
    },
    sql::tables::{
        BasicTableQueries, Category,
        players::{FilterPlayers, players_basic::PlayersBasic},
        scores::timesets::Timeset,
    },
};

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

impl ValidTimesetItem for RankingsTimesetData {
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
    fn get_date(&self) -> Option<chrono::NaiveDate> {
        None
    }
    fn get_comment(&self) -> Option<String> {
        None
    }
    fn get_time_id(&self) -> i32 {
        0
    }
    fn get_category(&self) -> crate::sql::tables::Category {
        Category::NonSc
    }
    fn get_ghost_link(&self) -> Option<String> {
        None
    }
    fn get_video_link(&self) -> Option<String> {
        None
    }
    fn get_initial_rank(&self) -> Option<i32> {
        None
    }
    fn get_player_region_id(&self) -> i32 {
        0
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
    ) -> Result<Vec<Rankings>, FinalErrorResponse> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        let mut players = decode_rows_to_table::<PlayersBasic>(
            PlayersBasic::get_players_by_region_ids(executor, region_ids).await?,
        )?;

        let player_ids = players.iter().map(|x| x.id).collect::<Vec<i32>>();

        let timeset = decode_rows_to_table::<RankingsTimesetData>(
            sqlx::query(&format!(
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
                    "AND is_lap = $2".to_string()
                } else {
                    String::new()
                }
            ))
            .bind(category)
            .bind(is_lap)
            .bind(max_date)
            .bind(&player_ids)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?,
        )?;

        let mut timeset_encoder = Timeset::default();
        timeset_encoder.timeset = timeset;
        timeset_encoder.filters.category = category;
        timeset_encoder.filters.is_lap = is_lap;
        timeset_encoder.filters.max_date = max_date;
        timeset_encoder.filters.player_ids = player_ids;
        timeset_encoder.filters.whitelist_player_ids = region_id != 1;

        let data = match ranking_type {
            RankingType::AverageFinish(_) => {
                timeset_encoder.calculate_average_finish_charts().await
            }
            RankingType::TotalTime(_) => timeset_encoder.calculate_total_time_charts().await,
            RankingType::PersonalRecordWorldRecord(_) => {
                timeset_encoder
                    .calculate_personal_record_world_record_charts()
                    .await
            }
            RankingType::TallyPoints(_) => timeset_encoder.calculate_tally_points_charts().await,
            RankingType::AverageRankRating(_) => {
                timeset_encoder.calculate_average_rank_rating_charts().await
            }
        };

        data.map(|value| {
            let mut value = value
                .into_iter()
                .map(|(rank, player_id, value)| Rankings {
                    player: {
                        let player_ref = players
                            .iter_mut()
                            .find(|player| player.id == player_id)
                            .unwrap();
                        std::mem::take(player_ref)
                    },
                    rank,
                    value,
                })
                .collect::<Vec<Rankings>>();
            value.sort_by(|x, y| x.rank.cmp(&y.rank));
            value
        })
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
