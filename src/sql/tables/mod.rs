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

#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
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
