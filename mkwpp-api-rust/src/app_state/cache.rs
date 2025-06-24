use std::{hash::Hash, sync::Arc};

use crate::sql::tables::{standard_levels::StandardLevels, standards::Standards};

#[derive(Default)]
pub struct Cache {
    // Non Variable Inputs
    legacy_standard_levels: Arc<[StandardLevels]>,
    standards: Arc<[Standards]>,
}

impl Cache {
    pub async fn get_legacy_standard_levels(&self) -> Arc<[StandardLevels]> {
        self.legacy_standard_levels.clone()
    }

    pub async fn get_standards(&self) -> Arc<[Standards]> {
        self.standards.clone()
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

macro_rules! update_loop_if_let_ok {
    ($table_name: ident, $var_name: ident, $executor: ident, $app_state: ident) => {
        if let Ok(v) = $table_name::load(&mut $executor, ()).await {
            let mut app_state_guard = $app_state.write().await;
            app_state_guard.cache.$var_name = v.into();
        }
    };
}

pub async fn update_loop() {
    let mut interval =
        tokio::time::interval(core::time::Duration::new(crate::ENV_VARS.cache_timeout, 0));
    loop {
        interval.tick().await;
        let app_state = super::access_app_state().await;
        let mut executor = {
            let app_state_guard = app_state.read().await;
            match app_state_guard.acquire_pg_connection().await {
                Ok(v) => v,
                Err(_) => continue,
            }
        };

        update_loop_if_let_ok!(Standards, standards, executor, app_state);
        update_loop_if_let_ok!(StandardLevels, legacy_standard_levels, executor, app_state);
    }
}
