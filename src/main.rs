mod api;
mod sql;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

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

    println!("Loading Config");

    dotenvy::dotenv().expect("Couldn't read .env file");
    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(
            std::env::var(MAX_CONN_KEY)
                .map(|v| v.parse::<u32>().unwrap_or(MAX_CONN))
                .unwrap_or(MAX_CONN),
        )
        .connect(
            &std::env::var(DATABASE_URL_KEY).unwrap_or(sql::config::to_url(
                &std::env::var(USERNAME_KEY).unwrap_or(String::from(USERNAME)),
                &std::env::var(PASSWORD_KEY).unwrap_or(String::from(PASSWORD)),
                &std::env::var(DATABASE_NAME_KEY).unwrap_or(String::from(DATABASE_NAME)),
                &std::env::var(HOST_KEY).unwrap_or(String::from(HOST)),
                &std::env::var(PORT_KEY).unwrap_or(String::from(PORT)),
            )),
        )
        .await
        .expect("Couldn't load Postgres Connection Pool");

    {
        // These braces force args to go out of scope before the server is ran.
        // Effectively working as std::mem::drop(args);
        let args: Vec<String> = std::env::args().collect();
        let args: Vec<&str> = args.iter().map(|v| return v.as_str()).collect();

        if args.contains(&"import") && args.contains(&"old") {
            sql::migrate::old::load_data(&pg_pool).await;
        }
        if args.contains(&"exit") {
            std::process::exit(0);
        }
    }

    println!("Starting Backend");

    // Need this to enable the Logger middleware
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        return App::new()
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
