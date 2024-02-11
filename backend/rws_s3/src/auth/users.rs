use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{internal_error, ConnectionPool};

pub async fn signin(
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
    match conn
        .execute(&statement, &[&user_id, &username, &password, &email])
        .await
        .map_err(internal_error)
    {
        Ok(_) => return Ok(Json(json!({"result":"User created successfuly"}))),
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Username already taken".to_owned())),
    };
}

pub async fn signup(
    State(pool): State<ConnectionPool>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    Ok(Json(json!({"result":true})))
}
