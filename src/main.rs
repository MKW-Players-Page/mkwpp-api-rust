use sql::config::PostgresConfig;

mod sql;

fn main() {
    println!("Loading Config");
    let sql_client_config = PostgresConfig::load_from_file();

    println!("Starting Backend");
}
