#[derive(serde::Deserialize, Debug)]
pub struct Tracks {
    abbr: String,
    cup: i32,
    categories: String,
}

#[async_trait::async_trait]
impl super::OldFixtureJson for Tracks {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return crate::sql::tables::tracks::Tracks {
            id: key,
            categories: self
                .categories
                .split(',')
                .map(|v| crate::sql::tables::Category::try_from(v.parse::<u8>().unwrap()).unwrap())
                .collect(),
            abbr: self.abbr,
            cup_id: self.cup,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
