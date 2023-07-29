
use std::sync::Arc;
use std::collections::HashMap;

use axum::{
    routing,
    Router,
    response::{Response, IntoResponse},
    extract::{Path, State},
    http::status::StatusCode,
};

struct AppState {
    data_server: String,
}

#[tokio::main]
async fn main() {
    let state: Arc<AppState> = Arc::new(AppState{
        data_server: "http://127.0.0.1:9000".to_string(),
    });

    let app = Router::new()
        .route("/api/health", routing::get(api_health))
        .route("/api/version", routing::get(|| async { "0.1.0" }))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:8999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn api_health(State(state): State<Arc<AppState>>) -> Response {
    let query = format!("{}/api/health", state.data_server);

    let Ok(resp) = reqwest::get(&query).await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "BACKEND: MISSING").into_response();
    };
    if resp.status() != StatusCode::OK {
        return (StatusCode::SERVICE_UNAVAILABLE, "BACKEND: UNAVAILABLE").into_response();
    }
    let Ok(resp) = resp.bytes().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "BACKEND: COMM ERROR").into_response();
    };
    if resp.as_ref() != b"OK" {
        return (StatusCode::SERVICE_UNAVAILABLE, "BACKEND: UNAVAILABLE").into_response();
    }

    (StatusCode::OK, "OK").into_response()
}
