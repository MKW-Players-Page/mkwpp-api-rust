use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

#[derive(serde::Deserialize, Debug)]
pub struct BlogPosts {
    author: String,
    title: String,
    content: String,
    is_published: bool,
    published_at: String,
}

impl super::OldFixtureJson for BlogPosts {
    const FILENAME: &str = "blogposts.json";

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        let user_id: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(&self.author)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

        let user_id = match user_id {
            Some(v) => v,
            None => {
                println!(
                    "Skipped importing blog post: User by name of {} doesn't exist, please create it.",
                    self.author
                );
                return Ok(sqlx::postgres::PgQueryResult::default());
            }
        };

        return crate::sql::tables::blog_posts::BlogPosts {
            id: key,
            title: self.title,
            content: self.content,
            is_published: self.is_published,
            published_at: chrono::DateTime::from_naive_utc_and_offset(
                chrono::NaiveDateTime::parse_from_str(&self.published_at, "%FT%T%.3fZ").unwrap(),
                chrono::Utc,
            ),
            author_id: Some(user_id),
            username: None,
        }
        .upsert(transaction)
        .await;
    }
}

impl crate::sql::tables::blog_posts::BlogPosts {
    async fn upsert(
        self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO blog_posts (id, title, content, is_published, published_at, author_id) VALUES($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (title, content, is_published, published_at, author_id) = ($2, $3, $4, $5, $6) WHERE blog_posts.id = $1;").bind(self.id).bind(self.title).bind(self.content).bind(self.is_published).bind(self.published_at).bind(self.author_id).execute(executor).await.map_err(| e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}
