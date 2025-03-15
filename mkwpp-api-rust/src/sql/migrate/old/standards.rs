#[derive(serde::Deserialize, Debug)]
pub struct Standards {
    level: i32,
    track: i32,
    category: u8,
    is_lap: bool,
    value: Option<i32>,
}

impl super::OldFixtureJson for Standards {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return crate::sql::tables::standards::Standards {
            id: key,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            standard_level_id: self.level,
            track_id: self.track,
            is_lap: self.is_lap,
            value: self.value,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
