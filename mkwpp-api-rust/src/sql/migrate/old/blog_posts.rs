use crate::api::errors::FinalErrorResponse;

#[derive(serde::Deserialize, Debug)]
pub struct BlogPosts {
    player: i32,
    date_instated: String,
    category: u8,
}

impl super::OldFixtureJson for Champs {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::champs::Champs {
            id: key,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            date_instated: chrono::NaiveDate::parse_from_str(
                self.date_instated.split('T').next().unwrap(),
                "%F",
            )
            .unwrap(),
            player_id: self.player,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
