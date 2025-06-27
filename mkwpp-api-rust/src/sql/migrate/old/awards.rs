use crate::api::errors::FinalErrorResponse;

#[derive(serde::Deserialize, Debug)]
pub struct Awards {
    player: i32,
    date: String,
    description: String,
    #[serde(rename = "type")]
    award_type: String,
}

impl super::OldFixtureJson for Awards {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::awards::Awards {
            id: key,
            player_award_type: crate::sql::tables::awards::AwardType::try_from(
                self.award_type.as_str(),
            )
            .unwrap(),
            date: chrono::NaiveDate::parse_from_str(&self.date, "%F").unwrap(),
            description: self.description,
            player_id: self.player,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
