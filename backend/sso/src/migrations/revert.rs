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
        Database::Users => {
            conn.execute("DROP TABLE IF EXISTS users CASCADE", &[])
                .await
                .map_err(internal_error)
                .unwrap();
        }
    }
    Ok(())
}
