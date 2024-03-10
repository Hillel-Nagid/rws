use super::database::Database;
use crate::internal_error;
use axum::http::StatusCode;
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub async fn create_db(
    db_name: Database,
    conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
) -> Result<(), (StatusCode, String)> {
    match db_name {
        Database::Users => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
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
        }
    }
    Ok(())
}
