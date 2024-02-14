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
        Database::Buckets => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS buckets (
  bucket_id uuid NOT NULL  UNIQUE,
  name text NOT NULL  UNIQUE,
  creation_date int8 NOT NULL,
  creator uuid,
  PRIMARY KEY(bucket_id)
  CONSTRAINT user_constraint
      FOREIGN KEY(creator) 
        REFERENCES users(user_id)
        ON DELETE SET NULL
)",
                &[],
            )
            .await
            .map_err(internal_error)
            .unwrap();
        }
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
        Database::Objects => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS objects (
  object_id uuid NOT NULL   UNIQUE,
  name text NOT NULL   UNIQUE,
  upload_date int8 NOT NULL,
  content_disposition text NOT NULL,
  content_length oid NOT NULL,
  content_type text NOT NULL,
  last_modified int8 NOT NULL,
  etag text NOT NULL,
  encrypted bool NOT NULL,
  bucket_id uuid NOT NULL,
  creator uuid,
  PRIMARY KEY(object_id),
  CONSTRAINT bucket_constraint
      FOREIGN KEY(bucket_id) 
        REFERENCES buckets(bucket_id)
        ON DELETE CASCADE
  CONSTRAINT creator_constraint
    FOREIGN KEY(creator) 
      REFERENCES users(user_id)
      ON DELETE SET NULL
)",
                &[],
            )
            .await
            .map_err(internal_error)
            .unwrap();
        }
        Database::Permisssions => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS permissions (
  permission_id uuid NOT NULL UNIQUE,
  user uuid,
  bucket uuid NOT NULL,
  permission_option uuid NOT NULL,
  PRIMARY KEY(permission_id),
  CONSTRAINT bucket_constraint
      FOREIGN KEY(bucket) 
        REFERENCES buckets(bucket_id)
        ON DELETE CASCADE
  CONSTRAINT user_constraint
    FOREIGN KEY(user) 
      REFERENCES users(user_id)
      ON DELETE SET NULL
  CONSTRAINT permission_option_constraint
    FOREIGN KEY(permission_option) 
      REFERENCES permission_options(permission_option_id)
      ON DELETE CASCADE
)",
                &[],
            )
            .await
            .map_err(internal_error)
            .unwrap();
        }
        Database::PermisssionOptions => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS permission_options (
  permission_option_id uuid NOT NULL UNIQUE,
  name text NOT NULL UNIQUE,
  PRIMARY KEY(permission_option_id)
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
