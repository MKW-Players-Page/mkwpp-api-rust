mod api;
mod auth;
mod sql;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};

struct AppState {
    pg_pool: sqlx::Pool<sqlx::Postgres>,
}

impl AppState {
    pub async fn acquire_pg_connection(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, HttpResponse> {
        self.pg_pool.acquire().await.map_err(|e| {
            crate::api::FinalErrorResponse::new_no_fields(vec![
                String::from("Couldn't get connection from data pool"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError)
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("- Loading environment variables");
    let env_vars = env_handler::EnvSettings::from_env_vars().expect("Couldn't load env vars");

    println!("| DATABASE URL: {}", env_vars.database_url);
    println!("| DATABASE MAX CONNECTIONS IN POOL: {}", env_vars.max_conn);
    println!(
        "| SERVER CLIENT REQUEST TIMEOUT: {}",
        env_vars.client_request_timeout
    );
    println!("| SERVER CONNECTION KEEP ALIVE: {}", env_vars.keep_alive);

    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(env_vars.max_conn)
        .connect(&env_vars.database_url)
        .await
        .expect("Couldn't load Postgres Connection Pool");

    println!("- Reading CLI args");
    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(|v| v.as_str()).collect();

    if args.contains(&"import") && args.contains(&"old") {
        sql::migrate::old::load_data(&pg_pool).await;
    }
    if args.contains(&"exit") {
        std::process::exit(0);
    }

    println!("- Dropping useless data");
    std::mem::drop(args);

    println!("- Enabling environment logger");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("- Starting Backend");
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(AppState {
                pg_pool: pg_pool.clone(),
            }))
            .service(api::v1::v1())
    })
    .bind(("127.0.0.1", 8080))?
    .client_request_timeout(std::time::Duration::from_micros(
        env_vars.client_request_timeout * 1000,
    ))
    .keep_alive(std::time::Duration::from_micros(env_vars.keep_alive * 1000))
    .run()
    .await
}
