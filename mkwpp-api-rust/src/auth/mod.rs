use std::net::IpAddr;

use anyhow::anyhow;
use base64::Engine;
use mail_send::{Credentials, SmtpClientBuilder, mail_builder::MessageBuilder};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, postgres::PgQueryResult};
use validated_strings::ValidatedString;

mod cooldown;
pub mod validated_strings;

struct Users {
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
) -> Result<LogInData, anyhow::Error> {
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
    .await?;

    if cooldown::LogInAttempts::is_on_cooldown(
        cooldown::LogInAttempts::get_from_sql(executor, ip, data.id).await?,
        ip,
        data.id,
    ) {
        cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
        return Err(anyhow::anyhow!("Player is on cooldown"));
    };

    if !data.is_verified {
        cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
        return Err(anyhow::anyhow!("Player is not verified"));
    };

    let hash = password.hash(data.salt.as_bytes());

    match hash == data.password {
        false => {
            cooldown::LogInAttempts::insert(executor, ip, data.id).await?;
            Err(anyhow::anyhow!("Data is wrong"))
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
            .await?;
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
) -> Result<(), anyhow::Error> {
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

    let email_msg = MessageBuilder::new()
        .from(("Mario Kart Wii Players' Page", "no-reply@mkwpp.com"))
        .to((username.as_str(), email.as_str()))
        .subject("Account Verification")
        .text_body(format!(
            r#"
            Hi {username},
            
            Your Mario Kart Wii Players' Page account has been successfully created.
            
            To activate your account, please visit the following link:
            
            https://mariokart64.com/mkw/activate?tkn={out_string}
            
            Happy karting!
            "#
        ));

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
    .await?;

    sqlx::query(const_format::formatc!(
        r#"
            INSERT INTO activation_tokens (token, user_id)
            VALUES($1, $2)
        "#
    ))
    .bind(out_string)
    .bind(user_id)
    .execute(&mut *executor)
    .await?;

    SmtpClientBuilder::new(
        crate::ENV_VARS.smtp_hostname.as_str(),
        crate::ENV_VARS.smtp_port,
    )
    .implicit_tls(false)
    .credentials(Credentials::new("username", "secret"))
    .allow_invalid_certs()
    .connect()
    .await?
    .send(email_msg)
    .await?;

    Ok(())
}

pub async fn activate_account(
    activation_token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
            UPDATE users AS u1
            SET is_verified = true
            FROM users AS u2
            LEFT JOIN activation_tokens AS a
                ON a.user_id = u2.id
            WHERE a.token = $1
        "#,
    )
    .bind(activation_token)
    .execute(&mut *executor)
    .await?;
    
    sqlx::query("DELETE FROM activation_tokens WHERE token = $1")
        .bind(activation_token)
        .execute(&mut *executor)
        .await?;
    
    Ok(())
}

pub async fn update_password(
    id: i32,
    old_password: validated_strings::password::Password,
    new_password: validated_strings::password::Password,
    // This should be a transaction!
    executor: &mut sqlx::PgConnection,
) -> Result<(), anyhow::Error> {
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
    .await?;

    if old_password.hash(data.salt.as_bytes()) != data.password {
        return Err(anyhow!("Old Password is wrong!"));
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
    .await?;

    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BareMinimumValidationData {
    pub user_id: i32,
    pub session_token: String,
}

pub enum TokenValidity {}

pub async fn is_valid_token(
    session_token: &str,
    user_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<bool, sqlx::Error> {
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
    .map(|x| x.is_some());
}

pub async fn is_user_admin(
    user_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<bool, sqlx::Error> {
    return sqlx::query_scalar(const_format::formatc!(
        r#"
            SELECT is_superuser
            FROM users
            WHERE id = $1
        "#
    ))
    .bind(user_id)
    .fetch_one(executor)
    .await;
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
) -> Result<Option<Result<ClientSideUserData, sqlx::Error>>, sqlx::Error> {
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
    .map(|x| x.map(|x| ClientSideUserData::from_row(&x)))
}

pub async fn get_user_id_from_player_id(
    player_id: i32,
    executor: &mut sqlx::PgConnection,
) -> Result<Option<i32>, sqlx::Error> {
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
}

pub async fn logout(
    session_token: &str,
    executor: &mut sqlx::PgConnection,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query(const_format::formatc!(
        r#"
        DELETE FROM auth_tokens
        WHERE session_token = $1
        "#
    ))
    .bind(session_token)
    .execute(executor)
    .await
}
