mod auth {
    pub mod jwt;
    pub mod users;
}
mod migrations {
    pub mod begin;
    pub mod database;
    pub mod revert;
}
use auth::{
    jwt,
    users::{signin, signup},
};
use migrations::{begin::create_db, database::Database};

use axum::{
    extract::DefaultBodyLimit,
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use tower_http::cors::{Any, CorsLayer};
pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;
pub enum Routes {
    Signup,
    Signin,
}
impl Routes {
    fn as_str(&self) -> &str {
        match self {
            Routes::Signup => "/signup",
            Routes::Signin => "/signin",
        }
    }
}
impl From<String> for Routes {
    fn from(value: String) -> Self {
        if value.starts_with("/signup") {
            return Routes::Signup;
        }
        Routes::Signin
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
    create_db(Database::Users, &conn).await.unwrap();
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route(Routes::Signup.as_str(), post(signup))
        .route(Routes::Signin.as_str(), get(signin))
        .layer(cors)
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2946").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
