use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::close_connection,
    },
    auth::{
        Users, is_user_admin,
        validated_strings::{self, ValidatedString},
    },
};
use actix_web::{HttpResponse, dev::HttpServiceFactory, web};

pub fn users() -> impl HttpServiceFactory {
    web::scope("/users")
        .route("/list", web::post().to(list))
        .route("/insert", web::put().to(insert_or_edit))
        .route("/edit", web::patch().to(insert_or_edit))
        .route(
            "/delete",
            web::delete().to(crate::api::v1::delete_by_id::<Users>),
        )
        .default_service(web::get().to(default))
}
default_paths_fn!("/list", "/insert", "/edit", "/delete");

async fn list(body: web::Json<super::UserDataBody>) -> Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    if !is_user_admin(
        crate::auth::get_user_data(&body.session_token, &mut connection)
            .await?
            .user_id,
        &mut connection,
    )
    .await?
    {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    crate::api::v1::get_star_query::<Users>().await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InsertOrEditBody {
    id: Option<i32>,
    username: String,
    password: String,
    email: String,
    is_staff: bool,
    is_active: bool,
    is_verified: bool,
    player_id: Option<i32>,
    session_token: String,
}

async fn insert_or_edit(
    body: web::Json<InsertOrEditBody>,
) -> Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    if !is_user_admin(
        crate::auth::get_user_data(&body.session_token, &mut connection)
            .await?
            .user_id,
        &mut connection,
    )
    .await?
    {
        return Err(EveryReturnedError::InsufficientPermissions.into_final_error(""));
    }

    let username = validated_strings::username::Username::new_from_string(body.username)?;
    let email = validated_strings::email::Email::new_from_string(body.email)?;
    let password = match body.password.is_empty() {
        false => Some(validated_strings::password::Password::new_from_string(
            body.password,
        )?),
        true => None,
    };

    Users::insert_or_edit(
        body.id,
        username,
        password,
        email,
        false,
        body.is_staff,
        body.is_active,
        body.is_verified,
        body.player_id,
        &mut connection,
    )
    .await?;

    close_connection(connection).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"success":true}"#))
}
