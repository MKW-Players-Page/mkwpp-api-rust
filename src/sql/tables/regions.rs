#[derive(sqlx::Type, serde::Serialize)]
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
            "world" => Ok(Self::World),
            "continent" => Ok(Self::Continent),
            "country_group" => Ok(Self::CountryGroup),
            "country" => Ok(Self::Country),
            "subnational_group" => Ok(Self::SubnationalGroup),
            "subnational" => Ok(Self::Subnational),
            _ => Err(()),
        };
    }
}

impl From<RegionType> for u8 {
    fn from(value: RegionType) -> Self {
        return match value {
            RegionType::World => 0,
            RegionType::Continent => 1,
            RegionType::CountryGroup => 2,
            RegionType::Country => 3,
            RegionType::SubnationalGroup => 4,
            RegionType::Subnational => 5,
        };
    }
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Regions {
    pub id: i32,
    pub code: String,
    pub region_type: RegionType,
    pub parent_id: Option<i32>,
    pub is_ranked: bool,
}

impl Regions {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO regions (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5);").bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await
    // }

    pub async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("SELECT * FROM regions;")
            .execute(executor)
            .await;
    }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO regions (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET code = $2, region_type = $3, parent_id = $4, is_ranked = $5 WHERE regions.id = $1;").bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await;
    }
}
