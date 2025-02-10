#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
#[sqlx(type_name = "player_award_type", rename_all = "snake_case")]
pub enum AwardType {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl TryFrom<&str> for AwardType {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        return match value {
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "yearly" => Ok(Self::Yearly),
            _ => Err(()),
        };
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Awards {
    pub id: i32,
    pub player_id: i32,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub player_award_type: AwardType,
}

impl super::BasicTableQueries for Awards {
    fn table_name() -> &'static str {
        return "player_awards";
    }
}

impl Awards {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO player_awards (id, player_id, date, description, player_award_type) VALUES($1, $2, $3, $4, $5);").bind(self.id).bind(self.player_id).bind(self.date).bind(&self.description).bind(&self.player_award_type).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO player_awards (id, player_id, date, description, player_award_type) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET player_id = $2, date = $3, description = $4, player_award_type = $5 WHERE player_awards.id = $1;").bind(self.id).bind(self.player_id).bind(self.date).bind(&self.description).bind(&self.player_award_type).execute(executor).await;
    }
}
