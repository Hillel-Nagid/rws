use std::{env::set_current_dir, fs::create_dir};

use axum::{extract::Path, Json};
use serde_json::{json, Value};

pub async fn create_bucket(Path(bucketid): Path<String>) -> Json<Value> {
    match std::path::Path::new("storage").exists() {
        true => {
            if set_current_dir("storage").is_ok() {
                Json(json!({"result":create_dir(bucketid).is_ok()}))
            } else {
                create_dir("storage").unwrap();
                Json(json!({"result":"Couln't create bucket"}))
            }
        }
        false => {
            create_dir("storage").unwrap();
            if set_current_dir("storage").is_ok() {
                Json(json!({"result":create_dir(bucketid).is_ok()}))
            } else {
                create_dir("storage").unwrap();
                Json(json!({"result":"Couln't create bucket"}))
            }
        }
    }
}
