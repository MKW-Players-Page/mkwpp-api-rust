use std::{collections::hash_map::HashMap, hash::Hash, sync::Arc};

use crate::sql::tables::{
    scores::slowest_times::{SlowestTimes, SlowestTimesInputs},
    standard_levels::StandardLevels,
    standards::Standards,
};

const SLOWEST_TIMES_REFRESH: i64 = 1200;
const STANDARDS_REFRESH: i64 = 1200;

#[derive(Default)]
pub struct Cache {
    slowest_times: HashMap<SlowestTimesInputs, (i64, Arc<[SlowestTimes]>)>,
    legacy_standard_levels: (i64, Arc<[StandardLevels]>),
    legacy_standards: (i64, Arc<[Standards]>),
}

impl Cache {
    fn flush(&mut self) {
        self.slowest_times
            .retain(|_, (date, _)| *date > chrono::Utc::now().timestamp());

        if self.legacy_standard_levels.0 > chrono::Utc::now().timestamp() {
            self.legacy_standard_levels.1 = Arc::new([]);
        }

        if self.legacy_standards.0 > chrono::Utc::now().timestamp() {
            self.legacy_standards.1 = Arc::new([]);
        }
    }

    pub async fn get_slowest_times(
        &mut self,
        executor: &mut sqlx::PgConnection,
        input: SlowestTimesInputs,
    ) -> Result<Arc<[SlowestTimes]>, anyhow::Error> {
        let out = match self.slowest_times.get(&input) {
            Some((date, dataset)) if *date > chrono::Utc::now().timestamp() => Ok(dataset.clone()),
            _ => self.insert_slowest_times(executor, input).await,
        };

        self.flush();

        out
    }

    async fn insert_slowest_times(
        &mut self,
        executor: &mut sqlx::PgConnection,
        input: SlowestTimesInputs,
    ) -> Result<Arc<[SlowestTimes]>, anyhow::Error> {
        SlowestTimes::load(executor, input.clone()).await.map(|x| {
            let x: Arc<[SlowestTimes]> = x.into();
            self.slowest_times.insert(
                input,
                (
                    chrono::Utc::now().timestamp() + SLOWEST_TIMES_REFRESH,
                    x.clone(),
                ),
            );
            x
        })
    }

    pub async fn get_legacy_standard_levels(
        &mut self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Arc<[StandardLevels]>, anyhow::Error> {
        if self.legacy_standard_levels.0 > chrono::Utc::now().timestamp() {
            return Ok(self.legacy_standard_levels.1.clone());
        }

        let out = self.insert_legacy_standard_levels(executor).await;

        self.flush();

        out
    }

    async fn insert_legacy_standard_levels(
        &mut self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Arc<[StandardLevels]>, anyhow::Error> {
        StandardLevels::load(executor, ()).await.map(|x| {
            let x: Arc<[StandardLevels]> = x.into();
            self.legacy_standard_levels = (
                chrono::Utc::now().timestamp() + STANDARDS_REFRESH,
                x.clone(),
            );
            x
        })
    }

    pub async fn get_legacy_standards(
        &mut self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Arc<[Standards]>, anyhow::Error> {
        if self.legacy_standards.0 > chrono::Utc::now().timestamp() {
            return Ok(self.legacy_standards.1.clone());
        }

        let out = self.insert_legacy_standards(executor).await;

        self.flush();

        out
    }

    async fn insert_legacy_standards(
        &mut self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Arc<[Standards]>, anyhow::Error> {
        Standards::load(executor, ()).await.map(|x| {
            let x: Arc<[Standards]> = x.into();
            self.legacy_standards = (
                chrono::Utc::now().timestamp() + STANDARDS_REFRESH,
                x.clone(),
            );
            x
        })
    }
}

pub trait CacheItem {
    type Input: Hash;
    async fn load(
        executor: &mut sqlx::PgConnection,
        input: Self::Input,
    ) -> Result<Vec<Self>, anyhow::Error>
    where
        Self: Sized;
}
