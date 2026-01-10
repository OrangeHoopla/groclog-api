mod handler;
mod model;
mod response;
mod route;

use axum::{
    extract::DefaultBodyLimit,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use route::create_router;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let _cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router()
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(1000000000));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
