use axum::{
    body::to_bytes,
    extract::Json,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::Value;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/alert", post(log_payload))
        .fallback(fallback_handler);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// async fn log_payload(Json(payload): Json<Value>) -> impl IntoResponse {
//     tracing::info!("Received payload: {:#?}", payload);
//
//     "Ok"
// }

async fn log_payload(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().to_owned();
    let body_bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    let body: Value = serde_json::from_str(&body_str).unwrap();

    tracing::info!(
        "Received a {} request for {}: \n headers: {:#?} \n body: {:#?}",
        method,
        path,
        headers,
        body
    );
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body("404 - Not Found".into())
        .unwrap()
}

async fn fallback_handler(req: Request<axum::body::Body>) -> Response<axum::body::Body> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().to_owned();
    let body_bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    tracing::info!(
        "Received a {} request for {}: \n headers: {:#?} \n body: {:#?}",
        method,
        path,
        headers,
        body_str
    );

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body("404 - Not Found".into())
        .unwrap()
}
