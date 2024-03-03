use crate::{
    auth::permissions::Permission,
    internal_error,
    utils::{check_match, check_since},
    ConnectionPool,
};
use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::{AppendHeaders, IntoResponse, Response},
    Extension, Json,
};
use chrono::prelude::*;
use md5;
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{self, create_dir_all, read, remove_file, File},
    io::{Error, ErrorKind, Write},
    path::Path as path,
};
use uuid::Uuid;

pub async fn create_object(
    State(pool): State<ConnectionPool>,
    Path((bucket_name, path)): Path<(String, String)>,
    Extension(user_id): Extension<Uuid>,
    Extension(perms): Extension<Vec<Permission>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) && !perms.contains(&Permission::Write) {
        println!("{:?}", perms);
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to create an object was granted".to_owned(),
        ));
    }
    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("/storage/{}", bucket_name)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if create_dir_all(path_vec.join("/")).is_ok() {
                if set_current_dir(path_vec.join("/")).is_ok() {
                    let mut file = File::create(object_name).map_err(internal_error)?;
                    let mut content_disposition = String::from("inline");
                    let mut content_length = 0u32;
                    let mut content_type = String::from("text/plain");
                    let mut etag = String::from("");
                    let mut encrypted: bool = false;
                    while let Some(field) = multipart.next_field().await.unwrap() {
                        if let Some(name) = &field.name() {
                            match name {
                                &"content-disposition" => {
                                    content_disposition =
                                        field.text().await.map_err(internal_error)?
                                }
                                &"content-type" => {
                                    content_type = field.text().await.map_err(internal_error)?
                                }
                                &"encrypted" => {
                                    encrypted = match field
                                        .text()
                                        .await
                                        .map_err(internal_error)?
                                        .parse::<bool>()
                                    {
                                        Ok(value) => value,
                                        Err(_) => false,
                                    }
                                }
                                &"file" => {
                                    let data = field.bytes().await.unwrap();
                                    content_length = data.len() as u32;
                                    etag = format!("{:x}", md5::compute(&data));
                                    file.write(&data).map_err(internal_error)?;
                                }
                                _ => {}
                            }
                        }
                    }
                    let statement = conn.prepare("INSERT INTO \"objects\" (object_id,name,upload_date,content_disposition,content_length,content_type,last_modified,etag,encrypted,bucket,creator) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)").await.map_err(internal_error)?;
                    let id = Uuid::new_v4();
                    let upload_date = Utc::now().timestamp();
                    let bucket_id: Uuid = conn
                        .query_one(
                            "SELECT bucket_id FROM buckets WHERE name =  $1",
                            &[&bucket_name],
                        )
                        .await
                        .map_err(internal_error)?
                        .get(0);
                    conn.execute(
                        &statement,
                        &[
                            &id,
                            &[bucket_name, path].join("/").to_string(),
                            &upload_date,
                            &content_disposition,
                            &content_length,
                            &content_type,
                            &upload_date,
                            &etag,
                            &encrypted,
                            &bucket_id,
                            &user_id,
                        ],
                    )
                    .await
                    .map_err(internal_error)?;
                } else {
                    return Err(internal_error(Error::new(
                        ErrorKind::Other,
                        "Failed setting working dir to provided path",
                    )));
                }
            } else {
                return Err(internal_error(Error::new(
                    ErrorKind::Other,
                    "Could't find provided path",
                )));
            }
        };
        return Ok(Json(json!({"result":"Success"})));
    } else {
        let paths = fs::read_dir("./").unwrap();

        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }
        return Err((StatusCode::BAD_REQUEST, "Couln't find bucket".to_owned()));
    }
}

pub async fn read_object(
    State(pool): State<ConnectionPool>,
    Extension(perms): Extension<Vec<Permission>>,
    Path((bucket_name, path)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) && !perms.contains(&Permission::Read) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to read an object was granted".to_owned(),
        ));
    }
    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("/storage/{}", bucket_name)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if set_current_dir(path_vec.join("/")).is_ok() {
                let file = read(object_name).map_err(internal_error)?;
                let statement = conn.prepare(
                            "SELECT content_type, content_disposition, last_modified, etag FROM objects WHERE name=$1",
                        ).await.map_err(internal_error)?;
                let row = conn
                    .query_one(&statement, &[&[bucket_name, path].join("/").to_string()])
                    .await
                    .map_err(internal_error)?; // Could'nt find object
                let content_type: String = row.get(0);
                let content_disposition: String = row.get(1);
                let last_modified: i64 = row.get(2);
                let etag: String = row.get(3);
                let result_response = (
                    StatusCode::OK,
                    AppendHeaders([
                        (CONTENT_TYPE, content_type),
                        (CONTENT_DISPOSITION, content_disposition),
                    ]),
                    file,
                )
                    .into_response();

                let match_check = check_match(&headers, etag).map_err(internal_error)?;
                let since_check = check_since(&headers, last_modified).map_err(internal_error)?;

                if let Some(is_match) = match_check {
                    if !is_match {
                        return Ok(
                            "Content didn't match 'match' headers restrictions".into_response()
                        );
                    }
                }
                if let Some(is_since) = since_check {
                    if !is_since {
                        return Ok(
                            "Content didn't match 'since' headers restrictions".into_response()
                        );
                    }
                }
                return Ok(result_response);
            } else {
                return Err(internal_error(Error::new(
                    ErrorKind::Other,
                    "Could't set work dir as file path",
                )));
            }
        } else {
            return Err(internal_error(Error::new(
                ErrorKind::Other,
                "Could't get file name",
            )));
        }
    } else {
        return Err(internal_error(Error::new(
            ErrorKind::Other,
            "Could't find bucket",
        )));
    }
}
pub async fn delete_object(
    State(pool): State<ConnectionPool>,
    Extension(perms): Extension<Vec<Permission>>,

    Path((bucket_name, path)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) && !perms.contains(&Permission::Write) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to delete an object was granted".to_owned(),
        ));
    }
    let conn = pool.get().await.map_err(internal_error)?;
    set_current_dir(format!("/storage/{}", bucket_name)).map_err(internal_error)?;
    let mut path_vec: Vec<&str> = path.split("/").collect();
    if let Some(object_name) = path_vec.pop() {
        set_current_dir(path_vec.join("/")).map_err(internal_error)?;
        remove_file(object_name).map_err(internal_error)?;
        let statement = conn
            .prepare("DELETE FROM objects WHERE name = $1")
            .await
            .map_err(internal_error)?;
        conn.execute(&statement, &[&[bucket_name, path].join("/").to_string()])
            .await
            .map_err(internal_error)?;
        return Ok(Json(json!({"result":"Object deleted"})));
    } else {
        return Err(internal_error(Error::new(
            ErrorKind::Other,
            "Could't get file name",
        )));
    }
}

pub async fn head_object(
    Extension(perms): Extension<Vec<Permission>>,
    Path((bucket_name, path)): Path<(String, String)>,
) -> Result<Response, (StatusCode, String)> {
    if !perms.contains(&Permission::Owner) && !perms.contains(&Permission::Read) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "No permission to head an object was granted".to_owned(),
        ));
    }
    set_current_dir(format!("/storage/{}", bucket_name)).map_err(internal_error)?;
    let mut path_vec: Vec<&str> = path.split("/").collect();
    if let Some(object_name) = path_vec.pop() {
        set_current_dir(path_vec.join("/")).map_err(internal_error)?;
        match path::new(object_name).exists() {
            true => {
                let object_length = read(object_name).map_err(internal_error)?.len();
                return Ok((
                    StatusCode::OK,
                    AppendHeaders([(CONTENT_LENGTH, object_length)]),
                )
                    .into_response());
            }
            false => return Err((StatusCode::NOT_FOUND, "Object not found".to_owned())),
        }
    } else {
        return Err(internal_error(Error::new(
            ErrorKind::Other,
            "Could't get file name",
        )));
    }
}
