mod utils;
mod migrations {
    pub mod begin;
    pub mod database;
    pub mod permission_values;
    pub mod revert;
}
mod filesystem {
    pub mod bucket;
    pub mod object;
}
mod auth {
    pub mod jwt;
    pub mod permissions;
    pub mod users;
}
use tower_http::cors::{Any, CorsLayer};

use auth::{
    jwt, permissions,
    users::{signin, signup},
};
use axum::{
    extract::DefaultBodyLimit,
    http::{Method, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use filesystem::{
    bucket::{create_bucket, delete_bucket, read_bucket},
    object::{create_object, delete_object, head_object, read_object},
};
use migrations::{begin::create_db, database::Database};
use tokio_postgres::NoTls;
const MB: usize = 1048576;
pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub enum Routes {
    Signup,
    Signin,
    CreateBucket,
    DeleteBucket,
    GetBucket,
    Object,
}
impl Routes {
    fn as_str(&self) -> &str {
        match self {
            Routes::Signup => "/signup",
            Routes::Signin => "/signin",
            Routes::CreateBucket => "/create_bucket/:bucket_name",
            Routes::DeleteBucket => "/delete_bucket/:bucket_name",
            Routes::GetBucket => "/get_bucket/:bucket_name",
            Routes::Object => "/:bucket_name/*objectpath",
        }
    }
}
impl From<String> for Routes {
    fn from(value: String) -> Self {
        if value.starts_with("/signup") {
            return Routes::Signup;
        } else if value.starts_with("/signin") {
            return Routes::Signin;
        } else if value.starts_with("/create_bucket") {
            return Routes::CreateBucket;
        } else if value.starts_with("/delete_bucket") {
            return Routes::DeleteBucket;
        } else if value.starts_with("/get_bucket") {
            return Routes::GetBucket;
        }
        return Routes::Object;
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

    create_db(Database::PermisssionOptions, &conn)
        .await
        .unwrap();
    create_db(Database::Users, &conn).await.unwrap();
    create_db(Database::Buckets, &conn).await.unwrap();
    create_db(Database::Permisssions, &conn).await.unwrap();
    create_db(Database::Objects, &conn).await.unwrap();
    // set_initial_permissions(&conn).await.unwrap();
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::HEAD,
            Method::DELETE,
        ])
        .allow_origin(Any);

    let app = Router::new()
        .route(Routes::CreateBucket.as_str(), put(create_bucket))
        .route(Routes::DeleteBucket.as_str(), delete(delete_bucket))
        .route(Routes::GetBucket.as_str(), get(read_bucket))
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
        .route(Routes::Signup.as_str(), post(signup))
        .route(Routes::Signin.as_str(), get(signin))
        .layer(DefaultBodyLimit::max(200 * MB)) //limits to 200MB file upload
        .layer(cors)
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
