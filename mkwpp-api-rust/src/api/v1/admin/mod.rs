use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::{api::{errors::FinalErrorResponse, v1::close_connection}, auth::is_user_admin};

pub fn admin() -> impl HttpServiceFactory {
    web::scope("/admin")
        .route("/is_admin", web::post().to(is_admin))
        .default_service(web::get().to(default))
}
default_paths_fn!(
    "/is_admin"
);

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataBody {
    pub session_token: String,
}

async fn is_admin(
    body: web::Json<UserDataBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    let is_admin = is_user_admin(
        crate::auth::get_user_data(&body.session_token, &mut connection)
            .await?
            .user_id,
        &mut connection,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!(r#"{{"isAdmin":{is_admin}}}"#)))
}
