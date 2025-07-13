mod api;
mod app_state;
mod auth;
mod custom_serde;
mod mail;
mod sql;

use std::sync::LazyLock;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};

static ENV_VARS: LazyLock<env_handler::EnvSettings> = LazyLock::new(|| {
    println!("- Loading environment variables");
    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(|v| v.as_str()).collect();
    let mut x = env_handler::EnvSettings::from_env_vars().expect("Couldn't load env vars");
    if args.contains(&"envline") {
        x.from_cli();
    }
    x
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("\x07");

    println!("| DATABASE URL: {}", ENV_VARS.database_url);
    println!("| DATABASE MAX CONNECTIONS IN POOL: {}", ENV_VARS.max_conn);
    println!(
        "| SERVER CLIENT REQUEST TIMEOUT: {}",
        ENV_VARS.client_request_timeout
    );
    println!("| SERVER CONNECTION KEEP ALIVE: {}", ENV_VARS.keep_alive);

    import_data().await;

    println!("- Starting Cache Update Loop");
    tokio::task::spawn(app_state::cache::update_loop());

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
            .wrap(middleware::Compress::default())
            .service(api::v1::v1())
    })
    .bind(("127.0.0.1", 8080))?
    .client_request_timeout(std::time::Duration::from_micros(
        ENV_VARS.client_request_timeout * 1000,
    ))
    .keep_alive(std::time::Duration::from_micros(ENV_VARS.keep_alive * 1000))
    .run()
    .await
}

#[cfg(feature = "import_data_old")]
async fn import_data() {
    let app_state = app_state::access_app_state().await;
    let app_state = app_state.write().await;

    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(|v| v.as_str()).collect();

    if args.contains(&"import") {
        println!("- Importing old data");
        sql::migrate::old::import_data(&app_state.pg_pool).await;
    }
    if args.contains(&"exit") {
        std::process::exit(0);
    }
}
#[cfg(feature = "import_data_new")]
async fn import_data() {
    let app_state = app_state::access_app_state().await;
    let app_state = app_state.write().await;

    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(|v| v.as_str()).collect();

    if args.contains(&"export") {
        println!("- Exporting data");
        sql::migrate::new::export_data(&app_state.pg_pool).await;
    }

    if args.contains(&"import") {
        println!("- Importing data");
        sql::migrate::new::import_data(&app_state.pg_pool).await;
    }

    if args.contains(&"exit") {
        std::process::exit(0);
    }
}

#[cfg(not(any(feature = "import_data_old", feature = "import_data_new")))]
async fn import_data() {
    println!("- Importing data was not compiled");
}
