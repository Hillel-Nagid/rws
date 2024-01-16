use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, http::StatusCode, routing::put, Router};
mod filesystem {
    pub mod bucket;
    pub mod object;
}
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use filesystem::{
    bucket::create_bucket,
    object::{create_object, get_object},
};
use tokio_postgres::NoTls;
#[tokio::main]
async fn main() {
    let manager =
        PostgresConnectionManager::new_from_stringlike("host=localhost user=postgres", NoTls)
            .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/createBucket/:bucketid", put(create_bucket))
        .route("/:bucketid/*objectpath", put(create_object).get(get_object))
        .layer(DefaultBodyLimit::max(204800)) //limits to 200MB file upload
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2945").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
