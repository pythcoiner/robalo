use axum::{extract::Json, response::IntoResponse, routing::post, Router};
use serde_json::Value;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/alert", post(log_payload));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn log_payload(Json(payload): Json<Value>) -> impl IntoResponse {
    tracing::info!("Received payload: {:?}", payload);
    "Ok"
}
