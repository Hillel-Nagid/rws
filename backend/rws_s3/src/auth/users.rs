use crate::{internal_error, ConnectionPool};
use axum::{
    extract::{Multipart, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use email_address::*;
use serde_json::{json, Value};
use time::Duration;
use uuid::Uuid;

use super::jwt::{authorize, encrypt_password};

pub async fn signup(
    State(pool): State<ConnectionPool>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn: bb8::PooledConnection<
        '_,
        bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>,
    > = pool.get().await.map_err(internal_error)?;
    let mut username = String::from("");
    let mut password = String::from("");
    let mut email = String::from("");
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(part) = &field.name() {
            match part {
                &"username" => username = field.text().await.map_err(internal_error)?,
                &"password" => password = field.text().await.map_err(internal_error)?,
                &"email" => email = field.text().await.map_err(internal_error)?,
                _ => {}
            }
        }
    }
    if username == "".to_owned() || password == "".to_owned() || email == "".to_owned() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username or password or email were not specified".to_owned(),
        ));
    }
    let user_id = Uuid::new_v4();
    let statement = conn
        .prepare("INSERT INTO users (user_id,name,password,email) VALUES ($1,$2,$3,$4)")
        .await
        .map_err(internal_error)?;
    let encrypted_password = encrypt_password(password.as_bytes()).await?;
    match conn
        .execute(
            &statement,
            &[&user_id, &username, &encrypted_password, &email],
        )
        .await
        .map_err(internal_error)
    {
        Ok(_) => return Ok(Json(json!({"result":"User created successfuly"}))),
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Username already taken".to_owned())),
    };
}

pub async fn signin(
    State(pool): State<ConnectionPool>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn: bb8::PooledConnection<
        '_,
        bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>,
    > = pool.get().await.map_err(internal_error)?;
    let mut username = String::from("");
    let mut password = String::from("");
    let mut email = String::from("");
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(part) = &field.name() {
            match part {
                &"identifier" => {
                    let identifier = field.text().await.map_err(internal_error)?;
                    let is_valid_email = EmailAddress::is_valid(&identifier);
                    match is_valid_email {
                        true => email = identifier,
                        false => username = identifier,
                    }
                }
                &"password" => password = field.text().await.map_err(internal_error)?,
                _ => {}
            }
        }
    }
    let encrypted_password = encrypt_password(password.as_bytes()).await?;
    let token = authorize(username, encrypted_password, email, &conn).await?;
    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(Duration::days(30))
        .same_site(SameSite::Lax)
        .http_only(true);
    let mut response = Response::new(json!({"status":"success","token":token}).to_string());
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie.to_string().parse().map_err(internal_error)?,
    );
    Ok(response)
}
