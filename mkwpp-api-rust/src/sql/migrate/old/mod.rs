mod awards;
mod champs;
mod edit_submissions;
mod players;
mod regions;
mod scores;
mod standard_levels;
mod standards;
mod submissions;
mod tracks;

const OLD_FIXTURES_PATH: &str = "./db/fixtures/old/";

fn enforce_file_order(file_name: &str) -> u8 {
    match file_name {
        "regions.json" => return 0,
        "players.json" => return 1,
        "trackcups.json" => return 2,
        "tracks.json" => return 3,
        "scores.json" => return 4,
        "scoresubmissions.json" => return 5,
        "editscoresubmissions.json" => return 6,
        "standardlevels.json" => return 7,
        "standards.json" => return 8,
        "sitechampions.json" => return 9,
        "playerawards.json" => return 10,
        _ => return 11,
    }
}

pub async fn load_data(pool: &sqlx::Pool<sqlx::Postgres>) {
    let transaction = pool.begin();

    let mut file_paths = match std::fs::read_dir(std::path::Path::new(OLD_FIXTURES_PATH)) {
        Err(e) => {
            println!("Error reading folder for fixtures");
            println!("{e}");
            println!();
            println!("Exiting the process");
            std::process::exit(0);
        }
        Ok(dir_read) => dir_read
            .into_iter()
            .filter_map(|dir_entry_result| match dir_entry_result {
                Err(e) => {
                    println!("Error reading file from folder for fixtures");
                    println!("{e}");
                    println!();
                    println!("Exiting the process");
                    std::process::exit(0);
                }
                Ok(file) => match file.file_name().to_str() {
                    None => {
                        println!("Error reading file path from folder for fixtures");
                        println!();
                        println!("Exiting the process");
                        std::process::exit(0);
                    }
                    Some(path) => {
                        if !path.ends_with(".json") {
                            return None;
                        }
                        return Some(String::from(path));
                    }
                },
            })
            .collect::<Vec<String>>(),
    };

    file_paths.sort_by(|a, b| return enforce_file_order(a).cmp(&enforce_file_order(b)));

    let mut transaction = match transaction.await {
        Ok(v) => v,
        Err(e) => {
            println!("Couldn't start Postgres Transaction");
            println!("{e}");
            println!();
            println!("Exiting the process");
            std::process::exit(0);
        }
    };

    for file_name in file_paths {
        println!("Loading fixture {file_name}");
        if let Err(e) = match file_name.as_str() {
            "regions.json" => {
                regions::Regions::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "players.json" => {
                players::Players::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "tracks.json" => {
                tracks::Tracks::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "scores.json" => {
                scores::Scores::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "scoresubmissions.json" => {
                println!("Fixture file skipped because it can't be imported");
                continue;
                // submissions::Submissions::read_file(
                //     &file_name,
                //     &mut String::new(),
                //     &mut transaction,
                // )
                // .await
            }
            "editscoresubmissions.json" => {
                println!("Fixture file skipped because it can't be imported");
                continue;
                // edit_submissions::EditSubmissions::read_file(
                //     &file_name,
                //     &mut String::new(),
                //     &mut transaction,
                // )
                // .await
            }
            "standardlevels.json" => {
                standard_levels::StandardLevels::read_file(
                    &file_name,
                    &mut String::new(),
                    &mut transaction,
                )
                .await
            }
            "standards.json" => {
                standards::Standards::read_file(&file_name, &mut String::new(), &mut transaction)
                    .await
            }
            "sitechampions.json" => {
                champs::Champs::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "playerawards.json" => {
                awards::Awards::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            _ => {
                println!("Fixture file skipped");
                continue;
            }
        } {
            println!("Error reading data. Rolling back transaction.");
            println!("{e}");
            match transaction.rollback().await {
                Ok(_) => std::process::exit(0),
                Err(e) => println!("Error rolling back transaction. You're fucked. :)\n{e}"),
            };
            std::process::exit(0);
        }
    }
    match transaction.commit().await {
        Ok(_) => println!("Transaction went through!"),
        Err(e) => {
            println!("Transaction failed.\n{e}\nExiting the process");
            std::process::exit(0)
        }
    }
}

trait OldFixtureJson: std::fmt::Debug {
    // buffer is because of lifetime. You can't declare the string within the function sadly enough.
    async fn read_file<'d>(
        file_name: &str,
        buffer: &'d mut String,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized + serde::Deserialize<'d> + std::marker::Sync + std::marker::Send,
    {
        let file_path = format!("{OLD_FIXTURES_PATH}{file_name}");
        match std::fs::read_to_string(&file_path) {
            Ok(v) => *buffer = v,
            Err(e) => {
                println!("Error reading file {file_path} from old fixtures");
                println!("{e}");
                println!();
                println!("Exiting the process");
                std::process::exit(0);
            }
        };
        let mut vec: Vec<OldFixtureWrapper<Self>> = match serde_json::from_str(buffer) {
            Ok(v) => v,
            Err(e) => {
                println!("Error converting fixture {file_path} from JSON");
                println!("{e}");
                println!();
                println!("Exiting the process");
                std::process::exit(0);
            }
        };

        vec.sort_by(Self::get_sort());

        for wrapper in vec {
            wrapper.add_to_db(transaction).await?;
        }

        return Ok(());
    }

    fn get_sort()
    -> impl FnMut(&OldFixtureWrapper<Self>, &OldFixtureWrapper<Self>) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        return |a, b| return a.pk.cmp(&b.pk);
    }

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error>;
}

#[derive(serde::Deserialize, Debug)]
struct OldFixtureWrapper<T: OldFixtureJson> {
    pk: i32,
    fields: T,
}

impl<T: OldFixtureJson + std::marker::Sync + std::marker::Send> OldFixtureWrapper<T> {
    async fn add_to_db(
        self,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        self.fields.add_to_db(self.pk, transaction).await
    }
}
