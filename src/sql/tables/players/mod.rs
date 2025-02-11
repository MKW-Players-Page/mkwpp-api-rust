pub mod players_basic;

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Players {
    pub id: i32,
    pub name: String,
    pub alias: Option<String>,
    pub bio: Option<String>,
    pub region_id: i32,
    pub joined_date: chrono::NaiveDate,
    pub last_activity: chrono::NaiveDate,
}

impl super::BasicTableQueries for Players {
    fn table_name() -> &'static str {
        return "players";
    }
}

impl Players {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity) VALUES($1, $2, $3, $4, $5, $6, $7);").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(&self.region_id).bind(self.joined_date).bind(self.last_activity).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity) VALUES($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET name = $2, alias = $3, bio = $4, region_id = $5, joined_date = $6, last_activity = $7 WHERE players.id = $1;").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(self.region_id).bind(self.joined_date).bind(self.last_activity).execute(executor).await;
    }
}
