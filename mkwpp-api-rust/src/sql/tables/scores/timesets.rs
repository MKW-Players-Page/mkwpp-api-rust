use anyhow::anyhow;
use chrono::NaiveDate;

use crate::{
    app_state::access_app_state,
    sql::tables::{Category, standard_levels::StandardLevels},
};

pub struct Timeset<K: ValidTimesetItem> {
    pub timeset: Vec<K>,
    pub filters: TimesetFilters,
    pub set_ranks_on_times: bool,
    pub set_prwr_on_times: bool,
    output: TimesetOutput,
    divvie_value: f64,
}

#[derive(Default)]
pub struct TimesetFilters {
    pub is_lap: Option<bool>,
    pub track_ids: Vec<i32>,
    pub player_ids: Vec<i32>,
    pub whitelist_player_ids: bool,
    pub category: Category,
    pub region_id: i32,
    pub max_date: NaiveDate,
}

enum TimesetOutput {
    None,
    AverageFinishCharts {
        out: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    TotalTimeCharts {
        out: Vec<Option<i32>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    PersonalRecordWorldRecordCharts {
        out: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    TallyPointsCharts {
        out: Vec<Option<i16>>,
        players_found: Vec<bool>,
    },
    AverageRankRatingCharts {
        out: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
}

pub trait ValidTimesetItem {
    fn get_time(&self) -> i32;
    fn get_track_id(&self) -> i32;
    fn get_is_lap(&self) -> bool;
    fn get_player_id(&self) -> i32;
    fn set_rank(&mut self, rank: i32);
    fn set_prwr(&mut self, prwr: f64);
}

impl<K: ValidTimesetItem> Default for Timeset<K> {
    fn default() -> Self {
        Self {
            timeset: Vec::new(),
            filters: TimesetFilters::default(),
            set_ranks_on_times: false,
            set_prwr_on_times: false,
            output: TimesetOutput::None,
            divvie_value: 64.0,
        }
    }
}

impl<K: ValidTimesetItem> Timeset<K> {
    pub async fn calculate_average_finish_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, f64)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        let mut out = vec![None; reserve_space];
        for player_id in &self.filters.player_ids {
            out[*player_id as usize] = Some(0.0)
        }

        self.output = TimesetOutput::AverageFinishCharts {
            out,
            players_found: vec![false; reserve_space],
            players_found_counter: 0,
        };

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::AverageFinishCharts { out, .. } => {
                let mut af_and_ids = out
                    .iter()
                    .enumerate()
                    .filter_map(|(id, rank_sum)| {
                        rank_sum.map(|rank_sum| ((id as i32), rank_sum / self.divvie_value))
                    })
                    .collect::<Vec<(i32, f64)>>();
                af_and_ids.sort_by(|(_id1, af1), (_id2, af2)| af1.total_cmp(&af2));
                Ok(af_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, af))| ((ranking as i32) + 1, id, af))
                    .collect())
            }
            _ => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn calculate_total_time_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, i32)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        let mut out = vec![None; reserve_space];
        for player_id in &self.filters.player_ids {
            out[*player_id as usize] = Some(0)
        }

        self.output = TimesetOutput::TotalTimeCharts {
            out,
            players_found: vec![false; reserve_space],
            players_found_counter: 0,
        };

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::TotalTimeCharts { out, .. } => {
                let mut tt_and_ids = out
                    .iter()
                    .enumerate()
                    .filter_map(|(id, time_sum)| time_sum.map(|time_sum| ((id as i32), time_sum)))
                    .collect::<Vec<(i32, i32)>>();
                tt_and_ids.sort_by(|(_id1, tt1), (_id2, tt2)| tt1.cmp(&tt2));
                Ok(tt_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, tt))| ((ranking as i32) + 1, id, tt))
                    .collect())
            }
            _ => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn calculate_tally_points_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, i16)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        let mut out = vec![None; reserve_space];
        for player_id in &self.filters.player_ids {
            out[*player_id as usize] = Some(0)
        }

        self.output = TimesetOutput::TallyPointsCharts {
            players_found: vec![false; reserve_space],
            out,
        };

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::TallyPointsCharts {
                out,
                players_found: _,
            } => {
                let mut tp_and_ids = out
                    .iter()
                    .enumerate()
                    .filter_map(|(id, points_sum)| {
                        points_sum.map(|points_sum| ((id as i32), points_sum))
                    })
                    .collect::<Vec<(i32, i16)>>();
                tp_and_ids.sort_by(|(_id1, tp1), (_id2, tp2)| tp2.cmp(&tp1));
                Ok(tp_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, tp))| ((ranking as i32) + 1, id, tp))
                    .collect())
            }
            _ => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn calculate_personal_record_world_record_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, f64)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        let mut out = vec![None; reserve_space];
        for player_id in &self.filters.player_ids {
            out[*player_id as usize] = Some(0.0)
        }

        self.output = TimesetOutput::PersonalRecordWorldRecordCharts {
            out,
            players_found: vec![false; reserve_space],
            players_found_counter: 0,
        };

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::PersonalRecordWorldRecordCharts { out, .. } => {
                let mut prwr_and_ids = out
                    .iter()
                    .enumerate()
                    .filter_map(|(id, prwr_sum)| {
                        prwr_sum.map(|prwr_sum| ((id as i32), prwr_sum / self.divvie_value))
                    })
                    .collect::<Vec<(i32, f64)>>();
                prwr_and_ids.sort_by(|(_id1, prwr1), (_id2, prwr2)| prwr2.total_cmp(&prwr1));
                Ok(prwr_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, prwr))| ((ranking as i32) + 1, id, prwr))
                    .collect())
            }
            _ => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn calculate_average_rank_rating_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, f64)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        let mut out = vec![None; reserve_space];
        for player_id in &self.filters.player_ids {
            out[*player_id as usize] = Some(0.0)
        }

        self.output = TimesetOutput::AverageRankRatingCharts {
            out,
            players_found: vec![false; reserve_space],
            players_found_counter: 0,
        };

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::AverageRankRatingCharts { out, .. } => {
                let mut arr_and_ids = out
                    .iter()
                    .enumerate()
                    .filter_map(|(id, arr_sum)| {
                        arr_sum.map(|arr_sum| ((id as i32), arr_sum / self.divvie_value))
                    })
                    .collect::<Vec<(i32, f64)>>();
                arr_and_ids.sort_by(|(_id1, arr1), (_id2, arr2)| arr1.total_cmp(&arr2));
                Ok(arr_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, arr))| ((ranking as i32) + 1, id, arr))
                    .collect())
            }
            _ => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    async fn invert_blacklist(&mut self) -> Result<(), anyhow::Error> {
        if self.filters.whitelist_player_ids {
            return Ok(());
        }

        let mut executor = access_app_state()
            .await
            .read()
            .unwrap()
            .acquire_pg_connection()
            .await?;

        self.filters.player_ids = crate::sql::tables::players::Players::get_ids_but_list(
            &mut executor,
            &self.filters.player_ids,
        )
        .await
        .map_err(|e| anyhow!("Couldn't get the player ids from the list. {e}"))?;
        self.filters.whitelist_player_ids = true;

        Ok(())
    }

    async fn core_loop(&mut self) -> Result<(), anyhow::Error> {
        let app_state = crate::app_state::access_app_state().await;
        let mut app_state = app_state.write().unwrap();

        let standard_levels = app_state.get_legacy_standard_levels().await?;
        let standards = app_state.get_standards().await?;

        std::mem::drop(app_state);

        self.divvie_value = match self.filters.is_lap {
            Some(_) => 32.0,
            None => 64.0,
        };

        let mut last_track = 0;
        let mut last_lap_type = false;
        let mut last_time = 0;
        let mut last_rank: i32 = 0;
        // TODO: Hardcoded Newbie Value
        let mut last_standard_level: StandardLevels = StandardLevels {
            id: 34,
            code: String::from("NW"),
            value: 33,
            is_legacy: true,
        };
        let mut wr_time = 0;

        let mut has_found_all_times = true;

        for time_data in &mut self.timeset {
            // "Category" check to reset last values
            // This is true whenever you're in the first time for the (track + lap type)
            if last_track != time_data.get_track_id() || last_lap_type != time_data.get_is_lap() {
                // Fill in values for players which have not been found
                if !has_found_all_times {
                    match &mut self.output {
                        TimesetOutput::AverageFinishCharts {
                            out,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let rank = (last_rank + 1) as f64;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += rank;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TotalTimeCharts {
                            out,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let time = last_time + 1;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += time;
                                    }
                                }
                            }
                        }
                        TimesetOutput::PersonalRecordWorldRecordCharts {
                            out,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let prwr = (wr_time as f64) / ((last_time + 1) as f64);
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += prwr;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TallyPointsCharts { out, players_found }
                            if last_rank < 11 =>
                        {
                            let pts = 11 - (last_rank as i16);
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += pts;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TallyPointsCharts { .. } => (),

                        TimesetOutput::AverageRankRatingCharts {
                            out,
                            players_found,
                            players_found_counter: _,
                        } => {
                            // TODO: Hardcoded Newbie Value
                            let arr_value = if last_standard_level.id == 34 {
                                last_standard_level.value
                            } else {
                                standard_levels
                                    .iter()
                                    .find(|standard_level| {
                                        standard_level.id
                                            == standards
                                                .iter()
                                                .find(|standard| match standard.value {
                                                    None => false,
                                                    Some(value) => {
                                                        standard.is_lap == last_lap_type
                                                            && standard.track_id == last_track
                                                            && standard.category
                                                                <= self.filters.category
                                                            && value >= last_time + 1
                                                    }
                                                })
                                                .map(|standard| standard.standard_level_id)
                                                .unwrap_or(34)
                                    })
                                    .unwrap()
                                    .value
                            } as f64;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += arr_value;
                                    }
                                }
                            }
                        }
                        TimesetOutput::None => (),
                    }
                }

                // Reset values
                wr_time = time_data.get_time();
                last_track = time_data.get_track_id();
                last_lap_type = time_data.get_is_lap();
                last_rank = 0;
                last_time = 0;

                match &mut self.output {
                    TimesetOutput::AverageFinishCharts {
                        out: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::TotalTimeCharts {
                        out: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::PersonalRecordWorldRecordCharts {
                        out: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::AverageRankRatingCharts {
                        out: _,
                        players_found,
                        players_found_counter,
                    } => {
                        *players_found = vec![false; players_found.len()];
                        *players_found_counter = 0;
                    }
                    TimesetOutput::TallyPointsCharts { out, players_found } => {
                        *players_found = vec![false; players_found.len()];
                    }
                    TimesetOutput::None => (),
                }
                has_found_all_times = false;
            }

            // Skip if has found all times
            if has_found_all_times {
                continue;
            }

            // Set all the last values
            if last_time != time_data.get_time() {
                last_rank += 1;
            }

            if self.set_ranks_on_times {
                time_data.set_rank(last_rank);
            }

            last_time = time_data.get_time();

            last_standard_level = standard_levels
                .iter()
                .find(|standard_level| {
                    standard_level.id
                        == standards
                            .iter()
                            .find(|standard| match standard.value {
                                None => false,
                                Some(value) => {
                                    standard.is_lap == time_data.get_is_lap()
                                        && standard.track_id == time_data.get_track_id()
                                        && standard.category <= self.filters.category
                                        && value >= time_data.get_time()
                                }
                            })
                            .map(|standard| standard.standard_level_id)
                            .unwrap_or(34)
                })
                .unwrap()
                .clone();

            let prwr = (wr_time as f64) / (last_time as f64);
            if self.set_prwr_on_times {
                time_data.set_prwr(prwr);
            }

            // Skip if whitelist / blacklist
            let player_id = time_data.get_player_id();
            match self.filters.whitelist_player_ids {
                false if self.filters.player_ids.iter().any(|x| *x == player_id) => continue,
                true if self.filters.player_ids.iter().all(|x| *x != player_id) => continue,
                _ => (),
            }

            // Set relevant values
            match &mut self.output {
                TimesetOutput::AverageFinishCharts {
                    out,
                    players_found,
                    players_found_counter,
                } => {
                    if players_found[player_id as usize] {
                        continue;
                    }

                    players_found[player_id as usize] = true;
                    *players_found_counter += 1;
                    if self.filters.whitelist_player_ids
                        && (self.filters.player_ids.len() as i32) == *players_found_counter
                    {
                        has_found_all_times = true;
                    }

                    if let Some(ref mut x) = out[player_id as usize] {
                        *x += last_rank as f64;
                    }
                }

                TimesetOutput::TotalTimeCharts {
                    out,
                    players_found,
                    players_found_counter,
                } => {
                    if players_found[player_id as usize] {
                        continue;
                    }

                    players_found[player_id as usize] = true;
                    *players_found_counter += 1;
                    if self.filters.whitelist_player_ids
                        && (self.filters.player_ids.len() as i32) == *players_found_counter
                    {
                        has_found_all_times = true;
                    }

                    if let Some(ref mut x) = out[player_id as usize] {
                        *x += last_time;
                    }
                }

                TimesetOutput::PersonalRecordWorldRecordCharts {
                    out,
                    players_found,
                    players_found_counter,
                } => {
                    if players_found[player_id as usize] {
                        continue;
                    }

                    players_found[player_id as usize] = true;
                    *players_found_counter += 1;
                    if self.filters.whitelist_player_ids
                        && (self.filters.player_ids.len() as i32) == *players_found_counter
                    {
                        has_found_all_times = true;
                    }

                    if let Some(ref mut x) = out[player_id as usize] {
                        *x += prwr;
                    }
                }

                TimesetOutput::TallyPointsCharts { out, players_found } => {
                    if players_found[player_id as usize] {
                        continue;
                    }

                    players_found[player_id as usize] = true;

                    if last_rank == 11 {
                        has_found_all_times = true;
                    }

                    if let Some(ref mut x) = out[player_id as usize] {
                        *x += 11 - (last_rank as i16);
                    }
                }

                TimesetOutput::AverageRankRatingCharts {
                    out,
                    players_found,
                    players_found_counter,
                } => {
                    if players_found[player_id as usize] {
                        continue;
                    }

                    players_found[player_id as usize] = true;
                    *players_found_counter += 1;
                    if self.filters.whitelist_player_ids
                        && (self.filters.player_ids.len() as i32) == *players_found_counter
                    {
                        has_found_all_times = true;
                    }

                    if let Some(ref mut x) = out[player_id as usize] {
                        *x += last_standard_level.value as f64;
                    }
                }

                TimesetOutput::None => (),
            }
        }

        Ok(())
    }
}
