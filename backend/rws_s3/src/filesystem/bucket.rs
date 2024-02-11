use std::{env::set_current_dir, fs::create_dir};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{internal_error, ConnectionPool};

pub async fn create_bucket(
    State(pool): State<ConnectionPool>,
    Path(bucketid): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    match std::path::Path::new("storage").exists() {
        true => match set_current_dir("storage").map_err(internal_error) {
            Ok(_) => match create_dir(&bucketid).map_err(internal_error) {
                Ok(_) => {
                    let id = Uuid::new_v4();
                    let creation_date = Utc::now().timestamp();
                    let statement =
                        conn.prepare("INSERT INTO buckets (bucket_id, name, creation_date, creator) VALUES ($1, $2, $3, $4)").await.map_err(internal_error)?;
                    conn.execute(&statement, &[&id, &bucketid, &creation_date])
                        .await
                        .map_err(internal_error)?;
                    return Ok(Json(json!({"result":"Successfuly created bucket"})));
                }
                Err(err) => return Err(err),
            },
            Err(err) => return Err(err),
        },
        false => match create_dir("storage").map_err(internal_error) {
            Ok(_) => match set_current_dir("storage").map_err(internal_error) {
                Ok(_) => match create_dir(bucketid).map_err(internal_error) {
                    Ok(_) => return Ok(Json(json!({"result":"Successfuly created bucket"}))),
                    Err(err) => return Err(err),
                },
                Err(err) => {
                    return Err(err);
                }
            },
            Err(err) => return Err(err),
        },
    }
}
