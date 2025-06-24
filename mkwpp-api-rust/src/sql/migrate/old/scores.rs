#[derive(serde::Deserialize, Debug)]
pub struct Scores {
    value: i32,
    category: u8,
    is_lap: bool,
    player: i32,
    track: i32,
    date: Option<String>,
    video_link: Option<String>,
    ghost_link: Option<String>,
    comment: Option<String>,
    admin_note: Option<String>,
    initial_rank: Option<i32>,
}

impl super::OldFixtureJson for Scores {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return crate::sql::tables::scores::Scores {
            id: key,
            value: self.value,
            category: crate::sql::tables::Category::try_from(self.category).unwrap(),
            is_lap: self.is_lap,
            player_id: self.player,
            track_id: self.track,
            date: self
                .date
                .map(|time_str| chrono::NaiveDate::parse_from_str(&time_str, "%F").unwrap()),
            video_link: self.video_link,
            ghost_link: self.ghost_link,
            comment: self.comment,
            admin_note: self.admin_note,
            initial_rank: self.initial_rank,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
