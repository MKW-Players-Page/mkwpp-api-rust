#[env_handler_proc_macros::expand_struct()]
pub struct EnvSettings {
    #[key = "DB_USERNAME"]
    #[value = "postgres"]
    #[description = "Database admin username"]
    pub username: String,

    #[key = "DB_PASSWORD"]
    #[value = "password"]
    #[description = "Database admin password"]
    pub password: String,

    #[key = "DB_NAME"]
    #[value = "mkwppdb"]
    #[description = "Database name"]
    pub database_name: String,

    #[key = "DB_HOST"]
    #[value = "localhost"]
    #[description = "Database IP Hostname"]
    pub host: String,

    #[key = "DB_PORT"]
    #[value = 5432]
    #[description = "Database IP Port"]
    pub port: u16,

    #[key = "DATABASE_URL"]
    #[value = "postgres://postgres:password@localhost:5432/mkwppdb"]
    #[description = "URL to the Database, can also be generated with the above keys"]
    pub database_url: String,

    #[key = "DB_MAX_CONN"]
    #[value = 25]
    #[description = "Connections in the Connection Pool"]
    pub max_conn: u32,

    #[key = "SRV_KEEP_ALIVE"]
    #[value = 60000]
    #[description = "Time for which a URL should hot reload, in milliseconds"]
    pub keep_alive: u64,

    #[key = "SRV_CLIENT_REQUEST_TIMEOUT"]
    #[value = 120000]
    #[description = "Max time a request should take before being dropped"]
    pub client_request_timeout: u64,

    #[key = "SRV_PORT"]
    #[value = 8080]
    #[description = "The open port for the server"]
    pub server_port: u16,

    #[key = "SRV_IP"]
    #[value = "127.0.0.1"]
    #[description = "The ip used by the server"]
    pub server_ip: String,

    #[key = "CACHE_TIMEOUT"]
    #[value = 1200]
    #[description = "Time it should take for each cache refresh loop"]
    pub cache_timeout: u64,

    #[key = "SMTP_HOST"]
    #[value = ""]
    #[description = "The hostname for the SMTP server"]
    pub smtp_hostname: String,

    #[key = "SMTP_PORT"]
    #[value = 25]
    #[description = "The port for the SMTP server"]
    pub smtp_port: u16,

    #[key = "SMTP_CREDS_NAME"]
    #[value = ""]
    #[description = "The credentials name for the SMTP client"]
    pub smtp_creds_name: String,

    #[key = "SMTP_CREDS_SECRET"]
    #[value = ""]
    #[description = "The credentials secret for the SMTP client"]
    pub smtp_creds_secret: String,

    #[key = "SMTP_TLS"]
    #[value = false]
    #[description = "Whether the TLS certificate for the SMTP server is valid or not"]
    pub smtp_tls_cert_valid: bool,
}

// run tests with
// cargo test -- --nocapture
// to see readme table in output
