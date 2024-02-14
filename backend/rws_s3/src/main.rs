mod utils;
mod migrations {
    pub mod begin;
    pub mod database;
    pub mod permission_values;
    pub mod revert;
}
use auth::{
    jwt, permissions,
    users::{signin, signup},
};
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware,
    routing::{delete, get, head, post, put},
    Router,
};
mod filesystem {
    pub mod bucket;
    pub mod object;
}
mod auth {
    pub mod jwt;
    pub mod permissions;
    pub mod users;
}
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use filesystem::{
    bucket::{create_bucket, delete_bucket, head_bucket},
    object::{create_object, delete_object, head_object, read_object},
};
use migrations::{
    begin::create_db, database::Database, permission_values::set_initial_permissions,
    revert::revert_db,
};
use tokio_postgres::NoTls;
const MB: usize = 1048576;
pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub enum Routes {
    Signup,
    Signin,
    CreateBucket,
    DeleteBucket,
    HeadBucket,
    Object,
    Unknown,
}
impl Routes {
    fn as_str(&self) -> &str {
        match self {
            Routes::Signup => "/signup",
            Routes::Signin => "/signin",
            Routes::CreateBucket => "/create_bucket/:bucket_name",
            Routes::DeleteBucket => "/delete_bucket/:bucket_name",
            Routes::HeadBucket => "/head_bucket/:bucket_name",
            Routes::Object => "/:bucket_name/*objectpath",
            Routes::Unknown => "/not-found",
        }
    }
}
impl From<String> for Routes {
    fn from(value: String) -> Self {
        match value.as_str() {
            "/signup" => Routes::Signup,
            "/signin" => Routes::Signin,
            "/create_bucket/:bucket_name" => Routes::CreateBucket,
            "/delete_bucket/:bucket_name" => Routes::DeleteBucket,
            "/head_bucket/:bucket_name" => Routes::HeadBucket,
            "/:bucket_name/*objectpath" => Routes::Object,
            _ => Routes::Unknown,
        }
    }
}
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
    revert_db(Database::Buckets, &conn).await.unwrap();
    revert_db(Database::Objects, &conn).await.unwrap();
    create_db(Database::PermisssionOptions, &conn)
        .await
        .unwrap();
    create_db(Database::Permisssions, &conn).await.unwrap();
    create_db(Database::Users, &conn).await.unwrap();
    create_db(Database::Buckets, &conn).await.unwrap();
    create_db(Database::Objects, &conn).await.unwrap();
    set_initial_permissions(&conn).await.unwrap();
    let app = Router::new()
        .route(Routes::Signup.as_str(), post(signup))
        .route(Routes::Signin.as_str(), get(signin))
        .route(Routes::CreateBucket.as_str(), put(create_bucket))
        .route(Routes::DeleteBucket.as_str(), delete(delete_bucket))
        .route(Routes::HeadBucket.as_str(), head(head_bucket))
        .route(
            Routes::Object.as_str(),
            put(create_object)
                .get(read_object)
                .delete(delete_object)
                .head(head_object),
        )
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            permissions::check_permissions,
        ))
        .layer(middleware::from_fn(jwt::auth_check))
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
