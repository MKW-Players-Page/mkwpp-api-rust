use std::{env::set_current_dir, io::Write};

fn main() -> Result<(), anyhow::Error> {
    // build.rs sets current dir to its own directory
    set_current_dir("..")?;

    run_migrations()?;
    sql_env_valid()?;

    // Do stuff to generate docs here

    return Ok(());
}

fn run_migrations() -> Result<(), anyhow::Error> {
    let migrations = std::process::Command::new("sqlx")
        .args(["database", "setup", "--source", "db/migrations"])
        .output()?;
    if !migrations.status.success() {
        anyhow::bail!("Couldn't run SQLX");
    }
    return Ok(());
}

fn sql_env_valid() -> Result<(), anyhow::Error> {
    if !std::path::Path::new("./.env").is_file(){
        let mut file = std::fs::File::create("./.env")?;
        file.write_all("".as_bytes())?;
    }
    
    

    return Ok(());
}
