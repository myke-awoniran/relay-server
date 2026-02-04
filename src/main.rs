mod llm;
mod models;
mod prompt;
mod routes;
mod store;
mod voice;
mod voice_provider;

use dotenvy::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;

use store::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    let mock_voice = env::var("MOCK_VOICE")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    let openai_api_key = env::var("OPENAI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

    let state = AppState::new(base_url, mock_voice, openai_api_key, openai_model);
    let app = routes::router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
