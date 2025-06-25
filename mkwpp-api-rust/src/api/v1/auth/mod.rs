use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::errors::EveryReturnedError,
    app_state::access_app_state,
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
        .route("/password_forgot", web::post().to(password_forgot))
        .route("/password_reset", web::post().to(password_reset))
        .route(
            "/password_reset_check_token",
            web::post().to(password_reset_check_token),
        )
        .route("/update_password", web::post().to(update_password))
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

async fn register(body: web::Json<RegisterBody>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let transaction = {
        let data = data.read().await;
        data.pg_pool.begin().await
    };

    let username =
        match crate::auth::validated_strings::username::Username::new_from_string(body.username) {
            Ok(v) => v,
            Err(e) => return e.http_response(""),
        };

    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => return e.http_response(""),
        };

    let email = match crate::auth::validated_strings::email::Email::new_from_string(body.email) {
        Ok(v) => v,
        Err(e) => return e.http_response(""),
    };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
    };

    return match crate::auth::register(username, password, email, &mut transaction).await {
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
        Ok(_) => {
            if let Err(e) = transaction.commit().await {
                return EveryReturnedError::CommitPGTransaction.http_response(e);
            }
            HttpResponse::Ok()
                .content_type("application/json")
                .body("{}")
        }
    };
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    username: String,
    password: String,
}

async fn login(req: HttpRequest, body: web::Json<LoginBody>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let transaction = {
        let data = data.read().await;
        data.pg_pool.begin().await
    };

    std::thread::sleep(std::time::Duration::from_secs(5));
    let body = body.into_inner();

    let ip = req.peer_addr().unwrap().ip();
    let username =
        match crate::auth::validated_strings::username::Username::new_from_string(body.username) {
            Ok(v) => v,
            Err(e) => return e.http_response(""),
        };
    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => return e.http_response(""),
        };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
    };

    let login_attempt = crate::auth::login(username, password, ip, &mut transaction).await;

    if let Err(e) = transaction.commit().await {
        return EveryReturnedError::CommitPGTransaction.http_response(e);
    }

    let login_attempt = match login_attempt {
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
        Ok(v) => v,
    };

    match serde_json::to_string(&login_attempt) {
        Err(e) => EveryReturnedError::SerializingDataToJSON.http_response(e),
        Ok(v) => HttpResponse::Ok().content_type("application/json").body(v),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataBody {
    pub session_token: String,
}

async fn user_data(body: web::Json<UserDataBody>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let mut connection = {
        let data = data.read().await;
        match data.acquire_pg_connection().await {
            Ok(conn) => conn,
            Err(e) => return EveryReturnedError::NoConnectionFromPGPool.http_response(e),
        }
    };

    let user_data = match crate::auth::get_user_data(&body.session_token, &mut connection).await {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
    };

    let user_data = match user_data {
        Some(v) => v,
        None => return EveryReturnedError::UserIDDoesntExist.http_response(""),
    };

    let user_data = match user_data {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::DecodingDatabaseRows.http_response(e),
    };

    let user_data = match serde_json::to_string(&user_data) {
        Ok(v) => v,
        Err(e) => {
            return EveryReturnedError::SerializingDataToJSON.http_response(e);
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(user_data)
}

async fn logout(body: web::Json<UserDataBody>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        match data.pg_pool.begin().await {
            Ok(v) => v,
            Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
        }
    };

    let body = body.into_inner();

    if let Err(e) = crate::auth::logout(&body.session_token, &mut transaction).await {
        return EveryReturnedError::GettingFromDatabase.http_response(e);
    }

    return match transaction.commit().await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("{}"),
        Err(e) => return EveryReturnedError::CommitPGTransaction.http_response(e),
    };
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePasswordBody {
    old_password: String,
    new_password: String,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData,
}

async fn update_password(body: web::Json<UpdatePasswordBody>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
        match data.pg_pool.begin().await {
            Ok(v) => v,
            Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
        }
    };

    let body = body.into_inner();
    if let Ok(false) | Err(_) = is_valid_token(
        &body.validation_data.session_token,
        body.validation_data.user_id,
        &mut transaction,
    )
    .await
    {
        return EveryReturnedError::InvalidSessionToken.http_response("");
    }

    let old_password = match crate::auth::validated_strings::password::Password::new_from_string(
        body.old_password,
    ) {
        Ok(v) => v,
        Err(e) => return e.http_response(""),
    };
    let new_password = match crate::auth::validated_strings::password::Password::new_from_string(
        body.new_password,
    ) {
        Ok(v) => v,
        Err(e) => return e.http_response(""),
    };

    if let Err(e) = crate::auth::update_password(
        body.validation_data.user_id,
        old_password,
        new_password,
        &mut transaction,
    )
    .await
    {
        return EveryReturnedError::GettingFromDatabase.http_response(e);
    };

    if let Err(e) = transaction.commit().await {
        return EveryReturnedError::CommitPGTransaction.http_response(e);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

#[derive(serde::Deserialize)]
struct ActivateBody {
    token: String,
}

async fn activate(body: web::Json<ActivateBody>) -> HttpResponse {
    let mut transaction = {
        let data = access_app_state().await;
        let data = data.read().await;
        match data.pg_pool.begin().await {
            Ok(v) => v,
            Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
        }
    };

    if let Err(e) = activate_account(&body.0.token, &mut transaction).await {
        return EveryReturnedError::GettingFromDatabase.http_response(e);
    }

    if let Err(e) = transaction.commit().await {
        return EveryReturnedError::CommitPGTransaction.http_response(e);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

#[derive(serde::Deserialize)]
struct PasswordForgotBody {
    email: String,
}

async fn password_forgot(body: web::Json<PasswordForgotBody>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let transaction = {
        let data = data.read().await;
        data.pg_pool.begin().await
    };

    let email = match crate::auth::validated_strings::email::Email::new_from_string(body.email) {
        Ok(v) => v,
        Err(e) => return e.http_response(""),
    };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
    };

    return match crate::auth::password_reset_token_gen(email, &mut transaction).await {
        Err(e) => return EveryReturnedError::GeneratingToken.http_response(e),
        Ok(_) => {
            if let Err(e) = transaction.commit().await {
                return EveryReturnedError::CommitPGTransaction.http_response(e);
            }
            HttpResponse::Ok()
                .content_type("application/json")
                .body("{}")
        }
    };
}

#[derive(serde::Deserialize)]
struct PasswordResetBody {
    password: String,
    token: String,
}
async fn password_reset(body: web::Json<PasswordResetBody>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let transaction = {
        let data = data.read().await;
        data.pg_pool.begin().await
    };

    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => return e.http_response(""),
        };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
    };

    return match crate::auth::reset_password(&body.token, password, &mut transaction).await {
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
        Ok(_) => {
            if let Err(e) = transaction.commit().await {
                return EveryReturnedError::CommitPGTransaction.http_response(e);
            }
            HttpResponse::Ok()
                .content_type("application/json")
                .body("{}")
        }
    };
}

#[derive(serde::Deserialize)]
struct TokenCheck {
    token: String,
}
async fn password_reset_check_token(body: web::Json<TokenCheck>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let transaction = {
        let data = data.read().await;
        data.pg_pool.begin().await
    };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(e) => return EveryReturnedError::CreatePGTransaction.http_response(e),
    };

    return match crate::auth::is_reset_password_token_valid(&body.token, &mut transaction).await {
        Err(e) => return EveryReturnedError::GettingFromDatabase.http_response(e),
        Ok(v) => {
            if let Err(e) = transaction.commit().await {
                return EveryReturnedError::CommitPGTransaction.http_response(e);
            }

            HttpResponse::Ok()
                .content_type("application/json")
                .body(format!("{{\"is_valid\": {v}}}"))
        }
    };
}
