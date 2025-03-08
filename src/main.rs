mod api;
mod sql;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};

const MAX_CONN_KEY: &str = "MAX_CONN";
const MAX_CONN: u32 = 25;
const DATABASE_URL_KEY: &str = "DATABASE_URL";
const USERNAME_KEY: &str = "USERNAME";
const USERNAME: &str = "postgres";
const PASSWORD_KEY: &str = "PASSWORD";
const PASSWORD: &str = "password";
const DATABASE_NAME_KEY: &str = "DATABASE_NAME";
const DATABASE_NAME: &str = "mkwppdb";
const HOST_KEY: &str = "HOST";
const HOST: &str = "localhost";
const PORT_KEY: &str = "PORT";
const PORT: &str = "5432";
const CLIENT_REQUEST_TIMEOUT_KEY: &str = "CLIENT_REQUEST_TIMEOUT";
const CLIENT_REQUEST_TIMEOUT: u64 = 120000;
const KEEP_ALIVE_KEY: &str = "KEEP_ALIVE";
const KEEP_ALIVE: u64 = 60000;

struct AppState {
    pg_pool: sqlx::Pool<sqlx::Postgres>,
}

impl AppState {
    pub async fn acquire_pg_connection(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, HttpResponse> {
        return self.pg_pool.acquire().await.map_err(|e| {
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(api::generate_error_json_string(
                    "Couldn't get connection from data pool",
                    e.to_string().as_str(),
                ));
        });
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = clear_terminal().status(); // silent error

    println!("- Loading environment variables");
    dotenvy::dotenv().expect("Couldn't read .env file");
    let database_url = std::env::var(DATABASE_URL_KEY).unwrap_or(sql::config::to_url(
        &std::env::var(USERNAME_KEY).unwrap_or(String::from(USERNAME)),
        &std::env::var(PASSWORD_KEY).unwrap_or(String::from(PASSWORD)),
        &std::env::var(DATABASE_NAME_KEY).unwrap_or(String::from(DATABASE_NAME)),
        &std::env::var(HOST_KEY).unwrap_or(String::from(HOST)),
        &std::env::var(PORT_KEY).unwrap_or(String::from(PORT)),
    ));

    let max_conn = std::env::var(MAX_CONN_KEY)
        .map(|v| v.parse::<u32>().unwrap_or(MAX_CONN))
        .unwrap_or(MAX_CONN);

    let client_request_timeout = std::env::var(CLIENT_REQUEST_TIMEOUT_KEY)
        .map(|x| x.parse::<u64>().unwrap_or(CLIENT_REQUEST_TIMEOUT))
        .unwrap_or(CLIENT_REQUEST_TIMEOUT);

    let keep_alive = std::env::var(KEEP_ALIVE_KEY)
        .map(|x| x.parse::<u64>().unwrap_or(KEEP_ALIVE))
        .unwrap_or(KEEP_ALIVE);

    println!("| DATABASE_URL: {database_url}");
    println!("| MAX_CONN: {max_conn}");
    println!("| CLIENT_REQUEST_TIMEOUT: {client_request_timeout}");
    println!("| KEEP_ALIVE: {keep_alive}");

    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_conn)
        .connect(&database_url)
        .await
        .expect("Couldn't load Postgres Connection Pool");

    println!("- Reading CLI args");
    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(|v| return v.as_str()).collect();

    if args.contains(&"import") && args.contains(&"old") {
        sql::migrate::old::load_data(&pg_pool).await;
    }
    if args.contains(&"exit") {
        std::process::exit(0);
    }

    println!("- Dropping useless data");
    std::mem::drop(args);
    std::mem::drop(database_url);

    println!("- Enabling environment logger");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("- Starting Backend");
    HttpServer::new(move || {
        let cors = Cors::permissive();

        return App::new()
            .wrap(cors)
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(AppState {
                pg_pool: pg_pool.clone(),
            }))
            .service(api::v1::v1());
    })
    .bind(("127.0.0.1", 8080))?
    .client_request_timeout(std::time::Duration::from_micros(client_request_timeout))
    .keep_alive(std::time::Duration::from_micros(keep_alive))
    .run()
    .await
}

#[cfg(target_family = "unix")]
fn clear_terminal() -> std::process::Command {
    return std::process::Command::new("clear");
}

#[cfg(target_family = "windows")]
fn clear_terminal() -> std::process::Command {
    return std::process::Command::new("cls");
}
