use crate::models::{AnalyzeResponse, Session};
use crate::store::AppState;
use serde_json::json;

pub async fn analyze_session(state: &AppState, session: &Session) -> anyhow::Result<AnalyzeResponse> {
    let transcript = session
        .transcript
        .clone()
        .unwrap_or_else(|| "".to_string());

    // Fallback heuristic if no API key
    if state.openai_api_key.is_none() {
        let (score, summary, next_step) = heuristic_analyze(&transcript);
        return Ok(AnalyzeResponse {
            intent_score: score,
            summary,
            next_step,
        });
    }

    // OpenAI analysis
    let key = state.openai_api_key.clone().unwrap();
    let model = state.openai_model.clone();

    let system = "You are an expert SDR analyst. Given a call transcript, output JSON only.";
    let user = format!(
        r#"Analyze this outbound SDR call transcript and output a strict JSON object with:
- intent_score: integer 0..100
- summary: short paragraph (max 80 words)
- next_step: one concrete next action

Transcript:
{}"#,
        transcript
    );

    let body = json!({
      "model": model,
      "messages": [
        { "role": "system", "content": system },
        { "role": "user", "content": user }
      ],
      "response_format": { "type": "json_object" }
    });

    let resp = state
        .http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("openai error: {} {}", status, text);
    }

    let v: serde_json::Value = resp.json().await?;
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing content"))?;

    let out: serde_json::Value = serde_json::from_str(content)?;
    let intent_score = out["intent_score"].as_i64().unwrap_or(0) as i32;
    let summary = out["summary"].as_str().unwrap_or("").to_string();
    let next_step = out["next_step"].as_str().unwrap_or("").to_string();

    Ok(AnalyzeResponse {
        intent_score,
        summary,
        next_step,
    })
}

fn heuristic_analyze(transcript: &str) -> (i32, String, String) {
    let t = transcript.to_lowercase();
    let mut score = 20;

    if t.contains("sure") || t.contains("sounds good") || t.contains("send") {
        score += 35;
    }
    if t.contains("meeting") || t.contains("calendar") || t.contains("this week") {
        score += 25;
    }
    if t.contains("not interested") || t.contains("stop") {
        score = 5;
    }

    let summary = if transcript.is_empty() {
        "No transcript yet. Unable to assess intent.".to_string()
    } else {
        "Prospect engaged briefly, confirmed role context, and showed openness to next steps based on the conversation.".to_string()
    };

    let next_step = if score >= 60 {
        "Send a short follow-up with 2 time options and a 1-paragraph value recap tied to the original intent signal.".to_string()
    } else {
        "Ask 1 clarifying question via follow-up (role + current tooling) and offer a low-friction next step.".to_string()
    };

    (score.clamp(0, 100), summary, next_step)
}
