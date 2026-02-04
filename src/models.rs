use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequest {
    pub name: String,
    pub company: String,
    pub phone: String,
    // Valley-style context
    pub signal: String,
    pub pain_point: Option<String>,
    pub persona: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponse {
    pub session_id: Uuid,
    pub status: CallStatus,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WebhookStatus {
    COMPLETED,
    FAILED,
    PENDING,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallStatus {
    NotStarted,
    Calling,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub created_at: OffsetDateTime,

    // Prospect + context
    pub name: String,
    pub company: String,
    pub phone: String,
    pub signal: String,
    pub pain_point: Option<String>,
    pub persona: Option<String>,

    // Call state
    pub status: CallStatus,
    pub provider_call_id: Option<String>,

    // Conversation artifacts
    pub prompt: String,
    pub transcript: Option<String>,

    // Analysis
    pub intent_score: Option<i32>, // 0..10
    pub summary: Option<String>,
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWebhookEvent {
    pub provider_call_id: String,

    pub status: WebhookStatus,

    pub transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub intent_score: i32,
    pub summary: String,
    pub next_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSessionView {
    pub session_id: Uuid,
    pub status: CallStatus,
    pub intent_score: Option<i32>,
    pub transcript: Option<String>,
    pub summary: Option<String>,
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProviderWebhook {
    pub provider: String,
    pub event: crate::voice_provider::NormalizedCallEvent,
}
