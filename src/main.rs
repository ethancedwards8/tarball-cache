use axum::{
    routing::get,
    Router,
    Json,
    http::StatusCode,
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrResponse {
    error: &'static str
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(fallback))
        .fallback(fallback);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

   axum::serve(listener, app).await.unwrap();
}

async fn fallback() -> (StatusCode, Json<ErrResponse>) {
    (StatusCode::BAD_REQUEST, Json(ErrResponse{ error: "expected /github/{owner}/{repo}/{rev}.tar.gz"}))
}
