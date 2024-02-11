use auth::{
    jwt,
    users::{signin, signup},
};
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post, put},
    Router,
};
mod filesystem {
    pub mod bucket;
    pub mod object;
}
mod auth {
    pub mod jwt;
    pub mod users;
}
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use filesystem::{
    bucket::create_bucket,
    object::{create_object, get_object},
};
use tokio_postgres::NoTls;
const MB: usize = 1024 * 1024;
pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

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
    conn.execute("DROP TABLE IF EXISTS objects", &[])
        .await
        .map_err(internal_error)
        .unwrap();
    conn.execute("DROP TABLE IF EXISTS buckets", &[])
        .await
        .map_err(internal_error)
        .unwrap();
    conn.execute("DROP TABLE IF EXISTS users", &[])
        .await
        .map_err(internal_error)
        .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS buckets (
  user_id uuid NOT NULL UNIQUE,
  name text NOT NULL UNIQUE,
  password text NOT NULL,
  email text,
  PRIMARY KEY(user_id)
)",
        &[],
    )
    .await
    .map_err(internal_error)
    .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
  bucket_id uuid NOT NULL  UNIQUE,
  name text NOT NULL  UNIQUE,
  creation_date int8 NOT NULL,
  creator uuid NOT NULL,
  PRIMARY KEY(bucket_id)
  CONSTRAINT user_constraint
      FOREIGN KEY(creator) 
        REFERENCES buckets(bucket_id)
        ON DELETE SET NULL
)",
        &[],
    )
    .await
    .map_err(internal_error)
    .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS objects (
  object_id uuid NOT NULL   UNIQUE,
  name text NOT NULL   UNIQUE,
  upload_date int8 NOT NULL,
  content_disposition text NOT NULL,
  content_length oid NOT NULL,
  content_type text NOT NULL,
  last_modified int8 NOT NULL,
  etag text NOT NULL,
  encrypted bool NOT NULL,
  bucket_id uuid NOT NULL,
  PRIMARY KEY(object_id),
  CONSTRAINT bucket_constraint
      FOREIGN KEY(bucket_id) 
        REFERENCES buckets(bucket_id)
        ON DELETE CASCADE
)",
        &[],
    )
    .await
    .map_err(internal_error)
    .unwrap();
    let app = Router::new()
        .route("/signup", post(signup))
        .route("/signin", get(signin))
        .route("/createBucket/:bucketid", put(create_bucket))
        .route("/:bucketid/*objectpath", put(create_object).get(get_object))
        .layer(jwt::auth_check)
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
