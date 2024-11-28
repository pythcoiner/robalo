pub mod body;
pub mod error;
pub mod mattermost;

use crate::body::Body;
use crate::error::Error;
use axum::{
    body::to_bytes,
    http::{HeaderMap, Request, Response, StatusCode},
    routing::post,
    Router,
};
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/alert", post(handle_alert))
        .fallback(fallback_handler);

    // early check that no env var is missing
    env::var("MATTERMOST_TOKEN").expect("MATTERMOST_TOKEN missing in .env!");
    env::var("MATTERMOST_CHANNEL_ID").expect("MATTERMOST_CHANNEL_ID missing in .env!");
    env::var("MATTERMOST_BASE_URL").expect("MATTERMOST_BASE_URL missing in .env!");
    env::var("SENTRY_SECRET").expect("SENTRY_SECRET missing in .env!");
    let bind = env::var("BIND").expect("BIND missing from .env !");

    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(l) => {
            tracing::info!(
                "{} listening on {}.",
                env::args().next().expect("binary"),
                bind
            );
            l
        }
        Err(e) => {
            panic!(
                "{} fail to bind to {}: {}",
                env::args().next().expect("binary"),
                bind,
                e
            );
        }
    };
    axum::serve(listener, app).await.unwrap();
}

fn verify_hmac(headers: &HeaderMap, body: &[u8], secret: &[u8]) -> Result<bool, Error> {
    let expected_digest = headers
        .get("sentry-hook-signature")
        .ok_or(Error::MissingHeaderEntry("sentry-hook-signature"))?
        .to_str()
        .map_err(|e| Error::ToStr("sentry-hook-signature", e))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| Error::InvalidSecret)?;
    mac.update(body);
    let computed_digest = hex::encode(mac.finalize().into_bytes());

    Ok(expected_digest == computed_digest)
}

pub fn server_error() -> Response<axum::body::Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "text/plain")
        .body("Internal server Error".into())
        .expect("hardcoded")
}

pub fn bad_request() -> Response<axum::body::Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain")
        .body("Bad request".into())
        .expect("hardcoded")
}

pub fn ok() -> Response<axum::body::Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("Ok".into())
        .expect("hardcoded")
}

async fn handle_alert(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let secret: String = env::var("SENTRY_SECRET").expect("SENTRY_SECRET missing in .env!");
    let headers = req.headers().to_owned();
    let body_bytes = match to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(
                "handle_alert() failed to convert `Body` into `Bytes`: {}",
                e
            );
            return server_error();
        }
    };

    match verify_hmac(&headers, &body_bytes, secret.as_bytes()) {
        Ok(true) => {
            let body_str = String::from_utf8_lossy(&body_bytes);
            let body = match serde_json::from_str::<Value>(&body_str) {
                Ok(b) => Body::new(b),
                Err(e) => {
                    tracing::error!("handle_alert() fail to parse body as json: {}", e);
                    return bad_request();
                }
            };

            let action = match body.action() {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("handle_alert() fail to get action from body: {}", e);
                    return ok();
                }
            };

            match action {
                action @ body::Action::Unknown => {
                    tracing::warn!("Unknown action: {:?}", action)
                }
                action => {
                    let base_url = env::var("MATTERMOST_BASE_URL").expect("already checked");
                    let token = env::var("MATTERMOST_TOKEN").expect("already checked");
                    let channel_id = env::var("MATTERMOST_CHANNEL_ID").expect("already checked");
                    let mattermost = mattermost::Client::new(base_url, token);
                    match mattermost.create_post(channel_id, action.to_string()) {
                        Ok(()) => {
                            tracing::debug!("created post for {:?}", action);
                        }
                        Err(e) => {
                            tracing::error!("fail to create post for action {:?}: {:?}", action, e);
                        }
                    }
                }
            }
        }
        e => {
            tracing::error!("verify_hmac() fails: {e:?}");
            return server_error();
        }
    }
    ok()
}

async fn fallback_handler(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().to_owned();

    tracing::warn!(
        "received a {} request at {}: \n headers: {:#?}",
        method,
        path,
        headers,
    );
    bad_request()
}
