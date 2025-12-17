use axum::{
    Router,
    extract::{Request, State},
    response::{Response, IntoResponse},
    routing::{get, post},
    http::{StatusCode, Method, HeaderMap},
    Json,
};
use bytes::Bytes;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Shared application state
#[derive(Clone)]
struct AppState {
    routes: Arc<RwLock<Vec<RouteConfig>>>,
    logs: Arc<RwLock<Vec<RequestLog>>>,
    http_client: reqwest::Client,
    target_bff_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RouteConfig {
    id: usize,
    path: String,
    method: String,
    mode: RouteMode,
    target_path: Option<String>,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RouteMode {
    Proxy,
    Handled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestLog {
    timestamp: String,
    method: String,
    path: String,
    status: u16,
    latency_ms: u64,
    request_body: Option<String>,
    response_body: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let target_bff_url = std::env::var("TARGET_BFF_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let state = AppState {
        routes: Arc::new(RwLock::new(vec![])),
        logs: Arc::new(RwLock::new(vec![])),
        http_client: reqwest::Client::new(),
        target_bff_url,
    };

    let app = Router::new()
        .route("/api/routes", get(get_routes).post(update_route))
        .route("/api/logs", get(get_logs))
        .fallback(proxy_handler)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    
    tracing::info!("BFF MVP listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_routes(State(state): State<AppState>) -> Json<Vec<RouteConfig>> {
    let routes = state.routes.read().unwrap();
    Json(routes.clone())
}

async fn update_route(
    State(state): State<AppState>,
    Json(payload): Json<RouteConfig>,
) -> impl IntoResponse {
    let mut routes = state.routes.write().unwrap();
    
    if let Some(route) = routes.iter_mut().find(|r| r.id == payload.id) {
        *route = payload;
        (StatusCode::OK, Json("Route updated"))
    } else {
        routes.push(payload);
        (StatusCode::CREATED, Json("Route created"))
    }
}

async fn get_logs(State(state): State<AppState>) -> Json<Vec<RequestLog>> {
    let logs = state.logs.read().unwrap();
    Json(logs.clone())
}

async fn proxy_handler(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let headers = req.headers().clone();
    
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default();
    let request_body = String::from_utf8(body_bytes.to_vec()).ok();

    let target_url = format!("{}{}", state.target_bff_url, path);
    
    match forward_request(&state.http_client, &method, &target_url, &headers, body_bytes).await {
        Ok(response) => {
            let status = response.status();
            let response_headers = response.headers().clone();
            let response_body_bytes = response.bytes().await.unwrap_or_default();
            let response_body = String::from_utf8(response_body_bytes.to_vec()).ok();

            log_request(
                &state,
                method.as_str().to_string(),
                path,
                status.as_u16(),
                start.elapsed().as_millis() as u64,
                request_body,
                response_body.clone(),
            );

            let mut builder = Response::builder().status(status);
            for (key, value) in response_headers.iter() {
                builder = builder.header(key, value);
            }
            builder.body(response_body_bytes.into()).unwrap()
        }
        Err(err) => {
            tracing::error!("Proxy error: {:?}", err);
            log_request(
                &state,
                method.as_str().to_string(),
                path,
                500,
                start.elapsed().as_millis() as u64,
                request_body,
                Some(format!("Proxy error: {}", err)),
            );
            
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("Proxy error: {}", err).into())
                .unwrap()
        }
    }
}

async fn forward_request(
    client: &reqwest::Client,
    method: &Method,
    url: &str,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut req = client.request(method.clone(), url);
    
    for (key, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            req = req.header(key.as_str(), val_str);
        }
    }
    
    if !body.is_empty() {
        req = req.body(body);
    }
    
    req.send().await
}

fn log_request(
    state: &AppState,
    method: String,
    path: String,
    status: u16,
    latency_ms: u64,
    request_body: Option<String>,
    response_body: Option<String>,
) {
    let log_entry = RequestLog {
        timestamp: Utc::now().to_rfc3339(),
        method,
        path,
        status,
        latency_ms,
        request_body,
        response_body,
    };
    
    let mut logs = state.logs.write().unwrap();
    logs.push(log_entry);
    
    // Keep only last 1000 logs
    if logs.len() > 1000 {
        logs.drain(0..logs.len() - 1000);
    }
}
