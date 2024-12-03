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
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use std::{env, fs, net::IpAddr, process, str::FromStr, sync::Arc};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Conf {
    pub ip: IpAddr,
    pub port: u16,
    pub sentry_secret: String,
    pub mattermost_token: String,
    pub mattermost_channel_id: String,
    pub mattermost_base_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        tracing::error!("configuration file path must be passed as argument!");
        process::exit(1);
    }

    let raw_conf = match fs::read_to_string(args[1].clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(" fail to open configuration file at {}: {}", args[1], e);
            process::exit(1);
        }
    };

    let conf: Conf = match toml::from_str(&raw_conf) {
        Ok(conf) => {
            tracing::info!("configuration: {:#?}", conf);
            conf
        }
        Err(e) => {
            tracing::error!("fail to parse configuration file: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = Url::from_str(&conf.mattermost_base_url) {
        tracing::error!(
            "mattermost url is not valid {} : {}",
            conf.mattermost_base_url,
            e
        );
        process::exit(1);
    }

    let arc = Arc::new(conf);

    let app = Router::new()
        .route(
            "/alert",
            post({
                let conf = arc.clone();
                move |body| handle_alert(body, conf)
            }),
        )
        .fallback(fallback_handler);

    let bind = format!("{}:{}", arc.ip, arc.port);

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
            tracing::error!(
                "{} fail to bind to {}: {}",
                env::args().next().expect("binary"),
                bind,
                e
            );
            process::exit(1);
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

async fn handle_alert(
    req: Request<axum::body::Body>,
    conf: Arc<Conf>,
) -> Response<axum::body::Body> {
    let secret = &conf.sentry_secret;
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
                    let base_url = &conf.mattermost_base_url;
                    let token = &conf.mattermost_token;
                    let channel_id = &conf.mattermost_channel_id;
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
