use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::{
    api::FinalErrorResponse, app_state::AppState, auth::validated_strings::ValidatedString,
};

pub fn auth() -> impl HttpServiceFactory {
    web::scope("/auth")
        .route("/register", web::put().to(register))
        .route("/login", web::put().to(login))
        .route("/logout", web::put().to(logout))
        .route("/user_data", web::post().to(user_data))
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
    let data = data.read().unwrap();

    let transaction_future = data.pg_pool.begin();

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

    let mut transaction = match transaction_future.await {
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
    let data = data.read().unwrap();

    let transaction_future = data.pg_pool.begin();
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

    let mut transaction = match transaction_future.await {
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
struct UserDataBody {
    session_token: String,
}

async fn user_data(body: web::Json<UserDataBody>) -> HttpResponse {
    let body = body.into_inner();

    let data = crate::app_state::access_app_state().await;
    let data = data.read().unwrap();

    let mut connection = match data.acquire_pg_connection().await {
        Ok(conn) => conn,
        Err(e) => return AppState::pg_conn_http_error(e),
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
    let data = data.read().unwrap();

    let mut transaction = match data.pg_pool.begin().await {
        Ok(v) => v,
        Err(e) => {
            return FinalErrorResponse::new_no_fields(vec![
                String::from("Error creating transaction"),
                e.to_string(),
            ])
            .generate_response(HttpResponse::InternalServerError);
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
