#[derive(serde::Deserialize, Debug)]
pub struct Submissions {
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
    status: u8,
    submitted_by: i32,
    submitted_at: String,
    submitter_note: Option<String>,
    reviewed_by: Option<i32>,
    reviewed_at: Option<String>,
    reviewer_note: Option<String>,
    score: Option<i32>,
}

impl super::OldFixtureJson for Submissions {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return crate::sql::tables::submissions::Submissions {
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
            status: crate::sql::tables::submissions::SubmissionStatus::try_from(self.status)
                .unwrap(),
            submitter_id: self.submitted_by,
            submitter_note: self.submitter_note,
            submitted_at: chrono::DateTime::from_naive_utc_and_offset(
                chrono::NaiveDateTime::parse_from_str(&self.submitted_at, "%FT%T%.3fZ").unwrap(),
                chrono::Utc,
            ),
            reviewer_id: self.reviewed_by,
            reviewer_note: self.reviewer_note,
            reviewed_at: self.reviewed_at.map(|v| {
                chrono::DateTime::from_naive_utc_and_offset(
                    chrono::NaiveDateTime::parse_from_str(&v, "%FT%T%.3fZ").unwrap(),
                    chrono::Utc,
                )
            }),
            score_id: self.score,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
