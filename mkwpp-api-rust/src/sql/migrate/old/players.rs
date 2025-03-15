#[derive(serde::Deserialize, Debug)]
pub struct Players {
    name: String,
    alias: Option<String>,
    bio: Option<String>,
    region: Option<i32>,
    joined_date: Option<String>,
    last_activity: Option<String>,
}

impl super::OldFixtureJson for Players {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let joined_date = self
            .joined_date
            .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap())
            .unwrap_or_default();

        return crate::sql::tables::players::Players {
            id: key,
            name: self.name,
            alias: self.alias,
            bio: self.bio,
            region_id: self.region.unwrap_or(1),
            joined_date,
            last_activity: self
                .last_activity
                .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap())
                .unwrap_or(joined_date),
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
