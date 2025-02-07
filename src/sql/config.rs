use std::io::Write;

const CFG_PATH: &str = "./sql.cfg";

#[derive(Debug)]
pub struct PostgresConfig {
    username: String,
    password: String,
    database_name: String,
    host: String,
    port: String,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        PostgresConfig {
            username: String::from("postgres"),
            password: String::from("password"),
            database_name: String::from("mkwppdb"),
            host: String::from("localhost"),
            port: String::from("5432"),
        }
    }
}

impl ToString for PostgresConfig {
    fn to_string(&self) -> String {
        format!(
            "username = {}\npassword = {}\ndatabase_name = {}\nhost = {}\nport = {}",
            self.username, self.password, self.database_name, self.host, self.port,
        )
    }
}

impl PostgresConfig {
    pub fn load_from_file() -> Self {
        if !std::path::Path::new(CFG_PATH).is_file() {
            create_config();
        }

        let mut config = PostgresConfig::default();

        let file_string = match std::fs::read_to_string(CFG_PATH) {
            Ok(v) => v,
            Err(e) => {
                println!("Error reading Config");
                println!("{e}");
                std::process::exit(0);
            }
        };

        for line in file_string.split('\n') {
            if !line.contains('=') {
                continue;
            }

            let mut kv_split = line.split('=');
            let key = match kv_split.next() {
                None => continue,
                Some(v) => v.trim(),
            };
            let value = match kv_split.next() {
                None => continue,
                Some(v) => v.trim(),
            };

            match key {
                "username" => config.username = String::from(value),
                "password" => config.password = String::from(value),
                "database_name" => config.database_name = String::from(value),
                "host" => config.host = String::from(value),
                "port" => config.port = String::from(value),
                _ => continue,
            }
        }

        return config;
    }
}

fn create_config() {
    println!("Creating Config");
    let mut new_cfg_file = match std::fs::File::create_new(CFG_PATH) {
        Ok(v) => v,
        Err(e) => {
            println!("Error creating Config");
            println!("{e}");
            std::process::exit(0);
        }
    };

    println!("");
    println!("If you don't know what to do:");
    println!("You should log into the postgres user on your terminal");
    println!("Then create a new OS user with the createuser command");
    println!("$ createuser username");
    println!("Then create a new database with the createdb command");
    println!("$ createdb database_name");
    println!("After that, enter the psql shell via the psql command");
    println!("Then pass the user you created all perms, with the following PSQL commands:");
    println!("$ alter user username with encrypted password 'password';");
    println!("$ grant all privileges on database database_name to username;");
    println!("Finally, insert the new data into the {CFG_PATH} file.");
    println!("");
    println!("Exiting the process");

    match new_cfg_file.write_all(PostgresConfig::default().to_string().as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            println!("Error creating Config");
            println!("{e}");
            std::process::exit(0);
        }
    };

    std::process::exit(0);
}
