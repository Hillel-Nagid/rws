use axum::{extract::DefaultBodyLimit, routing::put, Router};
mod filesystem {
    pub mod bucket;
    pub mod object;
}
use filesystem::{
    bucket::create_bucket,
    object::{create_object, get_object},
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/createBucket/:bucketid", put(create_bucket))
        .route("/:bucketid/*objectpath", put(create_object).get(get_object))
        .layer(DefaultBodyLimit::max(204800)); //limits to 200MB file upload
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2945").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
