use crate::api::errors::{EveryReturnedError, FinalErrorResponse};
use crate::custom_serde::DateAsTimestampNumber;
use crate::sql::tables::BasicTableQueries;

pub mod players_basic;

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Players {
    pub id: i32,
    pub name: String,
    pub alias: Option<String>,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub region_id: i32,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub joined_date: chrono::NaiveDate,
    #[serde(serialize_with = "DateAsTimestampNumber::serialize_as_timestamp")]
    pub last_activity: chrono::NaiveDate,
    pub submitters: Vec<i32>,
}

impl BasicTableQueries for Players {
    const TABLE_NAME: &'static str = "players";
}

impl Players {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
    //     sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity) VALUES($1, $2, $3, $4, $5, $6, $7);").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(&self.region_id).bind(self.joined_date).bind(self.last_activity).execute(executor).await
    // }

    // Feature only required because it's only used to import data currently
    #[cfg(feature = "import_data")]
    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity, submitters) VALUES($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET name = $2, alias = $3, bio = $4, region_id = $5, joined_date = $6, last_activity = $7, submitters = $8 WHERE players.id = $1;").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(self.region_id).bind(self.joined_date).bind(self.last_activity).bind(&self.submitters).execute(executor).await.map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn insert_or_edit(
        executor: &mut sqlx::PgConnection,
        id: Option<i32>,
        name: String,
        alias: Option<String>,
        bio: Option<String>,
        pronouns: Option<String>,
        region_id: i32,
        joined_date: chrono::NaiveDate,
        last_activity: chrono::NaiveDate,
        submitters: Vec<i32>,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        match id {
            None => {
                sqlx::query(const_format::formatcp!("INSERT INTO {table_name} (name, alias, bio, pronouns, region_id, joined_date, last_activity, submitters) VALUES ($1, $2, $3, $4, $5, $6, $7, $8);", table_name = Players::TABLE_NAME))
            }
            Some(id) => {
                sqlx::query(const_format::formatcp!("UPDATE {table_name} SET (name, alias, bio, pronouns, region_id, joined_date, last_activity, submitters) = ($2, $3, $4, $5, $6, $7, $, $9) WHERE id = $1;", table_name = Players::TABLE_NAME)).bind(id)

            }
        }
        .bind(name)
        .bind(alias)
        .bind(bio)
        .bind(pronouns)
        .bind(region_id)
        .bind(joined_date)
        .bind(last_activity)
        .bind(submitters)
        .execute(executor).await.map_err(| e | EveryReturnedError::GettingFromDatabase.into_final_error(e))
    }

    pub async fn update_player_bio(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        bio: &str,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        let bio = match bio.is_empty() {
            false => Some(bio),
            true => None,
        };
        return sqlx::query(const_format::formatc!(
            "UPDATE {} SET bio = $1 WHERE id = $2;",
            Players::TABLE_NAME
        ))
        .bind(bio)
        .bind(player_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn update_player_alias(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        alias: &str,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        let alias = match alias.is_empty() {
            false => Some(alias),
            true => None,
        };
        return sqlx::query(const_format::formatc!(
            "UPDATE {} SET alias = $1 WHERE id = $2;",
            Players::TABLE_NAME
        ))
        .bind(alias)
        .bind(player_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn update_player_pronouns(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        pronouns: &str,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        let pronouns = match pronouns.is_empty() {
            false => Some(pronouns),
            true => None,
        };
        return sqlx::query(const_format::formatc!(
            "UPDATE {} SET pronouns = $1 WHERE id = $2;",
            Players::TABLE_NAME
        ))
        .bind(pronouns)
        .bind(player_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn update_player_submitters(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
        new_list: &[i32],
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return sqlx::query(const_format::formatc!(
            "UPDATE {} SET submitters = $1 WHERE id = $2;",
            Players::TABLE_NAME
        ))
        .bind(new_list)
        .bind(player_id)
        .execute(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn get_player_submitters(
        executor: &mut sqlx::PgConnection,
        player_id: i32,
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar(const_format::formatc!(
            "SELECT submitters FROM {} WHERE id = $1",
            Players::TABLE_NAME
        ))
        .bind(player_id)
        .fetch_one(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn get_submittees(
        executor: &mut sqlx::PgConnection,
        user_id: i32,
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar(const_format::formatc!(
            "SELECT id FROM {} WHERE $1 = ANY(submitters)",
            Players::TABLE_NAME
        ))
        .bind(user_id)
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn get_player_id_from_user_id(
        executor: &mut sqlx::PgConnection,
        user_id: i32,
    ) -> Result<i32, FinalErrorResponse> {
        return sqlx::query_scalar("SELECT player_id FROM users WHERE id = $1;")
            .bind(user_id)
            .fetch_optional(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?
            .ok_or(EveryReturnedError::UserIDDoesntExist.into_final_error(""));
    }

    pub async fn get_player_ids_from_user_ids(
        executor: &mut sqlx::PgConnection,
        user_id: &[i32],
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar("SELECT player_id FROM users WHERE id = ANY($1);")
            .bind(user_id)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    pub async fn get_ids_but_list(
        executor: &mut sqlx::PgConnection,
        player_ids: &[i32],
    ) -> Result<Vec<i32>, FinalErrorResponse> {
        return sqlx::query_scalar(const_format::formatc!(
            "SELECT id FROM {} WHERE id != ANY($1);",
            Players::TABLE_NAME
        ))
        .bind(if player_ids.is_empty() {
            &[0]
        } else {
            player_ids
        })
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
}

pub trait FilterPlayers: BasicTableQueries {
    const GET_SELECT_PLAYERS_QUERY_STR: &'static str;
    const GET_SELECT_PLAYERS_BY_REGION_QUERY_STR: &'static str;

    async fn get_select_players(
        executor: &mut sqlx::PgConnection,
        player_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(Self::GET_SELECT_PLAYERS_QUERY_STR)
            .bind(player_ids)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }
    async fn get_players_by_region_ids(
        executor: &mut sqlx::PgConnection,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(Self::GET_SELECT_PLAYERS_BY_REGION_QUERY_STR)
            .bind(region_ids)
            .fetch_all(executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
    }

    async fn _get_players_by_region_id(
        executor: &mut sqlx::PgConnection,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        let region_ids =
            crate::sql::tables::regions::Regions::get_descendants(executor, region_id).await?;

        Self::get_players_by_region_ids(executor, region_ids).await
    }
}

impl FilterPlayers for Players {
    const GET_SELECT_PLAYERS_QUERY_STR: &'static str =
        const_format::formatc!("SELECT * FROM {} WHERE id = ANY($1);", Players::TABLE_NAME);
    const GET_SELECT_PLAYERS_BY_REGION_QUERY_STR: &'static str = const_format::formatc!(
        "SELECT * FROM {} WHERE region_id = ANY($1);",
        Players::TABLE_NAME
    );
}
