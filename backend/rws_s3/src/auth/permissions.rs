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
    req.extensions_mut().insert(permissions);
    Ok(next.run(req).await)
}
