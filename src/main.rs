#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Serialize;
use std::{
    collections::HashSet,
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
    cached_tarballs: RwLock<HashSet<String>>,
    tarball_bucket: Box<Bucket>,
}

#[tokio::main]
async fn main() {
    let account =
        std::env::var("CLOUDFLARE_ACCOUNT").expect("Please set the CLOUDFLARE_ACCOUNT variable");

    let bucket = std::env::var("CACHE_BUCKET").expect("Please set CACHE_BUCKET variable");

    let bucket = Bucket::new(
        bucket.as_str(),
        Region::R2 {
            account_id: account,
        },
        Credentials::default().expect("Please set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY"),
    )
    .expect("Bucket access failed");

    let shared_state = Arc::new(AppState {
        cached_tarballs: RwLock::new(HashSet::new()),
        tarball_bucket: bucket,
    });

    let app = Router::new()
        .route("/", get(fallback))
        .route("/{forge}/{owner}/{repo}/{archive}", get(serve_tarball))
        .fallback(fallback)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind local address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start axum server");
}

async fn serve_tarball(
    Path((forge, owner, repo, archive)): Path<(String, String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let Some(forgepath) = FORGES
        .iter()
        // rev.tar.gz == 47
        .find(|f| f.name() == forge)
    else {
        return fallback().await;
    };

    let tarball: Tarball = Tarball::new(*forgepath, owner, repo, archive);

    if state
        .cached_tarballs
        .read()
        .unwrap()
        .contains(&tarball.get_path())
    {
        let tarball_object = state
            .tarball_bucket
            .get_object(tarball.get_path())
            .await
            .expect("Unexpected entry in hashtable.");

        if tarball_object.status_code() == 200 {
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/gzip"),
                    (header::ETAG, tarball.get_path().as_str()),
                ],
                tarball_object.bytes().clone(),
            )
                .into_response()
        } else {
            println!(
                "We probably shouldn't have reached this state, but here we are... redirecting just in case"
            );
            Redirect::temporary(tarball.get_url().as_str()).into_response()
        }
    } else {
        let exists = state
            .tarball_bucket
            .object_exists(tarball.get_path())
            .await
            .expect("Failed to check bucket");

        if exists {
            state
                .cached_tarballs
                .write()
                .unwrap()
                .insert(tarball.get_path());

            return (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/gzip"),
                    (header::ETAG, tarball.get_path().as_str()),
                ],
                state
                    .tarball_bucket
                    .get_object(tarball.get_path())
                    .await
                    .unwrap()
                    .bytes()
                    .clone(),
            )
                .into_response();
        }

        let tarball_download = reqwest::get(tarball.get_url())
            .await
            .expect("Upstream tarball could not be downloaded");

        if tarball_download.status() != reqwest::StatusCode::OK {
            return (tarball_download.status(), "upstream returned an error").into_response();
        }

        let tarball_bytes = tarball_download.bytes().await.expect("Encountered error");
        let passed_bytes = tarball_bytes.clone();

        // don't want upload to block serving
        tokio::spawn(async move {
            state
                .tarball_bucket
                .put_object(tarball.get_path(), &passed_bytes)
                .await
                .expect("Failed to upload tarball to s3");

            state
                .cached_tarballs
                .write()
                .unwrap()
                .insert(tarball.get_path());
        });

        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/gzip")],
            tarball_bytes,
        )
            .into_response()
    }
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
