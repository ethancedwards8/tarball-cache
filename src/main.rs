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

struct Tarball {
    forge: String, // should be an enum but will handle later
    owner: String,
    repo: String,
    archive: String,
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

#[inline]
fn create_cache_key(tarball: &Tarball) -> String {
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("{forge}-{owner}-{repo}-{archive}")
}

#[inline]
fn get_bucket_path(tarball: &Tarball) -> String {
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("{forge}/{owner}/{repo}/{archive}")
}

#[inline]
fn github_url(tarball: &Tarball) -> String {
    #[allow(unused_variables)]
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("https://github.com/{owner}/{repo}/archive/{archive}")
}

#[inline]
fn serve_github_upstream(tarball: &Tarball) -> Response {
    #[allow(unused_variables)]
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;

    let upstream_url = github_url(&tarball);

    Redirect::temporary(upstream_url.as_str()).into_response()
}

async fn serve_github_tarball(
    Path((owner, repo, archive)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let tarball: Tarball = Tarball {
        forge: "github".to_string(),
        owner,
        repo,
        archive,
    };

    let key = create_cache_key(&tarball);

    if state.cached_tarballs.read().unwrap().contains_key(&key) {
        let tarball_object = state
            .tarball_bucket
            .get_object(get_bucket_path(&tarball))
            .await
            .unwrap();

        if tarball_object.status_code() == 200 {
            (StatusCode::OK, tarball_object.bytes().clone()).into_response()
        } else {
            println!("We probably shouldn't have reached this state, but here we are...");
            return serve_github_upstream(&tarball);
        }
    } else {
        {
            let res = reqwest::get(github_url(&tarball))
                .await
                .expect("Tarball could not be fetched")
                .bytes()
                .await
                .unwrap();

            state
                .tarball_bucket
                .put_object(get_bucket_path(&tarball), &res)
                .await
                .expect("Failed to upload tarball to s3");

            state
                .cached_tarballs
                .write()
                .unwrap()
                .insert(create_cache_key(&tarball), true);

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
