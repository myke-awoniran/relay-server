use crate::models::CallRequest;

pub fn build_sdr_prompt(req: &CallRequest) -> String {
    let persona = req
        .persona
        .clone()
        .unwrap_or_else(|| "Unknown persona".to_string());
    let pain = req
        .pain_point
        .clone()
        .unwrap_or_else(|| "Not specified".to_string());

    format!(
        r#"You are a Voice-First AI SDR calling {name} at {company}.

Context (from Valley intent signals):
- Signal: {signal}
- Persona: {persona}
- Likely pain point: {pain}

Your goal:
1) Confirm you’re speaking to the right person and their role.
2) Qualify interest and urgency.
3) If qualified, propose a short meeting and secure a verbal commitment.

Style:
- Crisp, friendly, confident.
- Ask short questions.
- If they object, handle objections calmly.
- Do not be overly pushy.
- End with a clear next step.

At the end of the call, ensure you have enough info to produce:
- intent_score (0-100)
- qualification summary (2-5 bullets)
- next_step (specific)
"#,
        name = req.name,
        company = req.company,
        signal = req.signal,
        persona = persona,
        pain = pain
    )
}
