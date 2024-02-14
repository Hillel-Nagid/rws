use std::slice::Iter;

use crate::{internal_error, ConnectionPool, Routes};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    Write,
    Read,
    Owner,
    Uknonwn,
}
impl Permission {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Permission::Owner => Some("Owner"),
            Permission::Read => Some("Read"),
            Permission::Write => Some("Write"),
            Permission::Uknonwn => None,
        }
    }
    pub fn iter() -> Iter<'static, Permission> {
        static DIRECTIONS: [Permission; 4] = [
            Permission::Owner,
            Permission::Read,
            Permission::Write,
            Permission::Uknonwn,
        ];
        DIRECTIONS.iter()
    }
}
impl From<String> for Permission {
    fn from(value: String) -> Self {
        match value.as_str() {
            "write" => Permission::Write,
            "read" => Permission::Read,
            "owner" => Permission::Owner,
            _ => Permission::Uknonwn,
        }
    }
}
pub async fn check_permissions(
    State(pool): State<ConnectionPool>,
    Extension(user_id): Extension<Uuid>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let bucket_name;
    let conn = pool.get().await.map_err(internal_error)?;
    let req_uri = req.uri().to_string();
    match Routes::from(req_uri.clone()) {
        Routes::Signin | Routes::Signup | Routes::Unknown => return Ok(next.run(req).await),
        Routes::CreateBucket | Routes::DeleteBucket | Routes::HeadBucket => {
            bucket_name = req_uri.split("/").collect::<Vec<_>>()[1].to_owned()
        }
        Routes::Object => bucket_name = req_uri.split("/").collect::<Vec<_>>()[0].to_owned(),
    };
    let statement = conn
        .prepare(
            "SELECT po.name AS permission_option_name
FROM users u
INNER JOIN permissions p ON u.user_id = p.user
INNER JOIN permission_options po ON p.permission_option = po.permission_option_id
INNER JOIN buckets b ON p.bucket = b.bucket_id
WHERE u.user_id = $1
AND b.name = $2",
        )
        .await
        .map_err(internal_error)?;
    let permissions: Vec<String> = conn
        .query(&statement, &[&user_id, &bucket_name])
        .await
        .map_err(internal_error)?
        .iter()
        .map(|row| row.get(0))
        .collect();
    if permissions.len() > 0 {
        req.extensions_mut().insert(
            permissions
                .iter()
                .map(|perm| Permission::from(perm.clone()))
                .collect::<Vec<_>>(),
        );
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "No permissions available".to_owned(),
        ))
    }
}
