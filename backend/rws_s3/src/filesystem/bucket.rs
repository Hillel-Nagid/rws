use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{create_dir, remove_dir_all},
    path::Path as path,
};
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::{auth::permissions::Permission, internal_error, ConnectionPool};
async fn add_bucket(
    user_id: &Uuid,
    bucket_name: &str,
    conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
) -> Result<(), (StatusCode, String)> {
    set_current_dir("/storage").map_err(internal_error)?;
    create_dir(&bucket_name).map_err(internal_error)?;
    let id = Uuid::new_v4();
    let creation_date = Utc::now().timestamp();
    let statement = conn
        .prepare(
            "INSERT INTO buckets (bucket_id, name, creation_date, creator) VALUES ($1, $2, $3, $4)",
        )
        .await
        .map_err(internal_error)?;
    conn.execute(&statement, &[&id, &bucket_name, &creation_date, &user_id])
        .await
        .map_err(internal_error)?;
    let permission_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO permissions(permission_id, \"user\", bucket, permission_option) VALUES ($1, $2, $3, (SELECT permission_option_id FROM permission_options WHERE name = $4))",
        &[&permission_id, &user_id, &id, &"Owner"],
    )
    .await
    .map_err(internal_error)?;
    Ok(())
}
pub async fn create_bucket(
    Extension(user_id): Extension<Uuid>,
    State(pool): State<ConnectionPool>,
    Path(bucket_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    match std::path::Path::new("/storage").exists() {
        true => {
            add_bucket(&user_id, &bucket_name, &conn).await?;
            return Ok(Json(json!({"result":"Successfuly created bucket"})));
        }
        false => {
            create_dir("storage").map_err(internal_error)?;
            add_bucket(&user_id, &bucket_name, &conn).await?;
            return Ok(Json(json!({"result":"Successfuly created bucket"})));
        }
    }
}
pub async fn delete_bucket(
    Extension(perms): Extension<Vec<Permission>>,
    State(pool): State<ConnectionPool>,
    Path(bucket_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to delete a bucket was granted".to_owned(),
        ));
    }
    let conn = pool.get().await.map_err(internal_error)?;
    set_current_dir("/storage").map_err(internal_error)?;
    remove_dir_all(&bucket_name).map_err(internal_error)?;
    let get_bucket_id_statement = conn
        .prepare("SELECT bucket_id FROM buckets WHERE name = $1")
        .await
        .map_err(internal_error)?;
    let bucket_row = conn
        .query_one(&get_bucket_id_statement, &[&bucket_name])
        .await
        .map_err(internal_error)?;
    let bucket_id: Uuid = bucket_row.get(0);
    let delete_objects_statement = conn
        .prepare("DELETE FROM objects WHERE bucket = $1")
        .await
        .map_err(internal_error)?;
    conn.execute(&delete_objects_statement, &[&bucket_id])
        .await
        .map_err(internal_error)?;
    let statement = conn
        .prepare("DELETE FROM buckets WHERE bucket_id = $1")
        .await
        .map_err(internal_error)?;
    conn.execute(&statement, &[&bucket_id])
        .await
        .map_err(internal_error)?;
    return Ok(Json(json!({"result":"Bucket deleted"})));
}

pub async fn head_bucket(
    Extension(perms): Extension<Vec<Permission>>,
    Path(bucket_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) || !perms.contains(&Permission::Read) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to head a bucket was granted".to_owned(),
        ));
    }
    set_current_dir("/storage").map_err(internal_error)?;
    match path::new(&bucket_name).exists() {
        true => return Ok(Json(json!({"exists":true}))),
        false => return Ok(Json(json!({"exists":false}))),
    }
}

pub async fn read_bucket(
    State(pool): State<ConnectionPool>,
    Extension(perms): Extension<Vec<Permission>>,
    Path(bucket_name): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) || !perms.contains(&Permission::Read) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to delete a bucket was granted".to_owned(),
        ));
    }

    let conn = pool.get().await.map_err(internal_error)?;
    let objects = conn.query("SELECT name FROM objects WHERE bucket = (SELECT bucket_id FROM buckets WHERE name = $1)", &[&bucket_name]).await.map_err(internal_error)?.iter().map(|row|row.get(0)).collect::<Vec<String>>();
    return Ok(Json(json!(objects)).into_response());
}
