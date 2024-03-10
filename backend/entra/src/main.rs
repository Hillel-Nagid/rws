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
    CreateUser,
    ResetPwd,
    EnableUser,
    DisableUser,
    DeleteUser,
    SearchUser,
    GetUser,
    ModifyUser,
    Authorize,
}
impl Routes {
    fn as_path(&self) -> &str {
        match self {
            Routes::CreateUser => "/createUser",
            Routes::ResetPwd => "/resetPwd",
            Routes::EnableUser => "/enableUser",
            Routes::DisableUser => "/disableUser",
            Routes::DeleteUser => "/deleteUser",
            Routes::SearchUser => "/searchUser",
            Routes::GetUser => "/getUser",
            Routes::ModifyUser => "/modifyUser",
            Routes::Authorize => "/authorize",
        }
    }
}
impl From<String> for Routes {
    fn from(value: String) -> Self {
        match value.as_str() {
            "/createUser" => Routes::CreateUser,
            "/resetPwd" => Routes::ResetPwd,
            "/enableUser" => Routes::EnableUser,
            "/disableUser" => Routes::DisableUser,
            "/deleteUser" => Routes::DeleteUser,
            "/modifyUser" => Routes::ModifyUser,
            "/authorize" => Routes::Authorize,
            _ => {
                if value.starts_with("/searchUser") {
                    return Routes::SearchUser;
                }
                Routes::GetUser
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=rws_db user=postgres port=5432 database=entra",
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
        .route(Routes::CreateUser.as_path(), post(signup))
        .route(Routes::Authorize.as_path(), get(signin))
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
