use axum::{
    body::to_bytes,
    http::{header::ToStrError, HeaderMap, Request, Response, StatusCode},
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

const SECRET: &[u8] = "4e79b95ddaa36e719d4ddb1d48564f550fb091b68fdc15aa6386d8508ba86bf0".as_bytes();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/alert", post(handle_alert))
        .fallback(fallback_handler);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
pub enum Error {
    MissingHeaderEntry(&'static str),
    ToStr(&'static str, ToStrError),
    InvalidSecret,
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

async fn handle_alert(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let headers = req.headers().to_owned();
    let body_bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap();

    match verify_hmac(&headers, &body_bytes, SECRET) {
        Ok(true) => {
            let body_str = String::from_utf8_lossy(&body_bytes);
            let body: Value = serde_json::from_str(&body_str).unwrap();

            tracing::info!("headers: {:#?} \n body: {:#?}", headers, body);
        }
        e => {
            tracing::error!("{e:?}");
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("Ok".into())
        .expect("hardcoded")
}

async fn fallback_handler(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().to_owned();

    tracing::info!(
        "Received a {} request for {}: \n headers: {:#?}",
        method,
        path,
        headers,
    );

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body("404 - Not Found".into())
        .unwrap()
}
