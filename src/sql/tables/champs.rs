#[derive(serde::Deserialize, Debug)]
pub struct Champs {
    pub id: i32,
    pub player_id: i32,
    pub category: super::Category,
    pub date_instated: chrono::NaiveDate,
}

impl Champs {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO site_champs (id, player_id, category, date_instated) VALUES($1, $2, $3, $4);").bind(self.id).bind(self.player_id).bind(&self.category).bind(self.date_instated).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO site_champs (id, player_id, category, date_instated) VALUES($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET player_id = $2, category = $3, date_instated = $4 WHERE site_champs.id = $1;").bind(self.id).bind(self.player_id).bind(&self.category).bind(self.date_instated).execute(executor).await;
    }
}
