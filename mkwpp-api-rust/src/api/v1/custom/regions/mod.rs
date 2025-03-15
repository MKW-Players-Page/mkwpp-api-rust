use crate::sql::tables::{
    BasicTableQueries,
    regions::{RegionType, Regions, with_player_count::RegionsWithPlayerCount},
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};
use std::collections::HashMap;

mod child_tree;

macro_rules! region_fn {
    ($fn_name:ident, $handle:expr) => {
        async fn $fn_name(path: web::Path<i32>, data: web::Data<crate::AppState>) -> HttpResponse {
            return basic_get_i32(path, data, $handle).await;
        }
    };
}

pub fn regions() -> impl HttpServiceFactory {
    web::scope("/regions")
        .service(web::scope("/ancestors/{region_id}").default_service(web::get().to(get_ancestors)))
        .service(
            web::scope("/descendants/{region_id}").default_service(web::get().to(get_descendants)),
        )
        .route("/type_hashmap", web::get().to(get_region_type_hashmap))
        .route(
            "/descendence_tree",
            web::get().to(child_tree::get_region_child_tree),
        )
        .route(
            "/with_player_count",
            web::get().to(crate::api::v1::get_star_query::<RegionsWithPlayerCount>),
        )
        .default_service(web::get().to(default))
}

default_paths_fn!(
    "/ancestors/:regionId",
    "/descendants/:regionId",
    "/type_hashmap",
    "/with_player_count",
    "/descendence_tree"
);

region_fn!(get_ancestors, Regions::get_ancestors);
region_fn!(get_descendants, Regions::get_descendants);

async fn get_region_type_hashmap(data: web::Data<crate::AppState>) -> HttpResponse {
    crate::api::v1::basic_get_with_data_mod::<Regions, HashMap<RegionType, Vec<i32>>>(
        data,
        Regions::select_star_query,
        async |data: Vec<Regions>| {
            let mut hashmap: HashMap<RegionType, Vec<i32>> = HashMap::new();
            hashmap.insert(RegionType::World, vec![]);
            hashmap.insert(RegionType::Continent, vec![]);
            hashmap.insert(RegionType::Country, vec![]);
            hashmap.insert(RegionType::CountryGroup, vec![]);
            hashmap.insert(RegionType::Subnational, vec![]);
            hashmap.insert(RegionType::SubnationalGroup, vec![]);

            for region in data {
                hashmap
                    .get_mut(&region.region_type)
                    .expect("A RegionType is missing from get_region_type_hashmap")
                    .push(region.id);
            }

            hashmap
        },
    )
    .await
}

pub async fn basic_get_i32(
    path: web::Path<i32>,
    data: web::Data<crate::AppState>,
    rows_function: impl AsyncFnOnce(&mut sqlx::PgConnection, i32) -> Result<Vec<i32>, sqlx::Error>,
) -> HttpResponse {
    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return e,
    };

    let rows_request = rows_function(&mut connection, path.into_inner()).await;

    if let Err(e) = crate::api::v1::close_connection(connection).await {
        return e;
    }

    let rows = match rows_request {
        Ok(rows) => rows,
        Err(e) => {
            return crate::api::generate_error_response(
                "Couldn't get rows from database",
                &e.to_string(),
                HttpResponse::InternalServerError,
            );
        }
    };

    crate::api::v1::send_serialized_data(rows)
}
