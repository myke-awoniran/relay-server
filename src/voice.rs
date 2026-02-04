use crate::models::CallStatus;
use crate::store::AppState;
use tracing::info;
use uuid::Uuid;

use crate::voice_provider::{NormalizedCallEvent, VapiProvider};

pub async fn start_call_flow(state: AppState, session_id: Uuid) -> anyhow::Result<()> {
    let provider = VapiProvider::from_env()?;

    let session = state
        .sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("session not found"))?
        .clone();

    let first_message = format!(
        "Hi {}, I’m reaching out because {}. Do you have a quick minute?",
        session.name, session.signal
    );

    // This metadata is GOLD: lets you map webhook → session instantly.
    let metadata = serde_json::json!({
        "session_id": session_id.to_string()
    });

    let call_id = provider
        .create_call(
            &state,
            &session.phone,
            first_message,
            session.prompt.clone(),
            metadata,
        )
        .await?;

    if let Some(mut s) = state.sessions.get_mut(&session_id) {
        s.provider_call_id = Some(call_id.clone());
        s.status = CallStatus::Calling;
    }

    info!("started provider call {}", call_id);
    Ok(())
}

pub async fn handle_normalized_event(
    state: AppState,
    ev: NormalizedCallEvent,
) -> anyhow::Result<()> {
    // Prefer session_id from metadata (fast + reliable)
    let session_id = ev
        .metadata
        .get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let session_id = if let Some(id) = session_id {
        id
    } else {
        // fallback: scan provider_call_id
        let mut found = None;
        for s in state.sessions.iter() {
            if s.provider_call_id.as_deref() == Some(ev.provider_call_id.as_str()) {
                found = Some(*s.key());
                break;
            }
        }
        found.ok_or_else(|| anyhow::anyhow!("unknown provider call id"))?
    };

    if let Some(mut s) = state.sessions.get_mut(&session_id) {
        let st = ev.status.to_lowercase();
        if st.contains("end") || st.contains("complete") {
            s.status = CallStatus::Completed;
        } else if st.contains("fail") {
            s.status = CallStatus::Failed;
        }

        if let Some(t) = ev.transcript {
            s.transcript = Some(t);
        }
    }

    Ok(())
}
