#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub mod utils;
use utils::*;

#[derive(Serialize)]
struct ErrResponse {
    error: &'static str,
}

struct AppState {
    cached_tarballs: RwLock<HashMap<String, bool>>,
    tarball_bucket: Box<Bucket>,
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
    .expect("Bucket access failed");

    let shared_state = Arc::new(AppState {
        cached_tarballs: RwLock::new(HashMap::new()),
        tarball_bucket: bucket,
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

async fn serve_github_tarball(
    Path((owner, repo, archive)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let tarball: Tarball = Tarball::new("github".to_string(), owner, repo, archive);

    let key = tarball.get_key();

    if state.cached_tarballs.read().unwrap().contains_key(&key) {
        let tarball_object = state
            .tarball_bucket
            .get_object(tarball.get_path())
            .await
            .expect("Unexpected entry in hashtable.");

        if tarball_object.status_code() == 200 {
            (StatusCode::OK, tarball_object.bytes().clone()).into_response()
        } else {
            println!("We probably shouldn't have reached this state, but here we are...");
            return serve_github_upstream(&tarball);
        }
    } else {
        {
            let res = reqwest::get(tarball.get_url())
                .await
                .expect("Tarball could not be fetched")
                .bytes()
                .await
                .unwrap();

            state
                .tarball_bucket
                .put_object(tarball.get_path(), &res)
                .await
                .expect("Failed to upload tarball to s3");

            state
                .cached_tarballs
                .write()
                .unwrap()
                .insert(tarball.get_key(), true);

            state.cached_tarballs.write().unwrap().insert(key, true);
        }
        return serve_github_upstream(&tarball);
    }
}

async fn fallback() -> (StatusCode, Json<ErrResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrResponse {
            error: "expected /github/{owner}/{repo}/{archive} with .tar.gz",
        }),
    )
}

#[inline]
fn serve_github_upstream(tarball: &Tarball) -> Response {
    let upstream_url = tarball.get_url();

    Redirect::temporary(upstream_url.as_str()).into_response()
}
