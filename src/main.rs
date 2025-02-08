mod api;
mod sql;

use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = clear_terminal().status(); // silent error
    
    println!("Loading Config");
    let sql_client_config = sql::config::PostgresConfig::load_from_file();

    let args = std::env::args();

    
    
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
