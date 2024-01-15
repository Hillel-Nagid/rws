use axum::{extract::DefaultBodyLimit, routing::put, Router};
mod filesystem {
    pub mod bucket;
    pub mod object;
}
use filesystem::{
    bucket::create_bucket,
    object::{create_object, get_object},
};
use tokio_postgres::{Error, NoTls};

#[tokio::main]
async fn main() {
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres", NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let app = Router::new()
        .route("/createBucket/:bucketid", put(create_bucket))
        .route("/:bucketid/*objectpath", put(create_object).get(get_object))
        .layer(DefaultBodyLimit::max(204800)) //limits to 200MB file upload
        .with_state(&client); //Fix lifetime issue
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2945").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
