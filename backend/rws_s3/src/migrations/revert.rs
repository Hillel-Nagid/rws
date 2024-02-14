use super::database::Database;
use crate::internal_error;
use axum::http::StatusCode;
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub async fn revert_db(
    db_name: Database,
    conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
) -> Result<(), (StatusCode, String)> {
    match db_name {
        Database::Buckets => {
            conn.execute("DROP TABLE IF EXISTS buckets", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
        Database::Users => {
            conn.execute("DROP TABLE IF EXISTS users", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
        Database::Objects => {
            conn.execute("DROP TABLE IF EXISTS objects", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
        Database::Permisssions => {
            conn.execute("DROP TABLE IF EXISTS permissions", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
        Database::PermisssionOptions => {
            conn.execute("DROP TABLE IF EXISTS permission_options", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
    }
    Ok(())
}
