pub mod awards;
pub mod champs;
pub mod cups;
pub mod edit_submissions;
pub mod players;
pub mod regions;
pub mod scores;
pub mod standard_levels;
pub mod standards;
pub mod submissions;
pub mod tracks;

#[derive(sqlx::Type, serde::Deserialize, Debug)]
#[sqlx(type_name = "category", rename_all = "lowercase")]
pub enum Category {
    NonSc,
    Sc,
    Unres,
}

impl TryFrom<u8> for Category {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return match value {
            0 => Ok(Self::NonSc),
            1 => Ok(Self::Sc),
            2 => Ok(Self::Unres),
            _ => Err(()),
        };
    }
}

impl Into<u8> for Category {
    fn into(self) -> u8 {
        return match self {
            Self::NonSc => 0,
            Self::Sc => 1,
            Self::Unres => 2,
        };
    }
}

impl<'a> Into<u8> for &'a Category {
    fn into(self) -> u8 {
        return match self {
            Category::NonSc => 0,
            Category::Sc => 1,
            Category::Unres => 2,
        };
    }
}

impl serde::Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.into())
    }
}

#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
#[sqlx(type_name = "submission_status", rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Accepted,
    Rejected,
    OnHold,
}

impl TryFrom<u8> for SubmissionStatus {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Accepted),
            2 => Ok(Self::Rejected),
            3 => Ok(Self::OnHold),
            _ => Err(()),
        };
    }
}

pub trait BasicTableQueries {
    fn table_name() -> &'static str;

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(&format!("SELECT * from {};", Self::table_name()))
            .fetch_all(executor)
            .await;
    }
}
