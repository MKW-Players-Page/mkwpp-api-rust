use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

use super::BasicTableQueries;

pub mod tree;
pub mod with_player_count;

#[derive(sqlx::Type, Debug, Hash, PartialEq, Eq, Clone)]
#[sqlx(type_name = "region_type", rename_all = "snake_case")]
pub enum RegionType {
    World,
    Continent,
    CountryGroup,
    Country,
    SubnationalGroup,
    Subnational,
}

impl TryFrom<&str> for RegionType {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "world" | "World" | "WORLD" => Ok(Self::World),
            "continent" | "Continent" | "CONTINENT" => Ok(Self::Continent),
            "country_group" | "COUNTRYGROUP" | "COUNTRY_GROUP" | "CountryGroup"
            | "countryGroup" => Ok(Self::CountryGroup),
            "country" | "Country" | "COUNTRY" => Ok(Self::Country),
            "subnational_group" | "SUBNATIONALGROUP" | "SUBNATIONAL_GROUP" | "SubnationalGroup"
            | "subnationalGroup" => Ok(Self::SubnationalGroup),
            "subnational" | "Subnational" | "SUBNATIONAL" => Ok(Self::Subnational),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for RegionType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegionType::World),
            1 => Ok(RegionType::Continent),
            2 => Ok(RegionType::CountryGroup),
            3 => Ok(RegionType::Country),
            4 => Ok(RegionType::SubnationalGroup),
            5 => Ok(RegionType::Subnational),
            _ => Err(()),
        }
    }
}

impl From<RegionType> for u8 {
    fn from(val: RegionType) -> Self {
        match val {
            RegionType::World => 0,
            RegionType::Continent => 1,
            RegionType::CountryGroup => 2,
            RegionType::Country => 3,
            RegionType::SubnationalGroup => 4,
            RegionType::Subnational => 5,
        }
    }
}

impl From<&RegionType> for u8 {
    fn from(val: &RegionType) -> Self {
        match val {
            RegionType::World => 0,
            RegionType::Continent => 1,
            RegionType::CountryGroup => 2,
            RegionType::Country => 3,
            RegionType::SubnationalGroup => 4,
            RegionType::Subnational => 5,
        }
    }
}

impl serde::Serialize for RegionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.into())
    }
}

impl<'de> serde::Deserialize<'de> for RegionType {
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
                        &"u8 < 6",
                    )
                })
                .and_then(|x| {
                    RegionType::try_from(x as u8).map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(x),
                            &"u8 < 6",
                        )
                    })
                })
        })
    }
}

#[either_field::make_template(
    GenStructs: true,
    DeleteTemplate: true,
    OmitEmptyTupleFields: true;
    pub Regions: [ player_count: _ ],
    pub RegionsWithPlayerCount: [ player_count: i32 ],
)]
#[derive(Debug, serde::Serialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionsTemplate {
    pub id: i32,
    pub code: String,
    pub region_type: RegionType,
    pub parent_id: Option<i32>,
    pub is_ranked: bool,
    pub player_count: either_field::either!(() | i32),
}

impl super::BasicTableQueries for Regions {
    const TABLE_NAME: &'static str = "regions";
}

impl Regions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO regions (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5);").bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query(const_format::formatcp!("INSERT INTO {table_name} (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET code = $2, region_type = $3, parent_id = $4, is_ranked = $5 WHERE {table_name}.id = $1;", table_name = Regions::TABLE_NAME)).bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }

    pub async fn get_ancestors(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar(const_format::formatcp!("WITH RECURSIVE ancestors AS (SELECT id, parent_id FROM {table_name} WHERE id = $1 UNION SELECT e.id, e.parent_id FROM {table_name} e INNER JOIN ancestors s ON s.parent_id = e.id) SELECT id FROM ancestors;", table_name = Regions::TABLE_NAME)).bind(id).fetch_all(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }

    pub async fn get_descendants(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar(const_format::formatcp!(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT z.id, z.parent_id FROM {regions_table} z
                WHERE id = $1
                UNION
                    SELECT e.id, e.parent_id
                    FROM {regions_table} e 
                INNER JOIN descendants s
                    ON s.id = e.parent_id
            ) SELECT id FROM descendants;
            "#,
            regions_table = Regions::TABLE_NAME
        ))
        .bind(id)
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }
}
