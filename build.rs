fn main() -> Result<(), anyhow::Error> {
    run_migrations()?;
    // Do stuff to generate docs here

    return Ok(());
}

fn run_migrations() -> Result<(), anyhow::Error> {
    let migrations = std::process::Command::new("sqlx")
        .args(["database", "setup", "--source", "db/migrations"])
        .output()
        .expect("Failed to run migrations");
    if !migrations.status.success() {
        anyhow::bail!("Couldn't run SQLX");
    }
    return Ok(()); 
}

fn sql_env_valid() -> Result<(), anyhow::Error> {
    let migrations = std::process::Command::new("sqlx")
        .args(["database", "setup", "--source", "db/migrations"])
        .output()
        .expect("Failed to run migrations");
    if !migrations.status.success() {
        anyhow::bail!("Couldn't run SQLX");
    }
    return Ok(()); 
}
