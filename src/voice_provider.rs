use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::store::AppState;

#[derive(Clone)]
pub struct VapiProvider {
    api_key: String,
    phone_number_id: String,
}

impl VapiProvider {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key =
            std::env::var("VAPI_API_KEY").map_err(|_| anyhow::anyhow!("VAPI_API_KEY missing"))?;
        let phone_number_id = std::env::var("VAPI_PHONE_NUMBER_ID")
            .map_err(|_| anyhow::anyhow!("VAPI_PHONE_NUMBER_ID missing"))?;
        Ok(Self {
            api_key,
            phone_number_id,
        })
    }

    /// Create an outbound call on Vapi.
    /// Returns provider call_id.
    pub async fn create_call(
        &self,
        state: &AppState,
        to_phone: &str,
        first_message: String,
        system_prompt: String,
        metadata: serde_json::Value,
    ) -> anyhow::Result<String> {
        let webhook_url = format!("{}/webhook/vapi", state.base_url);

        // Note: voice/provider settings can be changed later.
        let payload = json!({
            "phoneNumberId": self.phone_number_id,
            "customer": { "number": to_phone },

            // Attach metadata so webhook can map the call back to your session_id, etc.
            "metadata": metadata,

            "assistant": {
                "firstMessage": first_message,
                "systemPrompt": system_prompt,

                // You can switch these easily:
                "model": {
                    "provider": "openai",
                    "model": "gpt-4o-mini"
                },
                "voice": {
                    "provider": "11labs",
                    "voiceId": "Zayd"
                }
            },

            "webhookUrl": webhook_url
        });

        let resp = state
            .http
            .post("https://api.vapi.ai/call")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("vapi create_call error: {} {}", status, body);
        }

        let v: serde_json::Value = resp.json().await?;
        let call_id = v["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Vapi response missing id"))?
            .to_string();

        Ok(call_id)
    }

    /// Normalize Vapi webhook payload into a simple, provider-agnostic event.
    pub fn normalize_webhook(body: VapiWebhookBody) -> NormalizedCallEvent {
        // Vapi sends different shapes depending on event.
        // We'll extract: call_id, status, transcript (if present), and metadata (if present).
        let call_id = body.call_id.clone();

        // Status: prefer explicit, else infer from event type.
        let status = if let Some(s) = body.status.clone() {
            s
        } else if let Some(t) = body.event.clone() {
            t
        } else {
            "unknown".to_string()
        };

        // Transcript can be in different places; accept whichever exists.
        let transcript = body
            .transcript
            .clone()
            .or_else(|| body.call.as_ref().and_then(|c| c.transcript.clone()))
            .or_else(|| body.call.as_ref().and_then(|c| c.summary.clone())); // fallback

        let metadata = body
            .metadata
            .clone()
            .or_else(|| body.call.as_ref().and_then(|c| c.metadata.clone()))
            .unwrap_or_else(|| json!({}));

        NormalizedCallEvent {
            provider: "vapi".to_string(),
            provider_call_id: call_id,
            status,
            transcript,
            metadata,
        }
    }
}

/// Provider-agnostic event you can use everywhere in your app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCallEvent {
    pub provider: String,
    pub provider_call_id: String,
    pub status: String,
    pub transcript: Option<String>,
    pub metadata: serde_json::Value,
}

/// A permissive Vapi webhook body.
/// Vapi’s webhook can vary by event type; we keep it flexible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapiWebhookBody {
    #[serde(rename = "callId")]
    pub call_id: String,

    // Sometimes present
    #[serde(default)]
    pub status: Option<String>,

    // Sometimes Vapi uses `event`
    #[serde(default)]
    pub event: Option<String>,

    // Sometimes top-level transcript exists
    #[serde(default)]
    pub transcript: Option<String>,

    // Sometimes metadata exists top-level
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,

    // Sometimes call object exists
    #[serde(default)]
    pub call: Option<VapiCallObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapiCallObject {
    #[serde(default)]
    pub transcript: Option<String>,

    #[serde(default)]
    pub summary: Option<String>,

    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}


