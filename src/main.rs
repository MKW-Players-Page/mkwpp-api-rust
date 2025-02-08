mod api;
mod sql;

use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Loading Config");
    let sql_client_config = sql::config::PostgresConfig::load_from_file();

    println!("Starting Backend");

    HttpServer::new(|| App::new().wrap(middleware::Logger::default()).service(api::v1::v1()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
