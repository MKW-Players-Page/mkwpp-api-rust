use std::sync::{Arc, RwLock};

use actix_web::HttpResponse;
use anyhow::anyhow;

use crate::sql::tables::{
    scores::{SlowestTimes, slowest_times::SlowestTimesInputs},
    standard_levels::StandardLevels,
    standards::Standards,
};

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

    pub async fn get_slowest_times(
        &mut self,
        input: SlowestTimesInputs,
    ) -> Result<Arc<[SlowestTimes]>, HttpResponse> {
        let mut executor = self
            .acquire_pg_connection()
            .await
            .map_err(Self::pg_conn_http_error)?;
        self.cache
            .get_slowest_times(&mut executor, input)
            .await
            .map_err(|e| {
                crate::api::FinalErrorResponse::new_no_fields(vec![
                    String::from("Couldn't get slowest times"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError)
            })
    }

    pub async fn get_legacy_standard_levels(
        &mut self,
    ) -> Result<Arc<[StandardLevels]>, HttpResponse> {
        let mut executor = self
            .acquire_pg_connection()
            .await
            .map_err(Self::pg_conn_http_error)?;
        self.cache
            .get_legacy_standard_levels(&mut executor)
            .await
            .map_err(|e| {
                crate::api::FinalErrorResponse::new_no_fields(vec![
                    String::from("Couldn't get standard levels"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError)
            })
    }

    pub async fn get_legacy_standards(&mut self) -> Result<Arc<[Standards]>, HttpResponse> {
        let mut executor = self
            .acquire_pg_connection()
            .await
            .map_err(Self::pg_conn_http_error)?;
        self.cache
            .get_legacy_standards(&mut executor)
            .await
            .map_err(|e| {
                crate::api::FinalErrorResponse::new_no_fields(vec![
                    String::from("Couldn't get standards"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError)
            })
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
