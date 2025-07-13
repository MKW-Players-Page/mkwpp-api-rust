#[derive(serde::Deserialize, Debug, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracks {
    pub id: i32,
    pub abbr: String,
    pub cup_id: i32,
    pub categories: Vec<super::Category>,
}

impl super::BasicTableQueries for Tracks {
    const TABLE_NAME: &'static str = "tracks";
}
