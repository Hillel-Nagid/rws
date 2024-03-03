use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use uuid::Uuid;

use crate::internal_error;

#[derive(Debug, Serialize, Deserialize)]
struct JwtContent {
    exp: usize,
    username: String,
    user_id: Uuid,
    iat: usize,
}

pub async fn encrypt_password(password: &[u8]) -> Result<String, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password, &salt)
        .map_err(internal_error)?
        .to_string();
    Ok(format!("{}", password_hash))
}
pub fn compare_passwords(
    password: &[u8],
    comperable_password: String,
) -> Result<bool, (StatusCode, String)> {
    let parsed_hash = PasswordHash::new(comperable_password.as_str()).map_err(internal_error)?;
    return Ok(Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok());
}
pub async fn authorize(
    username: String,
    password: String,
    email: String,
    conn: &bb8::PooledConnection<
        '_,
        bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>,
    >,
) -> Result<String, (StatusCode, String)> {
    dotenv().ok();
    let statement = conn
        .prepare("SELECT user_id,name,password FROM users WHERE email = $1 OR name = $2")
        .await
        .map_err(internal_error)?;
    let user = conn
        .query_one(&statement, &[&email, &username])
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".to_owned()))?;
    let comperable_password = user.get(2);
    compare_passwords(password.as_bytes(), comperable_password)?;
    let secret_key = std::env::var("SECRET_JWT").unwrap();
    let user_id: Uuid = user.get(0);
    let username: String = user.get(1);
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::days(30)).timestamp() as usize;
    let content = JwtContent {
        exp,
        username,
        user_id,
        iat,
    };
    match encode(
        &Header::default(),
        &content,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ) {
        Ok(token) => return Ok(token),
        Err(err) => return Err((StatusCode::UNAUTHORIZED, err.to_string())),
    }
}
