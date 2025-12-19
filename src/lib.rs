use axum::body::Body;
use axum::response::Response;
use axum::routing::get_service;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<RwLock<Vec<RouteConfig>>>,
    pub logs: Arc<RwLock<Vec<RequestLog>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub id: usize,
    pub path: String,
    pub method: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status: u16,
}

pub fn build_app(state: AppState) -> Router {
    let frontend_dir =
        std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".to_string());
    let static_service =
        get_service(ServeDir::new(frontend_dir.clone())).handle_error(|err| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", err),
            )
        });

    Router::new()
        .route("/api/routes", get(get_routes).post(add_route))
        .route("/api/logs", get(get_logs))
        .nest_service("/", static_service)
        .fallback(spa_index)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_routes(State(state): State<AppState>) -> Json<Vec<RouteConfig>> {
    let routes = state.routes.read().unwrap();
    Json(routes.clone())
}

async fn add_route(
    State(state): State<AppState>,
    Json(payload): Json<RouteConfig>,
) -> impl IntoResponse {
    let mut routes = state.routes.write().unwrap();
    routes.push(payload);
    (StatusCode::CREATED, Json("ok"))
}

async fn get_logs(State(state): State<AppState>) -> Json<Vec<RequestLog>> {
    let logs = state.logs.read().unwrap();
    Json(logs.clone())
}

async fn spa_index() -> impl IntoResponse {
    // For SPA routing: serve index.html for any non-API route
    // This allows React Router to handle client-side routing
    let frontend_dir =
        std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".to_string());
    let index_path = std::path::Path::new(&frontend_dir).join("index.html");

    match std::fs::read_to_string(&index_path) {
        Ok(html) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body(Body::from(html))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"error":"Not Found"}"#))
            .unwrap(),
    }
}

pub fn log_request(state: &AppState, method: String, path: String, status: u16) {
    let log_entry = RequestLog {
        timestamp: Utc::now().to_rfc3339(),
        method,
        path,
        status,
    };

    let mut logs = state.logs.write().unwrap();
    logs.push(log_entry);

    if logs.len() > 1000 {
        let excess = logs.len() - 1000;
        logs.drain(0..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use http::{Method, Request};
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn integration_add_and_get_routes() {
        let state = AppState {
            routes: Arc::new(RwLock::new(vec![])),
            logs: Arc::new(RwLock::new(vec![])),
        };

        let app = build_app(state.clone());

        let payload = RouteConfig {
            id: 99,
            path: "/it".into(),
            method: "GET".into(),
            description: None,
        };

        let body = serde_json::to_string(&payload).unwrap();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/routes")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // now GET
        let req2 = Request::builder()
            .method(Method::GET)
            .uri("/api/routes")
            .body(Body::empty())
            .unwrap();

        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
        let routes: Vec<RouteConfig> = serde_json::from_slice(&body_bytes).unwrap();
        assert!(routes.iter().any(|r| r.id == 99 && r.path == "/it"));
    }
}
