use actix_web::{HttpRequest, HttpResponse, dev::HttpServiceFactory, web};

use crate::auth::validated_strings::ValidatedString;

pub fn auth() -> impl HttpServiceFactory {
    return web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .default_service(web::get().to(default));
}
default_paths_fn!("/register", "/login");

#[derive(serde::Deserialize)]
struct RegisterBody {
    username: String,
    password: String,
    email: String,
}

async fn register(body: web::Bytes, data: web::Data<crate::AppState>) -> HttpResponse {
    let transaction_future = data.pg_pool.begin();

    let body = match serde_json::from_slice::<RegisterBody>(&body) {
        Ok(v) => v,
        Err(error) => {
            return crate::api::generate_error_response(
                "Couldn't turn the request body into valid JSON data",
                error.to_string().as_str(),
                HttpResponse::BadRequest,
            );
        }
    };

    let username =
        match crate::auth::validated_strings::username::Username::new_from_string(body.username) {
            Ok(v) => v,
            Err(e) => {
                return crate::api::generate_error_response(
                    "Error validating the username",
                    &format!("{:?}", e),
                    HttpResponse::BadRequest,
                );
            }
        };
    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => {
                return crate::api::generate_error_response(
                    "Error validating the password",
                    &format!("{:?}", e),
                    HttpResponse::BadRequest,
                );
            }
        };

    let email = match crate::auth::validated_strings::email::Email::new_from_string(body.email) {
        Ok(v) => v,
        Err(e) => {
            return crate::api::generate_error_response(
                "Error validating the email",
                &format!("{:?}", e),
                HttpResponse::BadRequest,
            );
        }
    };

    let mut transaction = match transaction_future.await {
        Ok(v) => v,
        Err(error) => {
            return crate::api::generate_error_response(
                "Error starting transaction",
                error.to_string().as_str(),
                HttpResponse::InternalServerError,
            );
        }
    };

    return match crate::auth::register(username, password, email, &mut transaction).await {
        Err(e) => crate::api::generate_error_response(
            "Error registering user",
            e.to_string().as_str(),
            HttpResponse::InternalServerError,
        ),
        Ok(v) => match serde_json::to_string(&v) {
            Err(e) => crate::api::generate_error_response(
                "Error serializing data",
                e.to_string().as_str(),
                HttpResponse::InternalServerError,
            ),
            Ok(v) => {
                if let Err(x) = transaction.commit().await {
                    return crate::api::generate_error_response(
                        "Couldn't commit transaction",
                        x.to_string().as_str(),
                        HttpResponse::InternalServerError,
                    );
                }
                HttpResponse::Ok().content_type("application/json").body(v)
            }
        },
    };
}

#[derive(serde::Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

async fn login(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<crate::AppState>,
) -> HttpResponse {
    let transaction_future = data.pg_pool.begin();

    // default pause
    std::thread::sleep(std::time::Duration::from_secs(5));

    let body = match serde_json::from_slice::<LoginBody>(&body) {
        Ok(v) => v,
        Err(error) => {
            return crate::api::generate_error_response(
                "Couldn't turn the request body into valid JSON data",
                error.to_string().as_str(),
                HttpResponse::BadRequest,
            );
        }
    };

    let ip = req.peer_addr().unwrap().ip();
    let username =
        match crate::auth::validated_strings::username::Username::new_from_string(body.username) {
            Ok(v) => v,
            Err(e) => {
                return crate::api::generate_error_response(
                    "Error validating the username",
                    &format!("{:?}", e),
                    HttpResponse::BadRequest,
                );
            }
        };
    let password =
        match crate::auth::validated_strings::password::Password::new_from_string(body.password) {
            Ok(v) => v,
            Err(e) => {
                return crate::api::generate_error_response(
                    "Error validating the password",
                    &format!("{:?}", e),
                    HttpResponse::BadRequest,
                );
            }
        };

    let mut transaction = match transaction_future.await {
        Ok(v) => v,
        Err(error) => {
            return crate::api::generate_error_response(
                "Error starting transaction",
                error.to_string().as_str(),
                HttpResponse::InternalServerError,
            );
        }
    };
    return match crate::auth::log_in(username, password, ip, &mut transaction).await {
        Err(e) => crate::api::generate_error_response(
            "Error logging into user",
            e.to_string().as_str(),
            HttpResponse::InternalServerError,
        ),
        Ok(v) => match serde_json::to_string(&v) {
            Err(e) => crate::api::generate_error_response(
                "Error serializing data",
                e.to_string().as_str(),
                HttpResponse::InternalServerError,
            ),
            Ok(v) => {
                if let Err(x) = transaction.commit().await {
                    return crate::api::generate_error_response(
                        "Couldn't commit transaction",
                        x.to_string().as_str(),
                        HttpResponse::InternalServerError,
                    );
                }
                HttpResponse::Ok().content_type("application/json").body(v)
            }
        },
    };
}
