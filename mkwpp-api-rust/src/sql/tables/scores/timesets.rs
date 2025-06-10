use anyhow::anyhow;
use chrono::NaiveDate;

use crate::{
    app_state::access_app_state,
    sql::tables::{Category, scores::slowest_times::SlowestTimesInputs},
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
    AFCharts {
        out: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    TotalTimeCharts {
        out: Vec<Option<i32>>,
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

        self.output = TimesetOutput::AFCharts {
            out,
            players_found: vec![false; reserve_space],
            players_found_counter: 0,
        };

        self.core_loop().await;

        match &self.output {
            TimesetOutput::AFCharts { out, .. } => {
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
            _ => Ok(vec![]),
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

    async fn core_loop(&mut self) {
        let app_state = crate::app_state::access_app_state().await;
        let mut app_state = app_state.write().unwrap();

        let slowest_times = app_state
            .get_slowest_times(SlowestTimesInputs {
                category: self.filters.category,
                max_date: self.filters.max_date,
                region_id: self.filters.region_id,
            })
            .await;
        let standard_levels = app_state.get_legacy_standard_levels().await;
        let standards = app_state.get_legacy_standards().await;

        std::mem::drop(app_state);

        self.divvie_value = match self.filters.is_lap {
            Some(_) => 32.0,
            None => 64.0,
        };

        let mut last_track = 0;
        let mut last_lap_type = false;
        let mut last_time = 0;
        let mut last_rank = 0;
        let mut last_arr_value = 0;
        let mut wr_time = 0;

        let mut has_found_all_times = false;

        for time_data in &mut self.timeset {
            // "Category" check to reset last values
            // This is true whenever you're in the first time for the (track + lap type)
            if last_track != time_data.get_track_id() || last_lap_type != time_data.get_is_lap() {
                // Fill in values for players which have not been found
                if !has_found_all_times {
                    match &mut self.output {
                        TimesetOutput::AFCharts {
                            out,
                            players_found,
                            players_found_counter: _,
                        } => {
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = out[*player_id as usize] {
                                        *x += (last_rank + 1) as f64;
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }

                // Reset values
                wr_time = time_data.get_time();
                last_track = time_data.get_track_id();
                last_lap_type = time_data.get_is_lap();
                last_rank = 0;
                last_time = 0;

                match &mut self.output {
                    TimesetOutput::AFCharts {
                        out: _,
                        players_found,
                        players_found_counter,
                    } => {
                        *players_found = vec![false; players_found.len()];
                        *players_found_counter = 0;
                    }
                    _ => (),
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

            // Skip if whitelist / blacklist
            let player_id = time_data.get_player_id();
            match self.filters.whitelist_player_ids {
                false if self.filters.player_ids.iter().any(|x| *x == player_id) => continue,
                true if self.filters.player_ids.iter().all(|x| *x != player_id) => continue,
                _ => (),
            }

            // Set relevant values
            if let TimesetOutput::AFCharts {
                out,
                players_found,
                players_found_counter,
            } = &mut self.output
            {
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
        }
    }
}
