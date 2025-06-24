#[derive(serde::Deserialize, Debug)]
pub struct StandardLevels {
    code: String,
    value: i32,
    is_legacy: bool,
}

impl super::OldFixtureJson for StandardLevels {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return crate::sql::tables::standard_levels::StandardLevels {
            id: key,
            code: self.code,
            value: self.value,
            is_legacy: self.is_legacy,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
