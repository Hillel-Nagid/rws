use axum::{
    extract::{Multipart, Path, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{create_dir_all, read, File},
    io::{Error, ErrorKind, Write},
};
use tokio_postgres::NoTls;

use crate::internal_error;

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub async fn create_object(
    State(pool): State<ConnectionPool>,
    Path((bucketid, path)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    if set_current_dir(format!("storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if create_dir_all(path_vec.join("/")).is_ok() {
                if set_current_dir(path_vec.join("/")).is_ok() {
                    let file = File::create(object_name);
                    match file.map_err(internal_error) {
                        Ok(mut f) => {
                            while let Some(field) = multipart.next_field().await.unwrap() {
                                let data = field.bytes().await.unwrap();
                                if f.write(&data).is_err() {
                                    return Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "Failed writing to provided file".to_owned(),
                                    ));
                                }
                            }
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
        return Err((StatusCode::BAD_REQUEST, "Couln't find bucket".to_owned()));
    }
}

pub async fn get_object(
    Path((bucketid, path)): Path<(String, String)>,
) -> Result<Response, (StatusCode, String)> {
    if set_current_dir(format!("storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if set_current_dir(path_vec.join("/")).is_ok() {
                match read(object_name).map_err(internal_error) {
                    Ok(file) => {
                        return Ok((
                            StatusCode::OK,
                            AppendHeaders([(CONTENT_TYPE, "text/plain")]),
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
