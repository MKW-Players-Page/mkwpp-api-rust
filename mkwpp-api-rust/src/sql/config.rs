pub fn to_url(
    username: &str,
    password: &str,
    host: &str,
    port: &str,
    database_name: &str,
) -> String {
    return format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, host, port, database_name
    );
}
