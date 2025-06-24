use std::sync::Arc;
use tokio::sync::RwLock;

use actix_web::HttpResponse;
use anyhow::anyhow;

use crate::sql::tables::{standard_levels::StandardLevels, standards::Standards};

pub mod cache;

pub struct AppState {
    pub pg_pool: sqlx::Pool<sqlx::Postgres>,
    pub cache: cache::Cache,
}

impl AppState {
    pub async fn acquire_pg_connection(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, anyhow::Error> {
        self.pg_pool.acquire().await.map_err(|e| anyhow!("{e}"))
    }

    pub fn pg_conn_http_error(error: anyhow::Error) -> HttpResponse {
        crate::api::FinalErrorResponse::new_no_fields(vec![
            String::from("Couldn't get connection from data pool"),
            error.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError)
    }

    pub async fn get_legacy_standard_levels(&self) -> Arc<[StandardLevels]> {
        self.cache.get_legacy_standard_levels().await
    }

    pub async fn get_standards(&self) -> Arc<[Standards]> {
        self.cache.get_standards().await
    }
}

pub async fn access_app_state() -> &'static RwLock<AppState> {
    static APP_STATE: tokio::sync::OnceCell<RwLock<AppState>> = tokio::sync::OnceCell::const_new();
    APP_STATE
        .get_or_init(async || {
            let pg_pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(crate::ENV_VARS.max_conn)
                .connect(&crate::ENV_VARS.database_url)
                .await
                .expect("Couldn't load Postgres Connection Pool");

            let cache = cache::Cache::default();

            let app_state = AppState { pg_pool, cache };

            RwLock::new(app_state)
        })
        .await
}
