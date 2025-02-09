mod api;
mod sql;

use actix_web::{middleware, App, HttpServer};

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

        if args.contains(&"import") {
            if args.contains(&"old") {
                sql::migrate::old::load_data(&pg_pool).await;
            }
        }
        if args.contains(&"exit") {
            std::process::exit(0);
        }
    }

    println!("Starting Backend");

    // Need this to enable the Logger middleware
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        return App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(middleware::Logger::default())
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
