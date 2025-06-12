use std::collections::HashMap;

use anyhow::anyhow;
use chrono::NaiveDate;

use crate::{
    app_state::access_app_state,
    sql::tables::{
        Category,
        scores::{Times, matchup::MatchupData, rankings::RankingType, timesheet::Timesheet},
        standard_levels::StandardLevels,
    },
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
        rank_sums: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    TotalTimeCharts {
        total_times: Vec<Option<i32>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    PersonalRecordWorldRecordCharts {
        prwr_sums: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    TallyPointsCharts {
        tally_points: Vec<Option<i16>>,
        players_found: Vec<bool>,
    },
    AverageRankRatingCharts {
        arr_value_sums: Vec<Option<f64>>,
        players_found: Vec<bool>,
        players_found_counter: i32,
    },
    PlayerTimesheet {
        times: Vec<Option<Times>>,
        rank_sum: f64,
        total_time: i32,
        prwr_sum: f64,
        tally_points: i16,
        arr_value_sum: f64,
    },
    PlayerMatchup {
        times: Vec<Vec<Option<Times>>>,
        difference_to_first_times: Vec<Vec<i32>>,
        difference_to_next_times: Vec<Vec<i32>>,
        rank_sums: Vec<f64>,
        total_times: Vec<i32>,
        prwr_sums: Vec<f64>,
        tally_points: Vec<i16>,
        arr_value_sums: Vec<f64>,
        wins: Vec<i8>,
        first_time: i32,
        last_time: i32,
        players_found: Vec<bool>,
        players_found_counter: i32,
        player_ids_to_index: HashMap<i32, usize>,
    },
}

pub trait ValidTimesetItem {
    fn get_time(&self) -> i32;
    fn get_time_id(&self) -> i32;
    fn get_track_id(&self) -> i32;
    fn get_is_lap(&self) -> bool;
    fn get_player_id(&self) -> i32;
    fn get_date(&self) -> Option<chrono::NaiveDate>;
    fn get_initial_rank(&self) -> Option<i32>;
    fn get_category(&self) -> Category;
    fn get_video_link(&self) -> Option<String>;
    fn get_ghost_link(&self) -> Option<String>;
    fn get_comment(&self) -> Option<String>;

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
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        self.internal_rankings_charts(TimesetOutput::AverageFinishCharts {
            rank_sums: vec![None; 0],
            players_found: vec![false; 0],
            players_found_counter: 0,
        })
        .await
    }

    pub async fn calculate_total_time_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        self.internal_rankings_charts(TimesetOutput::TotalTimeCharts {
            total_times: vec![None; 0],
            players_found: vec![false; 0],
            players_found_counter: 0,
        })
        .await
    }

    pub async fn calculate_tally_points_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        self.internal_rankings_charts(TimesetOutput::TallyPointsCharts {
            tally_points: vec![None; 0],
            players_found: vec![false; 0],
        })
        .await
    }

    pub async fn calculate_personal_record_world_record_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        self.internal_rankings_charts(TimesetOutput::PersonalRecordWorldRecordCharts {
            prwr_sums: vec![None; 0],
            players_found: vec![false; 0],
            players_found_counter: 0,
        })
        .await
    }

    pub async fn calculate_average_rank_rating_charts(
        &mut self,
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        self.internal_rankings_charts(TimesetOutput::AverageRankRatingCharts {
            arr_value_sums: vec![None; 0],
            players_found: vec![false; 0],
            players_found_counter: 0,
        })
        .await
    }

    async fn internal_rankings_charts(
        &mut self,
        mut output_type: TimesetOutput,
    ) -> Result<Vec<(i32, i32, RankingType)>, anyhow::Error> {
        if self.filters.player_ids.len() == 0 && self.filters.whitelist_player_ids {
            return Ok(vec![]);
        }

        self.invert_blacklist().await?;
        self.filters.player_ids.sort_unstable();

        let reserve_space = *self.filters.player_ids.iter().last().unwrap() as usize + 1;

        match &mut output_type {
            TimesetOutput::AverageFinishCharts {
                rank_sums: out,
                players_found,
                players_found_counter,
            }
            | TimesetOutput::PersonalRecordWorldRecordCharts {
                prwr_sums: out,
                players_found,
                players_found_counter,
            }
            | TimesetOutput::AverageRankRatingCharts {
                arr_value_sums: out,
                players_found,
                players_found_counter,
            } => {
                *out = vec![None; reserve_space];
                for player_id in &self.filters.player_ids {
                    out[*player_id as usize] = Some(0.0)
                }
                *players_found_counter = 0;
                *players_found = vec![false; reserve_space];
            }
            TimesetOutput::TotalTimeCharts {
                total_times,
                players_found,
                players_found_counter,
            } => {
                *total_times = vec![None; reserve_space];
                for player_id in &self.filters.player_ids {
                    total_times[*player_id as usize] = Some(0)
                }
                *players_found_counter = 0;
                *players_found = vec![false; reserve_space];
            }
            TimesetOutput::TallyPointsCharts {
                tally_points,
                players_found,
            } => {
                *tally_points = vec![None; reserve_space];
                for player_id in &self.filters.player_ids {
                    tally_points[*player_id as usize] = Some(0)
                }
                *players_found = vec![false; reserve_space];
            }
            TimesetOutput::None
            | TimesetOutput::PlayerTimesheet { .. }
            | TimesetOutput::PlayerMatchup { .. } => {
                panic!("This code should never be encountered")
            }
        };
        self.output = output_type;

        self.core_loop().await?;

        match &self.output {
            TimesetOutput::AverageFinishCharts { rank_sums, .. } => {
                let mut af_and_ids = rank_sums
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
                    .map(|(ranking, (id, af))| {
                        ((ranking as i32) + 1, id, RankingType::AverageFinish(af))
                    })
                    .collect())
            }
            TimesetOutput::TotalTimeCharts { total_times, .. } => {
                let mut tt_and_ids = total_times
                    .iter()
                    .enumerate()
                    .filter_map(|(id, time_sum)| time_sum.map(|time_sum| ((id as i32), time_sum)))
                    .collect::<Vec<(i32, i32)>>();
                tt_and_ids.sort_by(|(_id1, tt1), (_id2, tt2)| tt1.cmp(&tt2));
                Ok(tt_and_ids
                    .into_iter()
                    .enumerate()
                    .map(|(ranking, (id, tt))| {
                        ((ranking as i32) + 1, id, RankingType::TotalTime(tt))
                    })
                    .collect())
            }
            TimesetOutput::TallyPointsCharts { tally_points, .. } => {
                let mut tp_and_ids = tally_points
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
                    .map(|(ranking, (id, tp))| {
                        ((ranking as i32) + 1, id, RankingType::TallyPoints(tp))
                    })
                    .collect())
            }
            TimesetOutput::PersonalRecordWorldRecordCharts { prwr_sums, .. } => {
                let mut prwr_and_ids = prwr_sums
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
                    .map(|(ranking, (id, prwr))| {
                        (
                            (ranking as i32) + 1,
                            id,
                            RankingType::PersonalRecordWorldRecord(prwr),
                        )
                    })
                    .collect())
            }
            TimesetOutput::AverageRankRatingCharts { arr_value_sums, .. } => {
                let mut arr_and_ids = arr_value_sums
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
                    .map(|(ranking, (id, arr))| {
                        (
                            (ranking as i32) + 1,
                            id,
                            RankingType::AverageRankRating(arr),
                        )
                    })
                    .collect())
            }
            TimesetOutput::None
            | TimesetOutput::PlayerTimesheet { .. }
            | TimesetOutput::PlayerMatchup { .. } => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn timesheet(&mut self, player_id: i32) -> Result<Timesheet, anyhow::Error> {
        self.calculate_divvie_value();
        self.output = TimesetOutput::PlayerTimesheet {
            times: vec![None; self.divvie_value as usize],
            rank_sum: 0.0,
            total_time: 0,
            prwr_sum: 0.0,
            tally_points: 0,
            arr_value_sum: 0.0,
        };

        self.filters.whitelist_player_ids = true;
        self.filters.player_ids = vec![player_id; 1];

        self.core_loop().await?;

        match &mut self.output {
            TimesetOutput::PlayerTimesheet {
                times,
                rank_sum,
                total_time,
                prwr_sum,
                tally_points,
                arr_value_sum,
                ..
            } => Ok(Timesheet {
                times: times.iter_mut().filter_map(|x| std::mem::take(x)).collect(),
                af: *rank_sum / self.divvie_value,
                total_time: *total_time,
                tally: *tally_points,
                arr: *arr_value_sum / self.divvie_value,
                prwr: *prwr_sum / self.divvie_value,
            }),
            TimesetOutput::None
            | TimesetOutput::AverageFinishCharts { .. }
            | TimesetOutput::TotalTimeCharts { .. }
            | TimesetOutput::PersonalRecordWorldRecordCharts { .. }
            | TimesetOutput::TallyPointsCharts { .. }
            | TimesetOutput::AverageRankRatingCharts { .. }
            | TimesetOutput::PlayerMatchup { .. } => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    pub async fn matchup(&mut self, player_ids: Vec<i32>) -> Result<MatchupData, anyhow::Error> {
        self.calculate_divvie_value();
        let player_numbers = player_ids.len();

        self.output = TimesetOutput::PlayerMatchup {
            times: vec![vec![None; self.divvie_value as usize]; player_numbers],
            difference_to_first_times: vec![vec![0; self.divvie_value as usize]; player_numbers],
            difference_to_next_times: vec![vec![0; self.divvie_value as usize]; player_numbers],
            rank_sums: vec![0.0; player_numbers],
            total_times: vec![0; player_numbers],
            prwr_sums: vec![0.0; player_numbers],
            tally_points: vec![0; player_numbers],
            arr_value_sums: vec![0.0; player_numbers],
            wins: vec![0; player_numbers],
            first_time: 0,
            last_time: 0,
            players_found: vec![false; player_numbers],
            players_found_counter: 0,
            player_ids_to_index: player_ids
                .iter()
                .enumerate()
                .map(|(index, player_id)| (*player_id, index))
                .collect::<HashMap<i32, usize>>(),
        };
        self.filters.player_ids = player_ids;
        self.filters.whitelist_player_ids = true;

        self.core_loop().await?;

        match &mut self.output {
            TimesetOutput::PlayerMatchup {
                times,
                difference_to_first_times,
                difference_to_next_times,
                rank_sums,
                total_times,
                prwr_sums,
                tally_points,
                arr_value_sums,
                wins,
                first_time: _,
                last_time: _,
                players_found: _,
                players_found_counter: _,
                player_ids_to_index: _,
            } => {
                let afs = rank_sums
                    .into_iter()
                    .map(|x| *x / self.divvie_value)
                    .collect::<Vec<f64>>();
                let prwrs = prwr_sums
                    .into_iter()
                    .map(|x| *x / self.divvie_value)
                    .collect::<Vec<f64>>();
                let arr_values = arr_value_sums
                    .into_iter()
                    .map(|x| *x / self.divvie_value)
                    .collect::<Vec<f64>>();

                let first_af = *afs.iter().min_by(|x, y| x.total_cmp(y)).unwrap();
                let first_total_time = *total_times.iter().min().unwrap();
                let first_prwr = *prwrs.iter().max_by(|x, y| x.total_cmp(y)).unwrap();
                let first_tally_points = *tally_points.iter().max().unwrap();
                let first_arr = *arr_values.iter().min_by(|x, y| x.total_cmp(y)).unwrap();
                let first_wins = *wins.iter().max().unwrap();

                let delta_af = *afs.iter().max_by(|x, y| x.total_cmp(y)).unwrap() - first_af;
                let delta_total_time = *total_times.iter().max().unwrap() - first_total_time;
                let delta_prwr = first_prwr - *prwrs.iter().min_by(|x, y| x.total_cmp(y)).unwrap();
                let delta_tally_points = first_tally_points - *tally_points.iter().min().unwrap();
                let delta_arr =
                    *arr_values.iter().max_by(|x, y| x.total_cmp(y)).unwrap() - first_arr;
                let delta_wins = first_wins - *wins.iter().min().unwrap();

                let diff_af_first: Vec<_> = afs.iter().map(|x| x - first_af).collect();
                let diff_total_time_first: Vec<_> =
                    total_times.iter().map(|x| x - first_total_time).collect();
                let diff_tally_first: Vec<_> = tally_points
                    .iter()
                    .map(|x| first_tally_points - x)
                    .collect();
                let diff_arr_first: Vec<_> = arr_values.iter().map(|x| x - first_arr).collect();
                let diff_prwr_first: Vec<_> = prwrs.iter().map(|x| first_prwr - x).collect();
                let diff_wins_first: Vec<_> = wins.iter().map(|x| first_wins - x).collect();

                let mut rgb_diff = vec![vec![0; self.divvie_value as usize]; player_numbers];
                for track_index in 0..(self.divvie_value as usize) {
                    let mut delta = i32::MIN;
                    for player_index in 0..player_numbers {
                        delta = std::cmp::max(
                            difference_to_first_times[player_index][track_index],
                            delta,
                        );
                    }

                    for player_index in 0..player_numbers {
                        rgb_diff[player_index][track_index] = (255.0
                            - ((difference_to_first_times[player_index][track_index] as f64
                                / (delta as f64))
                                * 155.0))
                            as u8;
                    }
                }

                let mut timesheet_vec = Vec::with_capacity(player_numbers);
                for player_index in 0..player_numbers {
                    timesheet_vec.push(Timesheet {
                        times: std::mem::take(&mut times[player_index])
                            .into_iter()
                            .filter_map(|x| x)
                            .collect(),
                        af: afs[player_index],
                        arr: arr_values[player_index],
                        prwr: prwrs[player_index],
                        tally: tally_points[player_index],
                        total_time: total_times[player_index],
                    });
                }

                // TODO: Whatever the fuck this is, please rewrite
                Ok(MatchupData {
                    player_data: timesheet_vec,
                    diff_first: std::mem::take(difference_to_first_times),
                    diff_next: std::mem::take(difference_to_next_times),
                    diff_af_next: {
                        let mut tmp = afs
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, f64)>>();
                        tmp.sort_by(|(_, x), (_, y)| y.total_cmp(x));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => x - y,
                                    None => 0.0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    diff_total_time_next: {
                        let mut tmp = total_times
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, i32)>>();
                        tmp.sort_by(|(_, x), (_, y)| y.cmp(x));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => x - y,
                                    None => 0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    diff_tally_next: {
                        let mut tmp = tally_points
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, i16)>>();
                        tmp.sort_by(|(_, x), (_, y)| x.cmp(y));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => y - x,
                                    None => 0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    diff_arr_next: {
                        let mut tmp = arr_values
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, f64)>>();
                        tmp.sort_by(|(_, x), (_, y)| y.total_cmp(x));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => x - y,
                                    None => 0.0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    diff_prwr_next: {
                        let mut tmp = prwrs
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, f64)>>();
                        tmp.sort_by(|(_, x), (_, y)| x.total_cmp(y));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => y - x,
                                    None => 0.0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    diff_wins_next: {
                        let mut tmp = wins
                            .iter()
                            .map(|x| *x)
                            .enumerate()
                            .collect::<Vec<(usize, i8)>>();
                        tmp.sort_by(|(_, x), (_, y)| x.cmp(y));
                        let mut tmp = tmp.into_iter().peekable();
                        let mut out = vec![];
                        while let Some((z, x)) = tmp.next() {
                            out.push((
                                z,
                                match tmp.peek() {
                                    Some((_, y)) => y - x,
                                    None => 0,
                                },
                            ));
                        }
                        out.sort_by(|(x, _), (y, _)| x.cmp(y));
                        out.into_iter().map(|(_, x)| x).collect()
                    },
                    rgb_diff,
                    rgb_diff_af: if delta_af == 0.0 {
                        vec![0; player_numbers]
                    } else {
                        diff_af_first
                            .iter()
                            .map(|x| (255.0 - (x / delta_af * 155.0)) as u8)
                            .collect()
                    },
                    rgb_diff_total_time: if delta_total_time == 0 {
                        vec![0; player_numbers]
                    } else {
                        diff_total_time_first
                            .iter()
                            .map(|x| {
                                (255.0 - ((*x as f64) / (delta_total_time as f64) * 155.0)) as u8
                            })
                            .collect()
                    },
                    rgb_diff_tally: if delta_tally_points == 0 {
                        vec![0; player_numbers]
                    } else {
                        diff_tally_first
                            .iter()
                            .map(|x| {
                                (255.0 - ((*x as f64) / (delta_tally_points as f64) * 155.0)) as u8
                            })
                            .collect()
                    },
                    rgb_diff_arr: if delta_arr == 0.0 {
                        vec![0; player_numbers]
                    } else {
                        diff_arr_first
                            .iter()
                            .map(|x| (255.0 - (x / delta_arr * 155.0)) as u8)
                            .collect()
                    },
                    rgb_diff_prwr: if delta_prwr == 0.0 {
                        vec![0; player_numbers]
                    } else {
                        diff_prwr_first
                            .iter()
                            .map(|x| (255.0 - (x / delta_prwr * 155.0)) as u8)
                            .collect()
                    },
                    rgb_diff_wins: if delta_wins == 0 {
                        vec![0; player_numbers]
                    } else {
                        diff_wins_first
                            .iter()
                            .map(|x| (255.0 - ((*x as f64) / (delta_wins as f64) * 155.0)) as u8)
                            .collect()
                    },
                    diff_af_first,
                    diff_prwr_first,
                    diff_total_time_first,
                    diff_tally_first,
                    diff_arr_first,
                    diff_wins_first,
                    wins: std::mem::take(wins),
                })
            }
            TimesetOutput::None
            | TimesetOutput::PlayerTimesheet { .. }
            | TimesetOutput::AverageFinishCharts { .. }
            | TimesetOutput::TotalTimeCharts { .. }
            | TimesetOutput::PersonalRecordWorldRecordCharts { .. }
            | TimesetOutput::TallyPointsCharts { .. }
            | TimesetOutput::AverageRankRatingCharts { .. } => Err(anyhow!(
                "Something went very wrong, the output type changed unexpectedly"
            )),
        }
    }

    async fn invert_blacklist(&mut self) -> Result<(), anyhow::Error> {
        if self.filters.whitelist_player_ids {
            return Ok(());
        }

        let app_state = access_app_state().await.read().await;
        let mut executor = app_state.acquire_pg_connection().await?;
        std::mem::drop(app_state);

        self.filters.player_ids = crate::sql::tables::players::Players::get_ids_but_list(
            &mut executor,
            &self.filters.player_ids,
        )
        .await
        .map_err(|e| anyhow!("Couldn't get the player ids from the list. {e}"))?;
        self.filters.whitelist_player_ids = true;

        Ok(())
    }

    fn calculate_divvie_value(&mut self) {
        self.divvie_value = match self.filters.is_lap {
            Some(_) => 32.0,
            None => 64.0,
        };
    }

    async fn core_loop(&mut self) -> Result<(), anyhow::Error> {
        let app_state = crate::app_state::access_app_state().await;
        let app_state = app_state.read().await;
        let standard_levels = app_state.get_legacy_standard_levels().await;
        let standards = app_state.get_standards().await;
        std::mem::drop(app_state);

        self.calculate_divvie_value();

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
                            rank_sums,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let rank = (last_rank + 1) as f64;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = rank_sums[*player_id as usize] {
                                        *x += rank;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TotalTimeCharts {
                            total_times,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let time = last_time + 1;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = total_times[*player_id as usize] {
                                        *x += time;
                                    }
                                }
                            }
                        }
                        TimesetOutput::PersonalRecordWorldRecordCharts {
                            prwr_sums,
                            players_found,
                            players_found_counter: _,
                        } => {
                            let prwr = (wr_time as f64) / ((last_time + 1) as f64);
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = prwr_sums[*player_id as usize] {
                                        *x += prwr;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TallyPointsCharts {
                            tally_points,
                            players_found,
                        } if last_rank < 11 => {
                            let pts = 11 - (last_rank as i16);
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = tally_points[*player_id as usize] {
                                        *x += pts;
                                    }
                                }
                            }
                        }
                        TimesetOutput::TallyPointsCharts { .. } => (),

                        TimesetOutput::AverageRankRatingCharts {
                            arr_value_sums,
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
                                    .expect("It should always find a standard level")
                                    .value
                            } as f64;
                            for player_id in &self.filters.player_ids {
                                if !players_found[*player_id as usize] {
                                    if let Some(ref mut x) = arr_value_sums[*player_id as usize] {
                                        *x += arr_value;
                                    }
                                }
                            }
                        }
                        TimesetOutput::PlayerTimesheet {
                            times: _,
                            rank_sum,
                            total_time,
                            prwr_sum,
                            tally_points,
                            arr_value_sum,
                        } => {
                            let time = last_time + 1;
                            let rank = last_rank + 1;
                            *rank_sum += rank as f64;
                            *total_time += time;
                            *prwr_sum += (wr_time as f64) / (time as f64);
                            *tally_points += std::cmp::max(11 - (rank as i16), 0);
                            // TODO: Hardcoded Newbie Value
                            *arr_value_sum += if last_standard_level.id == 34 {
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
                                                            && value >= time
                                                    }
                                                })
                                                .map(|standard| standard.standard_level_id)
                                                .unwrap_or(34)
                                    })
                                    .expect("It should always find a standard level")
                                    .value
                            } as f64
                        }

                        TimesetOutput::PlayerMatchup {
                            times: _,
                            difference_to_first_times,
                            difference_to_next_times,
                            rank_sums,
                            total_times,
                            prwr_sums,
                            tally_points,
                            arr_value_sums,
                            wins: _,
                            first_time,
                            last_time: last_time_selected,
                            players_found,
                            players_found_counter: _,
                            player_ids_to_index,
                        } => {
                            let time = last_time + 1;
                            let rank = last_rank + 1;
                            let prwr = (wr_time as f64) / (time as f64);
                            let tally_points_default = std::cmp::max(11 - (rank as i16), 0);
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
                                                            && value >= time
                                                    }
                                                })
                                                .map(|standard| standard.standard_level_id)
                                                .unwrap_or(34)
                                    })
                                    .expect("It should always find a standard level")
                                    .value
                            } as f64;
                            for player_id in &self.filters.player_ids {
                                let player_index = *player_ids_to_index.get(&player_id).expect(
                                    "Somehow there is no player id in relevant player_ids_to_index hashmap",
                                );
                                if !players_found[player_index] {
                                    let track_index = match self.filters.is_lap {
                                        Some(_) => (last_track as usize) - 1,
                                        None => {
                                            (((last_track as usize) - 1) * 2)
                                                + (last_lap_type as usize)
                                        }
                                    };

                                    difference_to_first_times[player_index][track_index] =
                                        time - *first_time;
                                    difference_to_next_times[player_index][track_index] =
                                        time - *last_time_selected;

                                    rank_sums[player_index] += rank as f64;
                                    total_times[player_index] += time;
                                    prwr_sums[player_index] += prwr;
                                    tally_points[player_index] += tally_points_default;
                                    arr_value_sums[player_index] += arr_value;
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
                        rank_sums: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::TotalTimeCharts {
                        total_times: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::PersonalRecordWorldRecordCharts {
                        prwr_sums: _,
                        players_found,
                        players_found_counter,
                    }
                    | TimesetOutput::AverageRankRatingCharts {
                        arr_value_sums: _,
                        players_found,
                        players_found_counter,
                    } => {
                        *players_found = vec![false; players_found.len()];
                        *players_found_counter = 0;
                    }
                    TimesetOutput::TallyPointsCharts {
                        tally_points: _,
                        players_found,
                    } => {
                        *players_found = vec![false; players_found.len()];
                    }
                    TimesetOutput::PlayerMatchup {
                        times: _,
                        difference_to_first_times: _,
                        difference_to_next_times: _,
                        rank_sums: _,
                        total_times: _,
                        prwr_sums: _,
                        tally_points: _,
                        arr_value_sums: _,
                        wins: _,
                        first_time,
                        last_time,
                        players_found,
                        players_found_counter,
                        player_ids_to_index: _,
                    } => {
                        *players_found_counter = 0;
                        *players_found = vec![false; self.filters.player_ids.len()];
                        *first_time = 0;
                        *last_time = 0;
                    }
                    TimesetOutput::PlayerTimesheet { .. } | TimesetOutput::None => (),
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

            // TODO: Hardcoded Newbie Value
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
                .expect("It should always find a standard level")
                .clone();

            let prwr = (wr_time as f64) / (last_time as f64);
            if self.set_prwr_on_times {
                time_data.set_prwr(prwr);
            }

            // Skip if whitelist / blacklist
            let player_id = time_data.get_player_id();
            match self.filters.whitelist_player_ids {
                false if self.filters.player_ids.iter().any(|x| *x == player_id) => continue,
                true if !self.filters.player_ids.iter().any(|x| *x == player_id) => continue,
                _ => (),
            }

            // Set relevant values
            match &mut self.output {
                TimesetOutput::AverageFinishCharts {
                    rank_sums,
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

                    if let Some(ref mut x) = rank_sums[player_id as usize] {
                        *x += last_rank as f64;
                    }
                }

                TimesetOutput::TotalTimeCharts {
                    total_times,
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

                    if let Some(ref mut x) = total_times[player_id as usize] {
                        *x += last_time;
                    }
                }

                TimesetOutput::PersonalRecordWorldRecordCharts {
                    prwr_sums,
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

                    if let Some(ref mut x) = prwr_sums[player_id as usize] {
                        *x += prwr;
                    }
                }

                TimesetOutput::TallyPointsCharts {
                    tally_points: out,
                    players_found,
                } => {
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
                    arr_value_sums,
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

                    if let Some(ref mut x) = arr_value_sums[player_id as usize] {
                        *x += last_standard_level.value as f64;
                    }
                }
                TimesetOutput::PlayerTimesheet {
                    times,
                    rank_sum,
                    total_time,
                    prwr_sum,
                    tally_points,
                    arr_value_sum,
                } => {
                    has_found_all_times = true;

                    let index = match self.filters.is_lap {
                        Some(_) => (last_track as usize) - 1,
                        None => (((last_track as usize) - 1) * 2) + (last_lap_type as usize),
                    };

                    times[index] = Some(Times {
                        value: last_time,
                        rank: last_rank,
                        id: time_data.get_time_id(),
                        prwr: prwr,
                        std_lvl_code: last_standard_level.code,
                        category: time_data.get_category(),
                        is_lap: last_lap_type,
                        track_id: last_track,
                        date: time_data.get_date(),
                        video_link: time_data.get_video_link(),
                        ghost_link: time_data.get_ghost_link(),
                        comment: time_data.get_comment(),
                        initial_rank: time_data.get_initial_rank(),
                    });

                    *rank_sum += last_rank as f64;
                    *total_time += last_time;
                    *prwr_sum += prwr;
                    *tally_points += std::cmp::max(11 - (last_rank as i16), 0);
                    *arr_value_sum += last_standard_level.value as f64;
                }

                TimesetOutput::PlayerMatchup {
                    times,
                    difference_to_first_times,
                    difference_to_next_times,
                    rank_sums,
                    total_times,
                    prwr_sums,
                    tally_points,
                    arr_value_sums,
                    wins,
                    players_found,
                    players_found_counter,
                    player_ids_to_index,
                    first_time,
                    last_time: last_time_selected,
                } => {
                    let player_index = *player_ids_to_index.get(&player_id).expect(
                        "Somehow there is no player id in relevant player_ids_to_index hashmap",
                    );
                    if players_found[player_index] {
                        continue;
                    }

                    if *players_found_counter == 0 {
                        *first_time = last_time;
                        *last_time_selected = last_time;
                        wins[player_index] += 1;
                    }

                    players_found[player_index] = true;
                    *players_found_counter += 1;
                    if self.filters.whitelist_player_ids
                        && (self.filters.player_ids.len() as i32) == *players_found_counter
                    {
                        has_found_all_times = true;
                    }

                    let track_index = match self.filters.is_lap {
                        Some(_) => (last_track as usize) - 1,
                        None => (((last_track as usize) - 1) * 2) + (last_lap_type as usize),
                    };

                    difference_to_first_times[player_index][track_index] = last_time - *first_time;
                    difference_to_next_times[player_index][track_index] =
                        last_time - *last_time_selected;

                    *last_time_selected = last_time;

                    times[player_index][track_index] = Some(Times {
                        value: last_time,
                        rank: last_rank,
                        id: time_data.get_time_id(),
                        prwr: prwr,
                        std_lvl_code: last_standard_level.code,
                        category: time_data.get_category(),
                        is_lap: last_lap_type,
                        track_id: last_track,
                        date: time_data.get_date(),
                        video_link: time_data.get_video_link(),
                        ghost_link: time_data.get_ghost_link(),
                        comment: time_data.get_comment(),
                        initial_rank: time_data.get_initial_rank(),
                    });

                    rank_sums[player_index] += last_rank as f64;
                    total_times[player_index] += last_time;
                    prwr_sums[player_index] += prwr;
                    tally_points[player_index] += std::cmp::max(11 - (last_rank as i16), 0);
                    arr_value_sums[player_index] += last_standard_level.value as f64;
                }

                TimesetOutput::None => (),
            }
        }

        Ok(())
    }
}
