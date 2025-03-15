use sqlx::migrate::MigrateDatabase;
use std::env::set_current_dir;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // build.rs sets current dir to its own directory
    set_current_dir("..")?;

    sql_env_valid().await?;
    run_migrations()?;

    // Do stuff to generate docs here

    return Ok(());
}

async fn sql_env_valid() -> Result<(), anyhow::Error> {
    if !std::path::Path::new("./.env").is_file() {
        let mut file = std::fs::File::create("./.env")?;
        env_handler::EnvSettings::from_env_vars()?.to_env_file(&mut file)?;
    }

    let env_vars = env_handler::EnvSettings::from_env_vars()?;

    // if the env vars were not valid it would fail here
    if !sqlx::Postgres::database_exists(&env_vars.database_url).await? {
        sqlx::Postgres::create_database(&env_vars.database_url).await?;
        anyhow::bail!(
            "Database doesn't exist! The program created it as {}, go set it up!",
            env_vars.database_name
        );
    }

    return Ok(());
}

fn run_migrations() -> Result<(), anyhow::Error> {
    let env_vars = env_handler::EnvSettings::from_env_vars()?;

    let migrations = std::process::Command::new("sqlx")
        .args([
            "database",
            "setup",
            "--source",
            "db/migrations",
            "--database-url",
            &env_vars.database_url,
        ])
        .output()?;
    if !migrations.status.success() {
        anyhow::bail!("Couldn't run SQLX");
    }
    return Ok(());
}
