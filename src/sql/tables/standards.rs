#[derive(serde::Deserialize, Debug)]
pub struct Standards {
    pub id: i32,
    pub standard_level_id: i32,
    pub track_id: i32,
    pub category: super::Category,
    pub is_lap: bool,
    pub value: Option<i32>,
}

impl super::BasicTableQueries for Standards {
    fn table_name() -> &'static str {
        return "standards";
    }
}

impl Standards {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO standards (id, standard_level_id, track_id, category, is_lap, value) VALUES($1, $2, $3, $4, $5, $6);").bind(self.id).bind(self.standard_level_id).bind(self.track_id).bind(&self.category).bind(self.is_lap).bind(self.value).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO standards (id, standard_level_id, track_id, category, is_lap, value) VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET standard_level_id = $2, track_id = $3, category = $4, is_lap = $5, value = $6 WHERE standards.id = $1;").bind(self.id).bind(self.standard_level_id).bind(self.track_id).bind(&self.category).bind(self.is_lap).bind(self.value).execute(executor).await;
    }
}
