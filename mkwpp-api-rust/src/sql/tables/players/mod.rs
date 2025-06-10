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
    pub region_id: i32,
    pub joined_date: chrono::NaiveDate,
    pub last_activity: chrono::NaiveDate,
}

impl BasicTableQueries for Players {
    const TABLE_NAME: &'static str = "players";
}

impl Players {
    // pub async fn insert_query(
    //     &self,
    //     executor: &mut sqlx::PgConnection,
    // ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    //     sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity) VALUES($1, $2, $3, $4, $5, $6, $7);").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(&self.region_id).bind(self.joined_date).bind(self.last_activity).execute(executor).await
    // }

    pub async fn insert_or_replace_query(
        &self,
        executor: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        return sqlx::query("INSERT INTO players (id, name, alias, bio, region_id, joined_date, last_activity) VALUES($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET name = $2, alias = $3, bio = $4, region_id = $5, joined_date = $6, last_activity = $7 WHERE players.id = $1;").bind(self.id).bind(&self.name).bind(&self.alias).bind(&self.bio).bind(self.region_id).bind(self.joined_date).bind(self.last_activity).execute(executor).await;
    }

    pub async fn get_ids_but_list(
        executor: &mut sqlx::PgConnection,
        player_ids: &[i32],
    ) -> Result<Vec<i32>, sqlx::Error> {
        return sqlx::query_scalar(const_format::formatc!(
            "SELECT * FROM {} WHERE id != ANY($1);",
            Players::TABLE_NAME
        ))
        .bind(player_ids)
        .fetch_all(executor)
        .await;
    }
}

pub trait FilterPlayers: BasicTableQueries {
    const GET_SELECT_PLAYERS_QUERY_STR: &'static str;
    const GET_SELECT_PLAYERS_BY_REGION_QUERY_STR: &'static str;

    async fn get_select_players(
        executor: &mut sqlx::PgConnection,
        player_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(Self::GET_SELECT_PLAYERS_QUERY_STR)
            .bind(player_ids)
            .fetch_all(executor)
            .await;
    }
    async fn get_players_by_region_ids(
        executor: &mut sqlx::PgConnection,
        region_ids: Vec<i32>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        return sqlx::query(Self::GET_SELECT_PLAYERS_BY_REGION_QUERY_STR)
            .bind(region_ids)
            .fetch_all(executor)
            .await;
    }

    async fn get_players_by_region_id(
        executor: &mut sqlx::PgConnection,
        region_id: i32,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
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
