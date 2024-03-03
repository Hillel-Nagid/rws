use axum::http::StatusCode;
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::{auth::permissions::Permission, internal_error};

pub async fn set_initial_permissions(
    conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
) -> Result<(), (StatusCode, String)> {
    let permission_options = Permission::iter()
        .filter_map(|perm| perm.as_str())
        .collect::<Vec<_>>();
    for permission in permission_options {
        let uid = Uuid::new_v4();
        conn.execute(
            "INSERT INTO permission_options(permission_option_id,name) VALUES ($1,$2)",
            &[&uid, &permission],
        )
        .await
        .map_err(internal_error)?;
    }
    Ok(())
}
