use std::sync::{Arc, RwLock};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bff_mvp::{build_app, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = AppState {
        routes: Arc::new(RwLock::new(vec![])),
        logs: Arc::new(RwLock::new(vec![])),
    };

    let app = build_app(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();

    tracing::info!("Listening on http://{}", bind_addr);
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
