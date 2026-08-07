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
use utils::Tarball;

use crate::utils::FORGES;

#[derive(Serialize)]
struct ErrResponse {
    error: &'static str,
}

struct AppState {
    cached_tarballs: RwLock<HashMap<String, String>>,
    tarball_bucket: Box<Bucket>,
}

#[tokio::main]
async fn main() {
    let account =
        std::env::var("CLOUDFLARE_ACCOUNT").expect("Please set the CLOUDFLARE_ACCOUNT variable");

    let bucket = Bucket::new(
        "tarball-cache",
        Region::R2 {
            account_id: account,
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
        .route("/{forge}/{owner}/{repo}/{archive}", get(serve_tarball))
        .fallback(fallback)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn serve_tarball(
    Path((forge, owner, repo, archive)): Path<(String, String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(forgepath) = FORGES
        .iter()
        // rev.tar.gz == 47
        .find(|f| f.name() == forge && archive.len() == 47)
    else {
        return fallback().await;
    };

    let tarball: Tarball = Tarball::new(*forgepath, owner, repo, archive);

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
            Redirect::temporary(tarball.get_url().as_str()).into_response()
        }
    } else {
        let tarball_response = cache_miss(tarball, axum::extract::State(state));

        tarball_response.await
    }
}

async fn cache_miss(tarball: Tarball, State(state): State<Arc<AppState>>) -> Response {
    let tarball_download = reqwest::get(tarball.get_url())
        .await
        .expect("Upstream tarball could not be downloaded")
        .bytes()
        .await
        .expect("Encountered error");

    let passed_bytes = tarball_download.clone();

    tokio::spawn(async move {
        state
            .tarball_bucket
            .put_object(tarball.get_path(), &tarball_download)
            .await
            .expect("Failed to upload tarball to s3");

        state
            .cached_tarballs
            .write()
            .unwrap()
            .insert(tarball.get_key(), tarball.get_path());
    });

    (StatusCode::OK, passed_bytes).into_response()
}

async fn fallback() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrResponse {
            error: "expected /{github,gitlab,sourcehut,codeberg}/{owner}/{repo}/{commit}.tar.gz",
        }),
    )
        .into_response()
}
