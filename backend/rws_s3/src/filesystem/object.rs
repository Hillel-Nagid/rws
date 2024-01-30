use crate::internal_error;
use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        StatusCode,
    },
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::prelude::*;
use md5;
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{self, create_dir_all, read, File},
    io::{Error, ErrorKind, Write},
};
use tokio_postgres::NoTls;
use uuid::Uuid;

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub async fn create_object(
    State(pool): State<ConnectionPool>,
    Path((bucketid, path)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("/storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if create_dir_all(path_vec.join("/")).is_ok() {
                if set_current_dir(path_vec.join("/")).is_ok() {
                    let file = File::create(object_name);
                    match file.map_err(internal_error) {
                        Ok(mut f) => {
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
                                            content_type =
                                                field.text().await.map_err(internal_error)?
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
                                            if f.write(&data).is_err() {
                                                return Err((
                                                    StatusCode::INTERNAL_SERVER_ERROR,
                                                    "Failed writing to provided file".to_owned(),
                                                ));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            println!("parsed multipart");
                            let statement = conn.prepare("INSERT INTO \"objects\" (uid,name,upload_date,content_disposition,content_length,content_type,last_modified,etag,encrypted) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)").await.map_err(internal_error)?;
                            let id = Uuid::new_v4();
                            let upload_date = Utc::now().timestamp();
                            conn.execute(
                                &statement,
                                &[
                                    &id,
                                    &[bucketid, path].join("/").to_string(),
                                    &upload_date,
                                    &content_disposition,
                                    &content_length,
                                    &content_type,
                                    &upload_date,
                                    &etag,
                                    &encrypted,
                                ],
                            )
                            .await
                            .map_err(internal_error)?;
                            println!("executed");
                        }
                        Err(e) => return Err(e),
                    };
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

pub async fn get_object(
    State(pool): State<ConnectionPool>,
    Path((bucketid, path)): Path<(String, String)>,
) -> Result<Response, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("/storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if set_current_dir(path_vec.join("/")).is_ok() {
                match read(object_name).map_err(internal_error) {
                    Ok(file) => {
                        let statement = conn.prepare(
                            "SELECT content_type, content_disposition FROM objects WHERE name=$1",
                        ).await.map_err(internal_error)?;
                        let row = conn
                            .query_one(&statement, &[&[bucketid, path].join("/").to_string()])
                            .await
                            .map_err(internal_error)?; // Could'nt find object
                        let content_type: String = row.get(0);
                        let content_disposition: String = row.get(1);
                        return Ok((
                            StatusCode::OK,
                            AppendHeaders([
                                (CONTENT_TYPE, content_type),
                                (CONTENT_DISPOSITION, content_disposition),
                            ]),
                            file,
                        )
                            .into_response());
                    }

                    Err(err) => return Err(err),
                }
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
