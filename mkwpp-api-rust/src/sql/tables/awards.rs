use crate::custom_serde::DateAsTimestampNumber;

// Feature only required because it's only used to import data currently
#[cfg(feature = "import_data_old")]
use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

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
        match value {
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "yearly" => Ok(Self::Yearly),
            _ => Err(()),
        }
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Awards {
    pub id: i32,
    pub player_id: i32,
    #[serde(
        serialize_with = "DateAsTimestampNumber::serialize_as_timestamp",
        deserialize_with = "DateAsTimestampNumber::deserialize_from_timestamp"
    )]
    pub date: chrono::NaiveDate,
    pub description: String,
    pub player_award_type: AwardType,
}

impl super::BasicTableQueries for Awards {
    const TABLE_NAME: &'static str = "player_awards";
}

impl Awards {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO player_awards (id, player_id, date, description, player_award_type) VALUES($1, $2, $3, $4, $5);").bind(self.id).bind(self.player_id).bind(self.date).bind(&self.description).bind(&self.player_award_type).execute(executor).await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data_old")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO player_awards (id, player_id, date, description, player_award_type) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET player_id = $2, date = $3, description = $4, player_award_type = $5 WHERE player_awards.id = $1;").bind(self.id).bind(self.player_id).bind(self.date).bind(&self.description).bind(&self.player_award_type).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
