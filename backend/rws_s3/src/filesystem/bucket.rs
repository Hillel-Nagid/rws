use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{create_dir, remove_dir_all},
    path::Path as path,
};
use uuid::Uuid;

use crate::{internal_error, ConnectionPool};

pub async fn create_bucket(
    State(pool): State<ConnectionPool>,
    Path(bucket_name): Path<String>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    match std::path::Path::new("storage").exists() {
        true => {
            set_current_dir("storage").map_err(internal_error)?;
            create_dir(&bucket_name).map_err(internal_error)?;
            let id = Uuid::new_v4();
            let creation_date = Utc::now().timestamp();
            let statement =
                        conn.prepare("INSERT INTO buckets (bucket_id, name, creation_date, creator) VALUES ($1, $2, $3, $4)").await.map_err(internal_error)?;
            conn.execute(&statement, &[&id, &bucket_name, &creation_date, &user_id])
                .await
                .map_err(internal_error)?;
            return Ok(Json(json!({"result":"Successfuly created bucket"})));
        }
        false => {
            create_dir("storage").map_err(internal_error)?;
            set_current_dir("storage").map_err(internal_error)?;
            create_dir(bucket_name).map_err(internal_error)?;
            return Ok(Json(json!({"result":"Successfuly created bucket"})));
        }
    }
}
pub async fn delete_bucket(
    State(pool): State<ConnectionPool>,
    Path(bucket_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    set_current_dir("/storage").map_err(internal_error)?;
    remove_dir_all(&bucket_name).map_err(internal_error)?;
    let get_bucket_id_statement = conn
        .prepare("SELECT bucket_id WHERE name = $1")
        .await
        .map_err(internal_error)?;
    let bucket_row = conn
        .query_one(&get_bucket_id_statement, &[&bucket_name])
        .await
        .map_err(internal_error)?;
    let bucket_id: String = bucket_row.get(0);
    let delete_objects_statement = conn
        .prepare("DELETE FROM objects WHERE bucket_id = $1")
        .await
        .map_err(internal_error)?;
    conn.execute(&delete_objects_statement, &[&bucket_id])
        .await
        .map_err(internal_error)?;
    let statement = conn
        .prepare("DELETE FROM buckets WHERE name = $1")
        .await
        .map_err(internal_error)?;
    conn.execute(&statement, &[&bucket_id])
        .await
        .map_err(internal_error)?;
    return Ok(Json(json!({"result":"Bucket deleted"})));
}

pub async fn head_bucket(
    Path(bucket_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    set_current_dir("/storage").map_err(internal_error)?;
    match path::new(&bucket_name).exists() {
        true => Ok(Json(json!({"exists":true}))),
        false => Ok(Json(json!({"exists":false}))),
    }
}
