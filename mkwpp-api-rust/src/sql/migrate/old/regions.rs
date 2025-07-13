use crate::api::errors::{EveryReturnedError, FinalErrorResponse};
use crate::sql::tables::BasicTableQueries;

#[derive(serde::Deserialize, Debug)]
pub struct Regions {
    code: String,
    parent: Option<i32>,
    is_ranked: bool,
    #[serde(rename = "type")]
    region_type: String,
}

impl super::OldFixtureJson for Regions {
    const FILENAME: &str = "regions.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::regions::Regions {
            id: key,
            code: self.code,
            parent_id: self.parent,
            is_ranked: self.is_ranked,
            region_type: crate::sql::tables::regions::RegionType::try_from(
                self.region_type.as_str(),
            )
            .unwrap(),
        }
        .insert_or_replace_query(transaction)
        .await;
    }

    fn get_sort()
    -> impl FnMut(&super::OldFixtureWrapper<Self>, &super::OldFixtureWrapper<Self>) -> std::cmp::Ordering
    {
        |a, b| {
            let a_parent = a.fields.parent.unwrap_or(0);
            let b_parent = b.fields.parent.unwrap_or(0);
            let a_type_num: u8 =
                crate::sql::tables::regions::RegionType::try_from(a.fields.region_type.as_str())
                    .unwrap()
                    .into();
            let b_type_num: u8 =
                crate::sql::tables::regions::RegionType::try_from(b.fields.region_type.as_str())
                    .unwrap()
                    .into();
            if a_type_num != b_type_num {
                a_type_num.cmp(&b_type_num)
            } else if a_parent != b_parent {
                return a_parent.cmp(&b_parent);
            } else {
                return a.pk.cmp(&b.pk);
            }
        }
    }
}

impl crate::sql::tables::regions::Regions {
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query(const_format::formatcp!("INSERT INTO {table_name} (id, code, region_type, parent_id, is_ranked) VALUES($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET code = $2, region_type = $3, parent_id = $4, is_ranked = $5 WHERE {table_name}.id = $1;", table_name = crate::sql::tables::regions::Regions::TABLE_NAME)).bind(self.id).bind(&self.code).bind(&self.region_type).bind(self.parent_id).bind(self.is_ranked).execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
