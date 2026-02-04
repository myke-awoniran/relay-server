use crate::voice_provider::{VapiProvider, VapiWebhookBody};
use crate::{
    llm,
    models::{
        AnalyzeResponse, CallRequest, CallResponse, CallStatus, Session, UiSessionView,
        VoiceWebhookEvent,
    },
    prompt,
    store::AppState,
    voice,
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use time::OffsetDateTime;
use uuid::Uuid;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/call", post(start_call))
        // .route("/webhook/voice", post(voice_webhook))
        .route("/session/:id", get(get_session))
        .route("/session/:id/analyze", post(analyze))
        .route("/webhook/vapi", post(vapi_webhook))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn home() -> &'static str {
    "Your AI SDR that turns LinkedIn intent signals into real-time sales calls is alive!"
}

async fn vapi_webhook(
    State(state): State<AppState>,
    Json(body): Json<VapiWebhookBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ev = VapiProvider::normalize_webhook(body);

    crate::voice::handle_normalized_event(state, ev)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn start_call(
    State(state): State<AppState>,
    Json(req): Json<CallRequest>,
) -> Result<Json<CallResponse>, (StatusCode, String)> {
    let id = Uuid::new_v4();
    let p = prompt::build_sdr_prompt(&req);

    let session = Session {
        id,
        created_at: OffsetDateTime::now_utc(),

        name: req.name,
        company: req.company,
        phone: req.phone,
        signal: req.signal,
        pain_point: req.pain_point,
        persona: req.persona,

        status: CallStatus::NotStarted,
        provider_call_id: None,

        prompt: p,
        transcript: None,

        intent_score: None,
        summary: None,
        next_step: None,
    };

    state.sessions.insert(id, session);

    // Start async call flow (mock or provider)
    let st = state.clone();
    tokio::spawn(async move {
        let _ = voice::start_call_flow(st, id).await;
    });

    Ok(Json(CallResponse {
        session_id: id,
        status: CallStatus::Calling,
        message: "Call started (or queued)".to_string(),
    }))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UiSessionView>, (StatusCode, String)> {
    let s = state
        .sessions
        .get(&id)
        .ok_or((StatusCode::NOT_FOUND, "session not found".to_string()))?;

    Ok(Json(UiSessionView {
        session_id: s.id,
        status: s.status.clone(),
        intent_score: s.intent_score,
        transcript: s.transcript.clone(),
        summary: s.summary.clone(),
        next_step: s.next_step.clone(),
    }))
}

async fn analyze(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    let session = state
        .sessions
        .get(&id)
        .ok_or((StatusCode::NOT_FOUND, "session not found".to_string()))?
        .clone();

    let out = llm::analyze_session(&state, &session)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // persist
    if let Some(mut s) = state.sessions.get_mut(&id) {
        s.intent_score = Some(out.intent_score);
        s.summary = Some(out.summary.clone());
        s.next_step = Some(out.next_step.clone());
    }

    Ok(Json(out))
}
