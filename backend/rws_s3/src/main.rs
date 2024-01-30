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
const MB: usize = 1024 * 1024;
#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=rws_db user=postgres port=5432",
        NoTls,
    )
    .map_err(internal_error)
    .unwrap();
    let pool = Pool::builder()
        .build(manager)
        .await
        .map_err(internal_error)
        .unwrap();
    let cloned_pool = pool.clone();
    let conn = cloned_pool.get().await.map_err(internal_error).unwrap();
    // conn.execute("DROP TABLE IF EXISTS objects", &[])
    //     .await
    //     .map_err(internal_error)
    //     .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS objects (
  uid uuid PRIMARY KEY,
  name text,
  upload_date int8,
  content_disposition text,
  content_length oid,
  content_type text,
  last_modified int8,
  etag text,
  encrypted bool
)",
        &[],
    )
    .await
    .map_err(internal_error)
    .unwrap();
    let app = Router::new()
        .route("/createBucket/:bucketid", put(create_bucket))
        .route("/:bucketid/*objectpath", put(create_object).get(get_object))
        .layer(DefaultBodyLimit::max(200 * MB)) //limits to 200MB file upload
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
