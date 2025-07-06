use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::{
        errors::{EveryReturnedError, FinalErrorResponse},
        v1::{send_serialized_data},
    },
    auth::{
        BareMinimumValidationData, activate_account, is_valid_token,
        validated_strings::ValidatedString,
    },
};

mod player;
pub mod submissions;

pub fn auth() -> impl HttpServiceFactory {
    web::scope("/auth")
        .route("/register", web::put().to(register))
        .route("/login", web::put().to(login))
        .route("/logout", web::put().to(logout))
        .route("/activate", web::put().to(activate))
        .route("/user_data", web::post().to(user_data))
        .route("/password_forgot", web::put().to(password_forgot))
        .route("/password_reset", web::put().to(password_reset))
        .route(
            "/password_reset_check_token",
            web::post().to(password_reset_check_token),
        )
        .route("/update_password", web::put().to(update_password))
        .service(player::player())
        .service(submissions::submissions())
        .default_service(web::get().to(default))
}
default_paths_fn!(
    "/register",
    "/logout",
    "/login",
    "/user_data",
    "/update_password",
    "/player",
    "/submissions"
);

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBody {
    username: String,
    password: String,
    email: String,
}

async fn register(
    body: web::Json<RegisterBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    let username =
        crate::auth::validated_strings::username::Username::new_from_string(body.username)?;
    let password =
        crate::auth::validated_strings::password::Password::new_from_string(body.password)?;
    let email = crate::auth::validated_strings::email::Email::new_from_string(body.email)?;

    crate::auth::register(username, password, email, &mut transaction).await?;
    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    username: String,
    password: String,
}

async fn login(
    req: HttpRequest,
    body: web::Json<LoginBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    std::thread::sleep(std::time::Duration::from_secs(5));
    let body = body.into_inner();

    let ip = req.peer_addr().unwrap().ip();

    let username =
        crate::auth::validated_strings::username::Username::new_from_string(body.username)?;
    let password =
        crate::auth::validated_strings::password::Password::new_from_string(body.password)?;

    let login_attempt = crate::auth::login(username, password, ip, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;

    send_serialized_data(login_attempt)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataBody {
    pub session_token: String,
}

async fn user_data(
    body: web::Json<UserDataBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        data.acquire_pg_connection().await?
    };

    send_serialized_data(crate::auth::get_user_data(&body.session_token, &mut connection).await?)
}

async fn logout(
    body: web::Json<UserDataBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    let body = body.into_inner();

    crate::auth::logout(&body.session_token, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePasswordBody {
    old_password: String,
    new_password: String,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

async fn update_password(
    body: web::Json<UpdatePasswordBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    let body = body.into_inner();
    if !is_valid_token(
        &body.validation_data.session_token,
        body.validation_data.user_id,
        &mut transaction,
    )
    .await?
    {
        return Err(EveryReturnedError::InvalidSessionToken.into_final_error(""));
    }

    let old_password =
        crate::auth::validated_strings::password::Password::new_from_string(body.old_password)?;
    let new_password =
        crate::auth::validated_strings::password::Password::new_from_string(body.new_password)?;

    crate::auth::update_password(
        body.validation_data.user_id,
        old_password,
        new_password,
        &mut transaction,
    )
    .await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
struct ActivateBody {
    token: String,
}

async fn activate(
    body: web::Json<ActivateBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    activate_account(&body.0.token, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
struct PasswordForgotBody {
    email: String,
}

async fn password_forgot(
    body: web::Json<PasswordForgotBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    let email = crate::auth::validated_strings::email::Email::new_from_string(body.email)?;

    crate::auth::password_reset_token_gen(email, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
struct PasswordResetBody {
    password: String,
    token: String,
}
async fn password_reset(
    body: web::Json<PasswordResetBody>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    let password =
        crate::auth::validated_strings::password::Password::new_from_string(body.password)?;

    crate::auth::reset_password(&body.token, password, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[derive(serde::Deserialize)]
struct TokenCheck {
    token: String,
}
async fn password_reset_check_token(
    body: web::Json<TokenCheck>,
) -> actix_web::Result<HttpResponse, FinalErrorResponse> {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        data.pg_pool
            .begin()
            .await
            .map_err(|e| EveryReturnedError::CreatePGTransaction.into_final_error(e))?
    };

    crate::auth::is_reset_password_token_valid(&body.token, &mut transaction).await?;

    transaction
        .commit()
        .await
        .map_err(|e| EveryReturnedError::CommitPGTransaction.into_final_error(e))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}