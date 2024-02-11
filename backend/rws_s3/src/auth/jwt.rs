use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use axum_extra::extract::cookie::CookieJar;

use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;
const DAY: usize = 86400000;

#[derive(Debug, Serialize, Deserialize)]
struct JwtContent {
    exp: usize,
    username: String,
    user_id: Uuid,
}

pub async fn auth_check(
    cookie_jar: CookieJar,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(token) = cookie_jar.get("token").map(|cookie| cookie.value()) {
        match decode::<JwtContent>(
            &token,
            &DecodingKey::from_secret(b"rws_secret"),
            &Validation::new(jsonwebtoken::Algorithm::ES256),
        ) {
            Ok(data) => {
                req.extensions_mut().insert(data.claims.user_id);
            }
            Err(err) => return Err((StatusCode::BAD_REQUEST, err.to_string())),
        };
    }
    return Ok(next.run(req).await);
}
