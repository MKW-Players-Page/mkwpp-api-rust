use std::net::IpAddr;

use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgQueryResult};
use validated_strings::ValidatedString;

use crate::api::{
    errors::{EveryReturnedError, FinalErrorResponse},
    v1::decode_row_to_table,
};

mod cooldown;
pub mod validated_strings;

struct _Users {
    id: i32,
    username: validated_strings::username::Username,
    password: validated_strings::password::Password,
    email: validated_strings::email::Email,
    last_login: Option<chrono::DateTime<chrono::Local>>,
    is_superuser: bool,
    is_staff: bool,
    is_active: bool,
    is_verified: bool,
    date_joined: chrono::DateTime<chrono::Local>,
    player_id: i32,
}

#[derive(sqlx::FromRow)]
struct BareMinimumData {
    id: i32,
    password: String,
    salt: String,
    is_verified: bool,
}

#[derive(serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LogInData {
    pub session_token: String,
    pub expiry: chrono::DateTime<chrono::Utc>,
}

pub async fn login(
    username: validated_strings::username::Username,
    password: validated_strings::password::Password,
    ip: IpAddr,
    // should be transaction
    executor: &mut sqlx::PgConnection,
) -> Result<LogInData, FinalErrorResponse> {
    let data = sqlx::query_as::<_, BareMinimumData>(const_format::formatc!(
        r#"
        SELECT
            id, password, salt, is_verified
        FROM users
        WHERE username = $1
    "#
    ))
    .bind(username.get_inner())
    .fetch_one(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    if cooldown::LogInAttempts::is_on_cooldown(
        cooldown::LogInAttempts::get_from_sql(executor, ip, data.id).await?,
        ip,
        data.id,
    ) {
        cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
        return Err(EveryReturnedError::UserOnCooldown.into_final_error(""));
    };

    if !data.is_verified {
        cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
        return Err(EveryReturnedError::UserNotVerified.into_final_error(""));
    };

    let hash = password.hash(data.salt.as_bytes());

    match hash == data.password {
        false => {
            cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
            Err(EveryReturnedError::InvalidInput.into_final_error(""))
        }
        true => {
            let token_engine = base64::engine::GeneralPurpose::new(
                &base64::alphabet::URL_SAFE,
                base64::engine::GeneralPurposeConfig::new(),
            );

            let mut hash_bytes = [0u8; 96];
            let mut out_string = String::new();
            rand::rng().fill(&mut hash_bytes);
            token_engine.encode_string(hash_bytes, &mut out_string);

            let log_in_data = sqlx::query_as::<_, LogInData>(const_format::formatc!(
                r#"
                    INSERT INTO auth_tokens (user_id, session_token)
                    VALUES ($1, $2)
                    RETURNING session_token, expiry
                "#
            ))
            .bind(data.id)
            .bind(out_string)
            .fetch_one(&mut *executor)
            .await
            .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;
            Ok(log_in_data)
        }
    }
}

pub async fn register(
    username: validated_strings::username::Username,
    password: validated_strings::password::Password,
    email: validated_strings::email::Email,
    // This should be a transaction!
    executor: &mut sqlx::PgConnection,
) -> Result<(), FinalErrorResponse> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash_string = password.hash(salt.as_str().as_bytes());

    let email = email.get_inner();
    let username = username.get_inner();

    let token_engine = base64::engine::GeneralPurpose::new(
        &base64::alphabet::URL_SAFE,
        base64::engine::GeneralPurposeConfig::new(),
    );

    let mut hash_bytes = [0u8; 45];
    let mut out_string = String::new();
    rand::rng().fill(&mut hash_bytes);
    token_engine.encode_string(hash_bytes, &mut out_string);

    let user_id: i32 = sqlx::query_scalar(const_format::formatc!(
        r#"
            INSERT INTO users (username, password, email, salt, is_active)
            VALUES($1, $2, $3, $4, true)
            RETURNING id
        "#
    ))
    .bind(username.as_str())
    .bind(hash_string)
    .bind(email.as_str())
    .bind(salt.as_str())
    .fetch_one(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    sqlx::query(const_format::formatc!(
        r#"
            INSERT INTO tokens (token, user_id, token_type)
            VALUES($1, $2, 'activation'::token_type)
        "#
    ))
    .bind(&out_string)
    .bind(user_id)
    .execute(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    crate::mail::MailService::account_verification(&username, &email, &out_string).await?;

    Ok(())
}

pub async fn password_reset_token_gen(
    email: validated_strings::email::Email,
    // This should be a transaction!
    executor: &mut sqlx::PgConnection,
) -> Result<(), FinalErrorResponse> {
    let email = email.get_inner();

    let token_engine = base64::engine::GeneralPurpose::new(
        &base64::alphabet::URL_SAFE,
        base64::engine::GeneralPurposeConfig::new(),
    );

    let mut hash_bytes = [0u8; 45];
    let mut out_string = String::new();
    rand::rng().fill(&mut hash_bytes);
    token_engine.encode_string(hash_bytes, &mut out_string);

    let user = sqlx::query(
        r#"
            SELECT id, username FROM users WHERE email = $1
        "#,
    )
    .bind(email.as_str())
    .fetch_optional(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    let user = match user {
        Some(v) => v,
        None => return Ok(()),
    };

    let id = user.get::<i32, &str>("id");
    let username = user.get::<String, &str>("username");

    sqlx::query(const_format::formatc!(
        r#"
            INSERT INTO tokens (token, user_id, token_type)
            VALUES($1, $2, 'password_reset'::token_type)
        "#
    ))
    .bind(&out_string)
    .bind(id)
    .execute(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    crate::mail::MailService::password_reset(&username, &email, &out_string).await?;

    Ok(())
}

pub async fn reset_password(
    token: &str,
    password: validated_strings::password::Password,
    executor: &mut sqlx::PgConnection,
) -> Result<(), FinalErrorResponse> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash_string = password.hash(salt.as_str().as_bytes());

    sqlx::query(
        r#"
            UPDATE users AS u1
            SET password = $2, salt = $3
            FROM users AS u2
            LEFT JOIN tokens AS t
                ON t.user_id = u2.id
            WHERE t.token = $1 AND token_type = 'password_reset'::token_type
        "#,
    )
    .bind(token)
    .bind(hash_string)
    .bind(salt.to_string())
    .execute(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    sqlx::query(
        "DELETE FROM tokens WHERE token = $1 AND token_type = 'password_reset'::token_type",
    )
    .bind(token)
    .execute(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    Ok(())
}

pub async fn is_reset_password_token_valid(
    token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<bool, FinalErrorResponse> {
    match sqlx::query(
        r#"
            SELECT user_id FROM tokens WHERE token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?
    {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub async fn activate_account(
    activation_token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<(), FinalErrorResponse> {
    sqlx::query(
        r#"
            UPDATE users AS u1
            SET is_verified = true
            FROM users AS u2
            LEFT JOIN tokens AS t
                ON t.user_id = u2.id
            WHERE t.token = $1 AND token_type = 'activation'::token_type
        "#,
    )
    .bind(activation_token)
    .execute(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    sqlx::query("DELETE FROM tokens WHERE token = $1 AND token_type = 'activation'::token_type")
        .bind(activation_token)
        .execute(&mut *executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    Ok(())
}

pub async fn update_password(
    id: i32,
    old_password: validated_strings::password::Password,
    new_password: validated_strings::password::Password,
    // This should be a transaction!
    executor: &mut sqlx::PgConnection,
) -> Result<(), FinalErrorResponse> {
    let data = sqlx::query_as::<_, BareMinimumData>(const_format::formatc!(
        r#"
        SELECT
            id, password, salt, is_verified
        FROM users
        WHERE id = $1
    "#
    ))
    .bind(id)
    .fetch_one(&mut *executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    if old_password.hash(data.salt.as_bytes()) != data.password {
        return Err(EveryReturnedError::InvalidInput.into_final_error(""));
    };

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash_string = new_password.hash(salt.as_str().as_bytes());

    sqlx::query(const_format::formatc!(
        r#"
            UPDATE users SET password = $1, salt = $2 WHERE id = $3
        "#
    ))
    .bind(hash_string)
    .bind(salt.as_str())
    .bind(id)
    .execute(executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?;

    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BareMinimumValidationData {
    pub user_id: i32,
    pub session_token: String,
}

pub async fn is_valid_token(
    session_token: &str,
    user_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<bool, FinalErrorResponse> {
    return sqlx::query(const_format::formatc!(
        r#"
            SELECT user_id
            FROM auth_tokens
            WHERE
                session_token = $1 AND
                user_id = $2 AND
                expiry >= NOW()
        "#
    ))
    .bind(session_token)
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map(|x| x.is_some())
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
}

pub async fn is_user_admin(
    user_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<bool, FinalErrorResponse> {
    return sqlx::query_scalar(const_format::formatc!(
        r#"
            SELECT is_superuser
            FROM users
            WHERE id = $1
        "#
    ))
    .bind(user_id)
    .fetch_one(executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e));
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSideUserData {
    pub player_id: i32,
    pub user_id: i32,
    pub username: String,
}

pub async fn get_user_data(
    session_token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<ClientSideUserData, FinalErrorResponse> {
    decode_row_to_table(
        sqlx::query(const_format::formatc!(
            r#"
            SELECT
                users.player_id,
                users.id AS user_id,
                username
            FROM users
            LEFT JOIN auth_tokens
                ON auth_tokens.user_id = users.id
            WHERE
                session_token = $1 AND
                expiry >= NOW()
        "#
        ))
        .bind(session_token)
        .fetch_optional(executor)
        .await
        .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?
        .ok_or(EveryReturnedError::UserIDDoesntExist.into_final_error(""))?,
    )
}

pub async fn get_user_id_from_player_id(
    player_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<i32, FinalErrorResponse> {
    sqlx::query_scalar(const_format::formatc!(
        r#"
            SELECT
                users.id AS user_id
            FROM users
            WHERE
                player_id = $1
        "#
    ))
    .bind(player_id)
    .fetch_optional(executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))?
    .ok_or(EveryReturnedError::UserHasNoAssociatedPlayer.into_final_error(""))
}

pub async fn logout(
    session_token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<PgQueryResult, FinalErrorResponse> {
    sqlx::query(const_format::formatc!(
        r#"
        DELETE FROM auth_tokens
        WHERE session_token = $1
        "#
    ))
    .bind(session_token)
    .execute(executor)
    .await
    .map_err(|e| EveryReturnedError::GettingFromDatabase.into_final_error(e))
}
