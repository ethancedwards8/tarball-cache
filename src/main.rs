use axum::{
    Json, Router, extract::Path, http::StatusCode, response::{IntoResponse, Redirect, Response}, routing::get
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
        .route("/github/{owner}/{repo}/{archive}", get(serve_tarball))
        .fallback(fallback);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

   axum::serve(listener, app).await.unwrap();
}

async fn serve_tarball(Path((owner, repo, archive)): Path<(String, String, String)>) -> Response {


    Redirect::to(format!("https://github.com/{owner}/{repo}/archive/{archive}").as_str()).into_response()
}

async fn fallback() -> (StatusCode, Json<ErrResponse>) {
    (StatusCode::BAD_REQUEST, Json(ErrResponse{ error: "expected /github/{owner}/{repo}/{archive} with .tar.gz and .zip supported"}))
}
