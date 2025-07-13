use crate::api::errors::FinalErrorResponse;

mod awards;
mod blog_posts;
mod champs;
mod players;
mod regions;
mod scores;
mod standard_levels;
mod standards;
mod tracks;

const PATH: &str = "./db/fixtures/old/";

use super::TABLE_NAMES;

use awards::Awards;
use blog_posts::BlogPosts;
use champs::Champs;
use players::Players;
use regions::Regions;
use scores::Scores;
use standard_levels::StandardLevels;
use standards::Standards;
use tracks::Tracks;

macro_rules! call_fn {
    ($structName: ident, $transaction: ident) => {
        $structName::read_file(&mut $transaction).await
    };
}

pub async fn import_data(pool: &sqlx::Pool<sqlx::Postgres>) {
    let mut transaction = pool
        .begin()
        .await
        .expect("Couldn't start Postgres Transaction");

    for table_name in TABLE_NAMES.into_iter() {
        if table_name == "edit_submissions" || table_name == "submissions" || table_name == "users"
        {
            continue;
        }

        println!("Loading fixture for {table_name}");

        let result: Result<(), FinalErrorResponse> = match table_name {
            "regions" => call_fn!(Regions, transaction),
            "players" => call_fn!(Players, transaction),
            "tracks" => call_fn!(Tracks, transaction),
            "scores" => call_fn!(Scores, transaction),
            "blog_posts" => call_fn!(BlogPosts, transaction),
            "standard_levels" => call_fn!(StandardLevels, transaction),
            "standards" => call_fn!(Standards, transaction),
            "site_champs" => call_fn!(Champs, transaction),
            "player_awards" => call_fn!(Awards, transaction),
            _ => {
                println!("Fixture file skipped");
                continue;
            }
        };

        result.expect("Couldn't load data");
    }

    super::reset_sequences(&mut transaction).await;

    transaction
        .commit()
        .await
        .expect("Transaction didn't go through");
}

trait OldFixtureJson: std::fmt::Debug {
    const FILENAME: &str;

    // buffer is because of lifetime. You can't declare the string within the function sadly enough.
    async fn read_file(transaction: &mut sqlx::PgConnection) -> Result<(), FinalErrorResponse>
    where
        Self: for<'d> serde::Deserialize<'d> + std::marker::Sync + std::marker::Send,
    {
        let file_path = format!("{PATH}{}", Self::FILENAME);
        let string =
            std::fs::read_to_string(&file_path).expect("Error reading file from old fixtures");
        let mut vec: Vec<OldFixtureWrapper<Self>> =
            serde_json::from_str(&string).expect("Error converting fixture from JSON");

        vec.sort_by(Self::get_sort());

        for wrapper in vec {
            wrapper.add_to_db(transaction).await?;
        }

        Ok(())
    }

    fn get_sort()
    -> impl FnMut(&OldFixtureWrapper<Self>, &OldFixtureWrapper<Self>) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        |a, b| a.pk.cmp(&b.pk)
    }

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse>;
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
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        self.fields.add_to_db(self.pk, transaction).await
    }
}
