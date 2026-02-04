use crate::models::Session;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<DashMap<Uuid, Session>>,
    pub http: reqwest::Client,
    pub base_url: String,
    pub mock_voice: bool,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
}

impl AppState {
    pub fn new(base_url: String, mock_voice: bool, openai_api_key: Option<String>, openai_model: String) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            http: reqwest::Client::new(),
            base_url,
            mock_voice,
            openai_api_key,
            openai_model,
        }
    }
}
