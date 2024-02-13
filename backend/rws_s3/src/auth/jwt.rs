use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
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
pub async fn auth_check(
    cookie_jar: CookieJar,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    dotenv().ok();
    if let Some(token) = cookie_jar.get("token").map(|cookie| cookie.value()) {
        match decode::<JwtContent>(
            &token,
            &DecodingKey::from_secret(
                std::env::var("SECRET_JWT")
                    .map_err(internal_error)?
                    .as_bytes(),
            ),
            &Validation::new(jsonwebtoken::Algorithm::ES256),
        ) {
            Ok(data) => {
                req.extensions_mut().insert(data.claims.user_id);
            }
            Err(err) => {
                if req.uri() != "/signin" || req.uri() != "/signup" {
                    return Ok(next.run(req).await);
                }
                match err.kind() {
                    ErrorKind::InvalidToken => {
                        return Err((StatusCode::UNAUTHORIZED, "Invalid Token".to_owned()))
                    }
                    _ => return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_owned())),
                }
            }
        };
    }
    return Ok(next.run(req).await);
}
pub async fn encrypt_password(password: &[u8]) -> Result<String, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password, &salt)
        .map_err(internal_error)?
        .to_string();
    Ok(format!("{}{}", salt.to_string(), password_hash))
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
        .prepare("SELECT user_id,name FROM users WHERE (email = $1 OR name = $2) AND password = $3")
        .await
        .map_err(internal_error)?;
    let query = conn
        .query_one(&statement, &[&email, &username, &password])
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".to_owned()))?;
    let secret_key = std::env::var("SECRET_JWT").unwrap();
    let user_id: Uuid = query.get(0);
    let username: String = query.get(0);
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
        &EncodingKey::from_secret(secret_key.as_bytes()),
    ) {
        Ok(token) => return Ok(token),
        Err(err) => return Err((StatusCode::UNAUTHORIZED, err.to_string())),
    }
}