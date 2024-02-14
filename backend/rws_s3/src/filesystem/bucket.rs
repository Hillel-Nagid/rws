use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env::set_current_dir,
    fs::{create_dir, read, read_dir, remove_dir_all, File},
    path::Path as path,
    rc::Rc,
};
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::{auth::permissions::Permission, internal_error, ConnectionPool};
async fn add_bucket(
    user_id: &Uuid,
    bucket_name: &str,
    conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
) -> Result<(), (StatusCode, String)> {
    set_current_dir("storage").map_err(internal_error)?;
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
        "INSERT INTO permissions(permission_id, user, bucket, permission) VALUES ($1, $2, $3, $4)",
        &[&permission_id, &user_id, &id, &"Owner"],
    )
    .await
    .map_err(internal_error)?;
    Ok(())
}
pub async fn create_bucket(
    State(pool): State<ConnectionPool>,
    Path(bucket_name): Path<String>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    match std::path::Path::new("storage").exists() {
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

fn list_dir(path: String) -> Result<Vec<(String, Vec<u8>)>, (StatusCode, String)> {
    let mut buff = vec![];
    let entries = read_dir(path).map_err(internal_error)?;
    for entry in entries {
        let dir_entry = entry.map_err(internal_error)?;
        let meta = dir_entry.metadata().map_err(internal_error)?;
        let entry_path = dir_entry.path();
        let entry_path_str = entry_path.to_str();
        if let Some(entry_path_some) = entry_path_str {
            if meta.is_dir() {
                let mut subdir = list_dir(entry_path_some.to_owned())?;
                buff.append(&mut subdir)
            }
            if meta.is_file() {
                let file = read(entry_path_some.to_owned()).map_err(internal_error)?;
                buff.push((entry_path_some.to_owned(), file))
            }
        }
    }
    Ok(buff)
}

pub async fn read_bucket(
    State(pool): State<ConnectionPool>,
    Extension(perms): Extension<Vec<Permission>>,
    Path(bucket_name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) || !perms.contains(&Permission::Read) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to delete a bucket was granted".to_owned(),
        ));
    }

    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("/storage/{}", bucket_name)).is_ok() {
        let objects = list_dir("".to_string())?;
        Ok(objects.into_response())
    }
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed reading bucket".to_owned(),
    ))
}
//                 let file = read(object_name).map_err(internal_error)?;
//                 let statement = conn.prepare(
//                             "SELECT content_type, content_disposition, last_modified, etag FROM objects WHERE name=$1",
//                         ).await.map_err(internal_error)?;
//                 let row = conn
//                     .query_one(&statement, &[&[bucket_name, path].join("/").to_string()])
//                     .await
//                     .map_err(internal_error)?; // Could'nt find object
//                 let content_type: String = row.get(0);
//                 let content_disposition: String = row.get(1);
//                 let last_modified: i64 = row.get(2);
//                 let etag: String = row.get(3);
//                 let result_response = (
//                     StatusCode::OK,
//                     AppendHeaders([
//                         (CONTENT_TYPE, content_type),
//                         (CONTENT_DISPOSITION, content_disposition),
//                     ]),
//                     file,
//                 )
//                     .into_response();

//                 let match_check = check_match(&headers, etag).map_err(internal_error)?;
//                 let since_check = check_since(&headers, last_modified).map_err(internal_error)?;

//                 if let Some(is_match) = match_check {
//                     if !is_match {
//                         return Ok(
//                             "Content didn't match 'match' headers restrictions".into_response()
//                         );
//                     }
//                 }
//                 if let Some(is_since) = since_check {
//                     if !is_since {
//                         return Ok(
//                             "Content didn't match 'since' headers restrictions".into_response()
//                         );
//                     }
//                 }
//                 return Ok(result_response);
//             } else {
//                 return Err(internal_error(Error::new(
//                     ErrorKind::Other,
//                     "Could't set work dir as file path",
//                 )));
//             }
//         } else {
//             return Err(internal_error(Error::new(
//                 ErrorKind::Other,
//                 "Could't get file name",
//             )));
//         }
//     } else {
//         return Err(internal_error(Error::new(
//             ErrorKind::Other,
//             "Could't find bucket",
//         )));
//     }
