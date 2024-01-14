use axum::{
    extract::{Multipart, Path},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::{
    env::set_current_dir,
    fs::{create_dir_all, read, File},
    io::Write,
};

pub async fn create_object(
    Path((bucketid, path)): Path<(String, String)>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    if set_current_dir(format!("storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if create_dir_all(path_vec.join("/")).is_ok() {
                if set_current_dir(path_vec.join("/")).is_ok() {
                    let file = File::create(object_name);
                    match file {
                        Ok(mut f) => {
                            while let Some(field) = multipart.next_field().await.unwrap() {
                                let data = field.bytes().await.unwrap();
                                if f.write(&data).is_err() {
                                    return (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(json!({
                                          "result":
                                          "Failed writing to provided file"
                                        })),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({
                                    "result":
                                        format!("Failed creating provided file with error: {:?}", e)
                                })),
                            )
                        }
                    };
                } else {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"result":"Failed setting working dir to provided path"})),
                    );
                }
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"result":"Failed creating provided path"})),
                );
            }
        };
        return (StatusCode::CREATED, Json(json!({"result":"Success"})));
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"result":"Couln't find bucket"})),
        );
    }
}

pub async fn get_object(Path((bucketid, path)): Path<(String, String)>) -> Response {
    if set_current_dir(format!("storage/{}", bucketid)).is_ok() {
        let mut path_vec: Vec<&str> = path.split("/").collect();
        if let Some(object_name) = path_vec.pop() {
            if set_current_dir(path_vec.join("/")).is_ok() {
                match read(object_name) {
                    Ok(file) => {
                        return (
                            StatusCode::OK,
                            AppendHeaders([(CONTENT_TYPE, "text/plain")]),
                            file,
                        )
                            .into_response();
                    }

                    Err(err) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "err": format!("Couln't find file. error: {:?}", err)
                            })),
                        )
                            .into_response();
                    }
                }
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"err":"Could't set work dir as file path"})),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"err":"Could't get file name"})),
            )
                .into_response();
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"err":"Couln't find bucket"})),
        )
            .into_response();
    }
}
