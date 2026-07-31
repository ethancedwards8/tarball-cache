use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Redirect, Response}, routing::get
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Serialize)]
struct ErrResponse {
    error: &'static str
}

struct AppState {
    cached_tarballs: RwLock<HashMap<String, bool>>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        cached_tarballs: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/", get(fallback))
        .route("/github/{owner}/{repo}/{archive}", get(serve_github_tarball))
        .fallback(fallback)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

   axum::serve(listener, app).await.unwrap();
}

fn create_github_key(owner: String, repo: String, archive: String) -> String {
    format!("github-{owner}-{repo}-{archive}")
}

async fn serve_github_tarball(Path((owner, repo, archive)): Path<(String, String, String)>, State(state): State<Arc<AppState>>) -> Response {
    let key = create_github_key(owner.clone(), repo.clone(), archive.clone());

    let value = state.cached_tarballs.read().unwrap().contains_key(&key);

    if value {
        (StatusCode::OK, format!("yay")).into_response()
    } else {
        let data = state.cached_tarballs.write().unwrap().insert(key, true);
        Redirect::temporary(format!("https://github.com/{owner}/{repo}/archive/{archive}").as_str()).into_response()
    }
}

async fn fallback() -> (StatusCode, Json<ErrResponse>) {
    (StatusCode::BAD_REQUEST, Json(ErrResponse{ error: "expected /github/{owner}/{repo}/{archive} with .tar.gz and .zip supported"}))
}
