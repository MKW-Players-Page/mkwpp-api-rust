use crate::custom_serde::DateAsTimestampNumber;
use crate::sql::tables::BasicTableQueries;
use sqlx::postgres::PgRow;

#[derive(serde::Deserialize, Debug, serde::Serialize, sqlx::FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlogPosts {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub is_published: bool,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub author_id: i32,
}

impl BasicTableQueries for BlogPosts {
    const TABLE_NAME: &'static str = "blog_posts";
}

impl BlogPosts {
    pub async fn get_limit(
        limit: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<PgRow>, sqlx::Error> {
        return sqlx::query(
            "SELECT * FROM blog_posts WHERE is_published = true ORDER BY published_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(executor)
        .await;
    }

    pub async fn get_by_id(
        id: i32,
        executor: &mut sqlx::PgConnection,
    ) -> Result<PgRow, sqlx::Error> {
        return sqlx::query("SELECT * FROM blog_posts WHERE is_published = true AND id = $1")
            .bind(id)
            .fetch_one(executor)
            .await;
    }
}
