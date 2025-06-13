use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::FinalErrorResponse, app_state::AppState, auth::{ is_valid_token, validated_strings::ValidatedString, BareMinimumValidationData},
};

mod player;

pub fn auth() -> impl HttpServiceFactory {
    web::scope("/auth")
        .route("/register", web::put().to(register))
        .route("/login", web::put().to(login))
        .route("/logout", web::put().to(logout))
        .route("/user_data", web::post().to(user_data))
        .route("/update_password", web::post().to(update_password))
        .service(player::player())
        .default_service(web::get().to(default))
}
default_paths_fn!("/register", "/logout", "/login", "/user_data");

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
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the username")],
                    std::collections::HashMap::from([(
                        String::from("username"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };
    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the password")],
                    std::collections::HashMap::from([(
                        String::from("password"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };

    let email = match crate::auth::validated_strings::email::Email::new_from_string(body.email) {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new(
                vec![String::from("Error validating the email")],
                std::collections::HashMap::from([(
                    String::from("email"),
                    vec![format!("{:?}", e)],
                )]),
            )
            .generate_response(HttpResponse::BadRequest);
        }
    };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(error) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error starting transaction"),
                error.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    return match crate::auth::register(username, password, email, &mut transaction).await {
        Err(e) => FinalErrorResponse::new_no_fields(vec![
            String::from("Error registering user"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError),
        Ok(_) => {
            if let Err(x) = transaction.commit().await {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Couldn't commit transaction"),
                    x.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
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
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the username")],
                    std::collections::HashMap::from([(
                        String::from("username"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };
    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the password")],
                    std::collections::HashMap::from([(
                        String::from("password"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };

    let mut transaction = match transaction {
        Ok(v) => v,
        Err(error) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error starting transaction"),
                error.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let login_attempt = crate::auth::login(username, password, ip, &mut transaction).await;

    if let Err(x) = transaction.commit().await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Couldn't commit transaction"),
            x.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }

    let login_attempt = match login_attempt {
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error logging into user"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
        Ok(v) => v,
    };

    match serde_json::to_string(&login_attempt) {
        Err(e) => FinalErrorResponse::new_no_fields(vec![
            String::from("Error serializing data"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError),
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
            Err(e) => return AppState::pg_conn_http_error(e),
        }
    };

    let user_data = match crate::auth::get_user_data(&body.session_token, &mut connection).await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error getting user data"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let user_data = match user_data {
        Some(v) => v,
        None => {
            return FinalErrorResponse::new_no_fields(vec![String::from("No user found")])
                .generate_response(HttpResponse::BadRequest);
        }
    };

    let user_data = match user_data {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error decoding user data rows"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
        }
    };

    let user_data = match serde_json::to_string(&user_data) {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error serializing user data"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
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
            Err(e) => {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Error creating transaction"),
                    e.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
            }
        }
    };

    let body = body.into_inner();

    if let Err(e) = crate::auth::logout(&body.session_token, &mut transaction).await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Error removing session"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }

    return match transaction.commit().await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("{}"),
        Err(e) => FinalErrorResponse::new_no_fields(vec![
            String::from("Error committing transaction"),
            e.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError),
    };
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePasswordBody {
    old_password: String,
    new_password: String,
    #[serde(flatten)]
    validation_data: BareMinimumValidationData
}

async fn update_password(body: web::Json<UpdatePasswordBody>) -> HttpResponse {
    let data = crate::app_state::access_app_state().await;
    let mut transaction = {
        let data = data.read().await;
       match data.pg_pool.begin().await {
            Ok(v) => v,
            Err(error) => {
                return FinalErrorResponse::new_no_fields(vec![
                    String::from("Error starting transaction"),
                    error.to_string(),
                ])
                .generate_response(HttpResponse::InternalServerError);
            }
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
        return FinalErrorResponse::new_no_fields(vec![String::from(
            "Error validating session token",
        )])
        .generate_response(HttpResponse::BadRequest);
    }
    
    let old_password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.old_password) {
            Ok(v) => v,
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the password")],
                    std::collections::HashMap::from([(
                        String::from("old_password"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };
    let new_password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.new_password) {
            Ok(v) => v,
            Err(e) => {
                return FinalErrorResponse::new(
                    vec![String::from("Error validating the password")],
                    std::collections::HashMap::from([(
                        String::from("new_password"),
                        vec![format!("{:?}", e)],
                    )]),
                )
                .generate_response(HttpResponse::BadRequest);
            }
        };

    if let Err(x) = crate::auth::update_password(body.validation_data.user_id, old_password, new_password,&mut transaction).await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Error updating password"),
            x.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    };

    if let Err(x) = transaction.commit().await {
        return FinalErrorResponse::new_no_fields(vec![
            String::from("Couldn't commit transaction"),
            x.to_string(),
        ])
        .generate_response(HttpResponse::InternalServerError);
    }
    
    return HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}