use crate::custom_serde::DateAsTimestampNumber;

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
