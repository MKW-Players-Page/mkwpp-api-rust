mod api;
mod sql;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

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
    let config = sql::config::PostgresConfig::load_from_file().to_url();
    let pg_pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            println!("Couldn't load Postgres Connection Pool");
            println!("{e}");
            println!();
            println!("Exiting the process");
            std::process::exit(0);
        }
    };

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
