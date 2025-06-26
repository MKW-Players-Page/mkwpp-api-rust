use crate::{
    api::errors::{EveryReturnedError, FinalErrorResponse},
    sql::tables::{
        BasicTableQueries,
        regions::tree::{ChildrenTree, generate_region_tree_player_count},
    },
};

pub use super::RegionsWithPlayerCount;

impl BasicTableQueries for RegionsWithPlayerCount {
    const TABLE_NAME: &'static str = super::Regions::TABLE_NAME;

    // This returns the number of players with the region selected
    async fn select_star_query(
        executor: &mut sqlx::PgConnection,
    ) -> Result<Vec<sqlx::postgres::PgRow>, FinalErrorResponse> {
        return sqlx::query(const_format::formatc!(
            r#"
            SELECT
                {table_name}.*,
                COALESCE(player_count, 0)::INTEGER AS player_count
            FROM {table_name}
            LEFT JOIN (
                SELECT region_id, COUNT(region_id) AS player_count
                FROM {players_table}
                GROUP BY region_id
            ) AS z ON
                z.region_id = {table_name}.id
            ORDER BY player_count DESC;
            "#,
            table_name = RegionsWithPlayerCount::TABLE_NAME,
            players_table = crate::sql::tables::players::Players::TABLE_NAME
        ))
        .fetch_all(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.to_final_error(e));
    }
}

impl RegionsWithPlayerCount {
    // This function gets all the regions, and adds to each region, the number of players from descendant regions
    // TODO: rewrite more optimally
    pub async fn collapse_counts_of_regions(data: &[Self]) -> Vec<Self> {
        let region_tree = generate_region_tree_player_count(data).await;

        let counts = data
            .iter()
            .map(|x| collapse_counts(data, &region_tree, x.id, false))
            .collect::<Vec<i32>>();

        let mut data = data.to_owned();
        data.iter_mut()
            .zip(counts)
            .for_each(|(x, new_count)| x.player_count = new_count);

        data
    }
}

// TODO: rewrite more optimally
fn collapse_counts(
    data: &[RegionsWithPlayerCount],
    region_tree: &ChildrenTree,
    lookup_id: i32,
    found: bool,
) -> i32 {
    let found = region_tree.id == lookup_id || found;

    let out = match found {
        false => 0,
        true => data
            .iter()
            .find(|x| x.id == region_tree.id)
            .map_or(0, |x| x.player_count),
    };

    match &region_tree.children {
        None => out,
        Some(children) => {
            out + children
                .iter()
                .map(|x| collapse_counts(data, x, lookup_id, found))
                .sum::<i32>()
        }
    }
}
