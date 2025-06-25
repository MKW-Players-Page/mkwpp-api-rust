use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        FinalErrorResponse,
        v1::{
            custom::params::{Params, ParamsDestructured},
            decode_row_to_table, decode_rows_to_table, send_serialized_data,
        },
    },
    app_state::AppState,
    sql::tables::{blog_posts::BlogPosts, players::Players},
};

pub fn blog() -> impl HttpServiceFactory {
    web::scope("/blog")
        .route("/list", web::get().to(get_list))
        .route("/id/{number}", web::get().to(get_by_id))
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/id/:id");

async fn get_list(req: HttpRequest) -> HttpResponse {
    let params = ParamsDestructured::from_query(
        web::Query::<Params>::from_query(req.query_string()).unwrap(),
    );

    let data = crate::app_state::access_app_state().await;
    let mut executor = {
        let data = data.read().await;
        match data.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    let data = match BlogPosts::get_limit(params.limit, &mut executor).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting data from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let mut data = match decode_rows_to_table::<BlogPosts>(data) {
        Ok(v) => v,
        Err(e) => return e,
    };

    for post in data.iter_mut() {
        post.author_id =
            match Players::get_player_ids_from_user_ids(&mut executor, &[post.author_id]).await {
                Ok(v) => match v.first() {
                    Some(v) => *v,
                    None => {
                        return FinalErrorResponse::new_no_fields(vec![String::from(
                            "Error converting user ids, no user by such ID",
                        )])
                        .generate_response(HttpResponse::InternalServerError);
                    }
                },
                Err(e) => {
                    return FinalErrorResponse::new_no_fields(vec![
                        String::from("Error converting user ids"),
                        e.to_string(),
                    ])
                    .generate_response(HttpResponse::InternalServerError);
                }
            };
    }

    if let Err(e) = crate::api::v1::close_connection(executor).await {
        return e;
    }
    send_serialized_data(data)
}
async fn get_by_id(path: web::Path<i32>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut executor = {
        let data = data.read().await;
        match data.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    let data = match BlogPosts::get_by_id(path.into_inner(), &mut executor).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting data from database"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let mut data = match decode_row_to_table::<BlogPosts>(data) {
        Ok(v) => v,
        Err(e) => return e,
    };

    data.author_id =
        match Players::get_player_ids_from_user_ids(&mut executor, &[data.author_id]).await {
            Ok(v) => match v.first() {
                Some(v) => *v,
                None => {
                    return FinalErrorResponse::new_no_fields(vec![String::from(
                        "Error converting user ids, no user by such ID",
                    )])
                    .generate_response(HttpResponse::InternalServerError);
                }
            },
            Err(e) => {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Error converting user ids"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
            }
        };

    if let Err(e) = crate::api::v1::close_connection(executor).await {
        return e;
    }

    send_serialized_data(data)
}
