#[derive(serde::Deserialize, Debug, sqlx::FromRow, serde::Serialize)]
pub struct Tracks {
    pub id: i32,
    pub abbr: String,
    pub cup_id: i32,
    pub categories: Vec<super::Category>,
}

impl Tracks {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO tracks (id, abbr, cup_id, categories) VALUES($1, $2, $3, $4);")
    //         .bind(self.id)
    //         .bind(&self.abbr)
    //         .bind(self.cup_id)
    //         .bind(&self.categories)
    //         .execute(executor)
    //         .await
    // }

    pub async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query("SELECT * FROM tracks;")
            .fetch_all(executor)
            .await;
    }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO tracks (id, abbr, cup_id, categories) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET abbr = $2, cup_id = $3, categories = $4 WHERE tracks.id = $1;").bind(self.id).bind(&self.abbr).bind(self.cup_id).bind(&self.categories).execute(executor).await;
    }
}
