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

