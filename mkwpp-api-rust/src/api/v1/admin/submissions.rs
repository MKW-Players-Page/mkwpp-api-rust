use crate::{
    api::{
        errors::FinalErrorResponse,
        v1::{decode_rows_to_table, send_serialized_data},
    },
    app_state::access_app_state,
    sql::tables::{
        BasicTableQueries,
        players::Players,
        submissions::{Submissions, edit_submissions::EditSubmissions},
    },
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn submissions() -> impl HttpServiceFactory {
    web::scope("/submissions")
        .route("/list_submissions", web::post().to(get))
        .route("/list_edit_submissions", web::post().to(get_edit))
        .route(
            "/delete_submission",
            web::delete().to(crate::api::v1::delete_by_id::<Submissions>),
        )
        .route(
            "/delete_edit_submission",
            web::delete().to(crate::api::v1::delete_by_id::<EditSubmissions>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!(
    "/list_submission",
    "/edit_submission",
    "/delete_submission",
    "/list_edit_submission",
    "/edit_edit_submission",
    "/delete_edit_submission"
);

async fn get() -> Result<HttpResponse, FinalErrorResponse> {
    let mut executor = {
        let app_state = access_app_state().await;
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    let data = Submissions::select_star_query(&mut executor).await?;
    let mut data = decode_rows_to_table::<Submissions>(data)?;

    for row in data.iter_mut() {
        row.submitter_id =
            Players::get_player_id_from_user_id(&mut executor, row.submitter_id).await?;
    }

    send_serialized_data(data)
}

async fn get_edit() -> Result<HttpResponse, FinalErrorResponse> {
    let mut executor = {
        let app_state = access_app_state().await;
        let app_state = app_state.read().await;
        app_state.acquire_pg_connection().await?
    };

    let data = EditSubmissions::select_star_query(&mut executor).await?;
    let mut data = decode_rows_to_table::<EditSubmissions>(data)?;

    for row in data.iter_mut() {
        row.submitter_id =
            Players::get_player_id_from_user_id(&mut executor, row.submitter_id).await?;
    }

    send_serialized_data(data)
}
