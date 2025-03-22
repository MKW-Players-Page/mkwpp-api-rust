use super::BasicTableQueries;

pub mod with_player_count;

#[derive(sqlx::Type, Debug, serde::Deserialize, Hash, PartialEq, Eq)]
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

#[either_field::make_template(
    GenStructs: true,
    DeleteTemplate: true,
    OmitEmptyTupleFields: true;
    pub Regions: [ player_count: _ ],
    pub RegionsWithPlayerCount: [ player_count: i64 ],
)]
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RegionsTemplate {
    pub id: i32,
    pub code: String,
    pub region_type: RegionType,
    pub parent_id: Option<i32>,
    pub is_ranked: bool,
    pub player_count: either_field::either!(() | i64),
}

impl super::BasicTableQueries for Regions {
    const TABLE_NAME: &'static str = "regions";
}

impl Regions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO regions (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5);").bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query(const_format::formatcp!("INSERT INTO {table_name} (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET code = $2, region_type = $3, parent_id = $4, is_ranked = $5 WHERE {table_name}.id = $1;", table_name = Regions::TABLE_NAME)).bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await;
    }

    pub async fn get_ancestors(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
        return sqlx::query_scalar(const_format::formatcp!("WITH RECURSIVE ancestors AS (SELECT id, parent_id FROM {table_name} WHERE id = $1 UNION SELECT e.id, e.parent_id FROM {table_name} e INNER JOIN ancestors s ON s.parent_id = e.id) SELECT id FROM ancestors;", table_name = Regions::TABLE_NAME)).bind(id).fetch_all(executor).await;
    }

    pub async fn get_descendants(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
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
        .await;
    }
}
