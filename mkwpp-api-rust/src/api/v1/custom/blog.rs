use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::FinalErrorResponse,
        v1::{
            custom::params::{Params, ParamsDestructured},
            decode_row_to_table, decode_rows_to_table, send_serialized_data,
        },
    },
    sql::tables::{blog_posts::BlogPosts, players::Players},
};

pub fn blog() -> impl HttpServiceFactory {
    web::scope("/blog")
        .route("/list", web::get().to(get_list))
        .route("/id/{number}", web::get().to(get_by_id))
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/id/:id");

async fn get_list(req: HttpRequest) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let mut executor = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let mut data = decode_rows_to_table::<BlogPosts>(
        BlogPosts::get_limit(params.limit, &mut executor).await?,
    )?;

    for post in data.iter_mut() {
        if let Some(author_id) = post.author_id {
            post.author_id =
                match Players::get_player_id_from_user_id(&mut executor, author_id).await {
                    Ok(v) => Some(v),
                    Err(_) => None,
                };
        }
    }

    crate::api::v1::close_connection(executor).await?;

    send_serialized_data(data)
}
async fn get_by_id(path: web::Path<i32>) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut executor = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let mut data = decode_row_to_table::<BlogPosts>(
        BlogPosts::get_by_id(path.into_inner(), &mut executor).await?,
    )?;

    if let Some(author_id) = data.author_id {
        data.author_id = match Players::get_player_id_from_user_id(&mut executor, author_id).await {
            Ok(v) => Some(v),
            Err(_) => None,
        };
    }

    crate::api::v1::close_connection(executor).await?;
    send_serialized_data(data)
}
