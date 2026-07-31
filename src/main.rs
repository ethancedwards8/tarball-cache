use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use s3::Bucket;
use s3::Region;
use s3::creds::Credentials;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Serialize)]
struct ErrResponse {
    error: &'static str,
}

struct AppState {
    cached_tarballs: RwLock<HashMap<String, bool>>,
}

#[tokio::main]
async fn main() {
    let bucket = Bucket::new(
        "tarball-cache",
        Region::R2 {
            account_id: "14a8704b05622c623affefb0d8dd93d4".to_string(),
        },
        Credentials::default().unwrap(),
    )
    .unwrap();

    let content = "I want to go to R2".as_bytes();

    let _response_data = bucket.put_object("/test.file", content).await;

    let shared_state = Arc::new(AppState {
        cached_tarballs: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/", get(fallback))
        .route(
            "/github/{owner}/{repo}/{archive}",
            get(serve_github_tarball),
        )
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

async fn serve_github_tarball(
    Path((owner, repo, archive)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let key = create_github_key(owner.clone(), repo.clone(), archive.clone());

    if state.cached_tarballs.read().unwrap().contains_key(&key) {
        (StatusCode::OK, "yay".to_string()).into_response()
    } else {
        state.cached_tarballs.write().unwrap().insert(key, true);
        Redirect::temporary(format!("https://github.com/{owner}/{repo}/archive/{archive}").as_str())
            .into_response()
    }
}

async fn fallback() -> (StatusCode, Json<ErrResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrResponse {
            error: "expected /github/{owner}/{repo}/{archive} with .tar.gz and .zip supported",
        }),
    )
}
