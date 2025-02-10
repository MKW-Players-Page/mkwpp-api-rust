#[derive(serde::Deserialize, Debug)]
pub struct Regions {
    code: String,
    parent: Option<i32>,
    is_ranked: bool,
    #[serde(rename = "type")]
    region_type: String,
}

#[async_trait::async_trait]
impl super::OldFixtureJson for Regions {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
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

    fn get_sort(
    ) -> impl FnMut(&super::OldFixtureWrapper<Self>, &super::OldFixtureWrapper<Self>) -> std::cmp::Ordering
    {
        return |a, b| {
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
                a_parent.cmp(&b_parent)
            } else {
                a.pk.cmp(&b.pk)
            }
        };
    }
}
