#[derive(sqlx::Type, Debug, serde::Deserialize)]
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
        return match value {
            "world" | "World" | "WORLD" => Ok(Self::World),
            "continent" | "Continent" | "CONTINENT" => Ok(Self::Continent),
            "country_group" | "COUNTRYGROUP" | "COUNTRY_GROUP" | "CountryGroup"
            | "countryGroup" => Ok(Self::CountryGroup),
            "country" | "Country" | "COUNTRY" => Ok(Self::Country),
            "subnational_group" | "SUBNATIONALGROUP" | "SUBNATIONAL_GROUP" | "SubnationalGroup"
            | "subnationalGroup" => Ok(Self::SubnationalGroup),
            "subnational" | "Subnational" | "SUBNATIONAL" => Ok(Self::Subnational),
            _ => Err(()),
        };
    }
}

impl From<RegionType> for u8 {
    fn from(val: RegionType) -> Self {
        return match val {
            RegionType::World => 0,
            RegionType::Continent => 1,
            RegionType::CountryGroup => 2,
            RegionType::Country => 3,
            RegionType::SubnationalGroup => 4,
            RegionType::Subnational => 5,
        };
    }
}

impl From<&RegionType> for u8 {
    fn from(val: &RegionType) -> Self {
        return match val {
            RegionType::World => 0,
            RegionType::Continent => 1,
            RegionType::CountryGroup => 2,
            RegionType::Country => 3,
            RegionType::SubnationalGroup => 4,
            RegionType::Subnational => 5,
        };
    }
}

impl serde::Serialize for RegionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return serializer.serialize_u8(self.into());
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Regions {
    pub id: i32,
    pub code: String,
    pub region_type: RegionType,
    pub parent_id: Option<i32>,
    pub is_ranked: bool,
}

impl super::BasicTableQueries for Regions {
    fn table_name() -> &'static str {
        return "regions";
    }
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
        return sqlx::query("INSERT INTO regions (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET code = $2, region_type = $3, parent_id = $4, is_ranked = $5 WHERE regions.id = $1;").bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await;
    }

    pub async fn get_ancestors(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
        return sqlx::query_scalar("WITH RECURSIVE ancestors AS (SELECT id, parent_id FROM regions WHERE id = $1 UNION SELECT e.id, e.parent_id FROM regions e INNER JOIN ancestors s ON s.parent_id = e.id) SELECT id FROM ancestors;").bind(id).fetch_all(executor).await;
    }

    pub async fn get_descendants(
        executor: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
        return sqlx::query_scalar("WITH RECURSIVE descendants AS (SELECT id, parent_id FROM regions WHERE id = $1 UNION SELECT e.id, e.parent_id FROM regions e INNER JOIN descendants s ON s.id = e.parent_id) SELECT id FROM descendants;").bind(id).fetch_all(executor).await;
    }
}
