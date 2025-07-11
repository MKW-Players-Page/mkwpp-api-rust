use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

pub mod awards;
pub mod blog_posts;
pub mod champs;
pub mod cups;
pub mod players;
pub mod regions;
pub mod scores;
pub mod standard_levels;
pub mod standards;
pub mod submissions;
pub mod tracks;

#[derive(sqlx::Type, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "category", rename_all = "lowercase")]
#[derive(Default)]
pub enum Category {
    #[default]
    NonSc,
    Sc,
    Unres,
}

impl TryFrom<u8> for Category {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NonSc),
            1 => Ok(Self::Sc),
            2 => Ok(Self::Unres),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Category {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "nonsc" | "nosc" | "normal" | "non-sc" | "non_sc" | "no-shortcut" | "noshortcut"
            | "n" => Ok(Self::NonSc),
            "shortcut" | "sc" | "s" => Ok(Self::Sc),
            "unrestricted" | "unres" | "unr" | "glitch" | "g" | "u" => Ok(Self::Unres),
            _ => Err(()),
        }
    }
}

impl From<Category> for u8 {
    fn from(val: Category) -> Self {
        match val {
            Category::NonSc => 0,
            Category::Sc => 1,
            Category::Unres => 2,
        }
    }
}

impl From<&Category> for u8 {
    fn from(val: &Category) -> Self {
        match val {
            Category::NonSc => 0,
            Category::Sc => 1,
            Category::Unres => 2,
        }
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

impl<'de> serde::Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde_json::Number::deserialize(deserializer).and_then(|x| {
            x.as_u64()
                .ok_or_else(|| {
                    serde::de::Error::invalid_type(
                        if x.is_f64() {
                            serde::de::Unexpected::Float(x.as_f64().unwrap())
                        } else if x.is_i64() {
                            serde::de::Unexpected::Signed(x.as_i64().unwrap())
                        } else {
                            serde::de::Unexpected::Other("integer")
                        },
                        &"u8 < 3",
                    )
                })
                .and_then(|x| {
                    Category::try_from(x as u8).map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(x),
                            &"u8 < 3",
                        )
                    })
                })
        })
    }
}

pub trait BasicTableQueries {
    const TABLE_NAME: &'static str;

    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(&format!("SELECT * FROM {};", Self::TABLE_NAME))
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    async fn delete_by_id(
        id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query(&format!("DELETE FROM {} WHERE id = $1;", Self::TABLE_NAME))
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
