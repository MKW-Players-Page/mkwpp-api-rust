#[cfg(feature = "import_data_old")]
pub mod old;

#[cfg(feature = "import_data_new")]
pub mod new;

const TABLE_NAMES: [&str; 12] = [
    "regions",
    "tracks",
    "players",
    "users",
    "blog_posts",
    "edit_submissions",
    "player_awards",
    "site_champs",
    "standard_levels",
    "standards",
    "scores",
    "submissions",
];

async fn reset_sequences(transaction: &mut sqlx::PgConnection) {
    sqlx::query(
        "SELECT setval('regions_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM regions));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for regions");

    sqlx::query(
        "SELECT setval('blog_posts_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM blog_posts));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for blog_posts");

    sqlx::query("SELECT setval('edit_submission_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM edit_submissions));").execute(&mut *transaction).await.expect("Should've reset the minimum id for edit_submissions");

    sqlx::query("SELECT setval('player_awards_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM player_awards));").execute(&mut *transaction).await.expect("Should've reset the minimum id for player_awards");

    sqlx::query(
        "SELECT setval('players_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM players));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for players");

    sqlx::query(
        "SELECT setval('players_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM players));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for players");

    sqlx::query("SELECT setval('scores_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM scores));")
        .execute(&mut *transaction)
        .await
        .expect("Should've reset the minimum id for scores");

    sqlx::query(
        "SELECT setval('site_champs_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM site_champs));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for site_champs");

    sqlx::query("SELECT setval('standard_levels_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM standard_levels));").execute(&mut *transaction).await.expect("Should've reset the minimum id for standard_levels");

    sqlx::query(
        "SELECT setval('standards_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM standards));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for standards");

    sqlx::query(
        "SELECT setval('submissions_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM submissions));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for submissions");

    sqlx::query("SELECT setval('tracks_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM tracks));")
        .execute(&mut *transaction)
        .await
        .expect("Should've reset the minimum id for tracks");
}
