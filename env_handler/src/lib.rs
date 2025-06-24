use dotenvy::dotenv;
use std::{fmt::Display, io::Write};

pub const DEFAULT_SCHEMA: EnvSettingsSchema = EnvSettingsSchema {
    username: EnvSettingsSchemaField {
        key: "DB_USERNAME",
        type_name: "String",
        value: "postgres",
        description: "Database admin username",
    },
    password: EnvSettingsSchemaField {
        key: "DB_PASSWORD",
        type_name: "String",
        value: "password",
        description: "Database admin password",
    },
    database_name: EnvSettingsSchemaField {
        key: "DB_NAME",
        type_name: "String",
        value: "mkwppdb",
        description: "Database name",
    },
    host: EnvSettingsSchemaField {
        key: "DB_HOST",
        type_name: "String",
        value: "localhost",
        description: "Database IP Hostname",
    },
    port: EnvSettingsSchemaField {
        key: "DB_PORT",
        type_name: "u16",
        value: 5432,
        description: "Database IP Port",
    },
    database_url: EnvSettingsSchemaField {
        key: "DATABASE_URL",
        type_name: "String",
        value: "postgres://postgres:password@localhost:5432/mkwppdb",
        description: "URL to the Database, can also be generated with the above keys",
    },
    max_conn: EnvSettingsSchemaField {
        key: "DB_MAX_CONN",
        type_name: "u32",
        value: 25,
        description: "Connections in the Connection Pool",
    },
    keep_alive: EnvSettingsSchemaField {
        key: "SRV_KEEP_ALIVE",
        type_name: "u64",
        value: 60000,
        description: "Time for which a URL should hot reload, in milliseconds",
    },
    client_request_timeout: EnvSettingsSchemaField {
        key: "SRV_CLIENT_REQUEST_TIMEOUT",
        type_name: "u64",
        value: 120000,
        description: "Max time a request should take before being dropped",
    },
    cache_timeout: EnvSettingsSchemaField {
        key: "CACHE_TIMEOUT",
        type_name: "u64",
        value: 1200,
        description: "Time it should take for each cache refresh loop",
    },
    smtp_port: EnvSettingsSchemaField {
        key: "SMTP_PORT",
        type_name: "u16",
        value: 25,
        description: "The port for the SMTP server",
    },
    smtp_hostname: EnvSettingsSchemaField {
        key: "SMTP_HOST",
        type_name: "String",
        value: String::new(),
        description: "The hostname for the SMTP server",
    },
    smtp_creds_name: EnvSettingsSchemaField {
        key: "SMTP_CREDS_NAME",
        type_name: "String",
        value: String::new(),
        description: "The credentials name for the SMTP client",
    },
    smtp_creds_secret: EnvSettingsSchemaField {
        key: "SMTP_CREDS_SECRET",
        type_name: "String",
        value: String::new(),
        description: "The credentials secret for the SMTP client",
    },
    smtp_tls_cert_valid: EnvSettingsSchemaField {
        key: "SMTP_TLS",
        type_name: "bool",
        value: false,
        description: "Whether the TLS certificate for the SMTP server is valid or not",
    },
};

struct EnvSettingsSchemaField<T: Display> {
    key: &'static str,
    value: T,
    type_name: &'static str,
    description: &'static str,
}

impl<T: Display> EnvSettingsSchemaField<T> {
    fn to_readme_line(&self) -> String {
        format!(
            "\n| {} | {} | {} | {} |",
            self.key, self.type_name, self.description, self.value,
        )
    }
}

pub struct EnvSettingsSchema {
    username: EnvSettingsSchemaField<&'static str>,
    password: EnvSettingsSchemaField<&'static str>,
    database_name: EnvSettingsSchemaField<&'static str>,
    host: EnvSettingsSchemaField<&'static str>,
    port: EnvSettingsSchemaField<u16>,
    database_url: EnvSettingsSchemaField<&'static str>,
    max_conn: EnvSettingsSchemaField<u32>,
    keep_alive: EnvSettingsSchemaField<u64>,
    client_request_timeout: EnvSettingsSchemaField<u64>,
    cache_timeout: EnvSettingsSchemaField<u64>,
    smtp_hostname: EnvSettingsSchemaField<String>,
    smtp_port: EnvSettingsSchemaField<u16>,
    smtp_creds_name: EnvSettingsSchemaField<String>,
    smtp_creds_secret: EnvSettingsSchemaField<String>,
    smtp_tls_cert_valid: EnvSettingsSchemaField<bool>,
}

impl EnvSettingsSchema {
    pub fn to_readme_table(&self) -> String {
        let mut out = String::from("| Key | Value Type | Description | Default |\n|-|-|-|-|");
        out += &self.username.to_readme_line();
        out += &self.password.to_readme_line();
        out += &self.database_name.to_readme_line();
        out += &self.host.to_readme_line();
        out += &self.port.to_readme_line();
        out += &self.database_url.to_readme_line();
        out += &self.max_conn.to_readme_line();
        out += &self.keep_alive.to_readme_line();
        out += &self.client_request_timeout.to_readme_line();
        out += &self.cache_timeout.to_readme_line();
        out += &self.smtp_hostname.to_readme_line();
        out += &self.smtp_port.to_readme_line();
        out += &self.smtp_creds_name.to_readme_line();
        out += &self.smtp_creds_secret.to_readme_line();
        out += &self.smtp_tls_cert_valid.to_readme_line();
        out
    }
}

pub struct EnvSettings {
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub max_conn: u32,
    pub keep_alive: u64,
    pub client_request_timeout: u64,
    pub cache_timeout: u64,
    pub smtp_hostname: String,
    pub smtp_port: u16,
    pub smtp_creds_name: String,
    pub smtp_creds_secret: String,
    pub smtp_tls_cert_valid: bool,
}

impl EnvSettings {
    fn generate_url(&mut self) {
        self.database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        );
    }

    pub fn from_env_vars() -> Result<Self, anyhow::Error> {
        dotenv()?;
        let mut out = Self {
            username: std::env::var(DEFAULT_SCHEMA.username.key)
                .unwrap_or(DEFAULT_SCHEMA.username.value.to_string()),
            password: std::env::var(DEFAULT_SCHEMA.password.key)
                .unwrap_or(DEFAULT_SCHEMA.password.value.to_string()),
            database_name: std::env::var(DEFAULT_SCHEMA.database_name.key)
                .unwrap_or(DEFAULT_SCHEMA.database_name.value.to_string()),
            host: std::env::var(DEFAULT_SCHEMA.host.key)
                .unwrap_or(DEFAULT_SCHEMA.host.value.to_string()),
            port: std::env::var(DEFAULT_SCHEMA.port.key).map_or(DEFAULT_SCHEMA.port.value, |x| {
                x.parse::<u16>().unwrap_or(DEFAULT_SCHEMA.port.value)
            }),
            database_url: DEFAULT_SCHEMA.database_url.value.to_string(),
            max_conn: std::env::var(DEFAULT_SCHEMA.max_conn.key)
                .map_or(DEFAULT_SCHEMA.max_conn.value, |v| {
                    v.parse::<u32>().unwrap_or(DEFAULT_SCHEMA.max_conn.value)
                }),
            keep_alive: std::env::var(DEFAULT_SCHEMA.keep_alive.key)
                .map_or(DEFAULT_SCHEMA.keep_alive.value, |x| {
                    x.parse::<u64>().unwrap_or(DEFAULT_SCHEMA.keep_alive.value)
                }),
            client_request_timeout: std::env::var(DEFAULT_SCHEMA.client_request_timeout.key)
                .map_or(DEFAULT_SCHEMA.client_request_timeout.value, |x| {
                    x.parse::<u64>()
                        .unwrap_or(DEFAULT_SCHEMA.client_request_timeout.value)
                }),
            cache_timeout: std::env::var(DEFAULT_SCHEMA.cache_timeout.key).map_or(
                DEFAULT_SCHEMA.cache_timeout.value,
                |x| {
                    x.parse::<u64>()
                        .unwrap_or(DEFAULT_SCHEMA.cache_timeout.value)
                },
            ),
            smtp_hostname: std::env::var(DEFAULT_SCHEMA.smtp_hostname.key)
                .map_or(DEFAULT_SCHEMA.smtp_hostname.value, |x| x.to_string()),
            smtp_port: std::env::var(DEFAULT_SCHEMA.smtp_port.key)
                .map_or(DEFAULT_SCHEMA.smtp_port.value, |x| {
                    x.parse().unwrap_or(DEFAULT_SCHEMA.smtp_port.value)
                }),
            smtp_creds_name: std::env::var(DEFAULT_SCHEMA.smtp_creds_name.key)
                .map_or(DEFAULT_SCHEMA.smtp_creds_name.value, |x| x.to_string()),
            smtp_creds_secret: std::env::var(DEFAULT_SCHEMA.smtp_creds_secret.key)
                .map_or(DEFAULT_SCHEMA.smtp_creds_secret.value, |x| x.to_string()),
            smtp_tls_cert_valid: std::env::var(DEFAULT_SCHEMA.smtp_tls_cert_valid.key).map_or(
                DEFAULT_SCHEMA.smtp_tls_cert_valid.value,
                |x| {
                    x.parse()
                        .unwrap_or(DEFAULT_SCHEMA.smtp_tls_cert_valid.value)
                },
            ),
        };
        out.generate_url();

        Ok(out)
    }

    pub fn to_env_file(&self, file: &mut std::fs::File) -> Result<(), anyhow::Error> {
        let mut out = std::io::LineWriter::new(file);
        writeln!(out, "{}={}", DEFAULT_SCHEMA.username.key, self.username)?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.password.key, self.password)?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.database_name.key, self.database_name
        )?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.host.key, self.host)?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.port.key, self.port)?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.database_url.key, self.database_url
        )?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.max_conn.key, self.max_conn)?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.keep_alive.key, self.keep_alive)?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.client_request_timeout.key, self.client_request_timeout
        )?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.cache_timeout.key, self.cache_timeout
        )?;
        writeln!(out, "{}={}", DEFAULT_SCHEMA.smtp_port.key, self.smtp_port)?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.smtp_hostname.key, self.smtp_hostname
        )?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.smtp_creds_name.key, self.smtp_creds_name
        )?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.smtp_creds_secret.key, self.smtp_creds_secret
        )?;
        writeln!(
            out,
            "{}={}",
            DEFAULT_SCHEMA.smtp_tls_cert_valid.key, self.smtp_tls_cert_valid
        )?;

        Ok(())
    }
}

mod test {
    #[test]
    fn generate_readme_table() {
        // run tests with
        // cargo test -- --nocapture
        // to see it in output
        println!("{}", super::DEFAULT_SCHEMA.to_readme_table());
    }
}
