# Valley Voice SDR — Rust MVP

A Voice-First AI SDR prototype built in Rust.

This project demonstrates how **intent signals (e.g. LinkedIn activity)** can be converted into **real-time outbound
voice conversations**, producing:

- Call transcript
- Customer intent score
- Qualification summary
- Concrete next step

The goal is to show a complete pipeline:

This is designed as a lightweight MVP to validate product direction and technical feasibility.

---

## Features

- Rust + Axum async API
- Provider abstraction (Mock or Vapi)
- Real outbound calls via Vapi (optional)
- Local mock voice provider for fast iteration
- Transcript ingestion via webhook
- LLM-based or heuristic call analysis
- Session-based state (in-memory)
- Intent scoring + qualification summary
- Clean separation:
    - routes
    - voice logic
    - voice providers
    - LLM analysis

---

## Architecture

High-level flow:

POST /call
↓
voice.rs
↓
voice_provider.rs (Mock or Vapi)
↓
Outbound Call
↓
Webhook (/webhook/vapi)
↓
Transcript stored
↓
POST /session/:id/analyze
↓
Intent + Summary + Next Step

### Key Files

src/
main.rs — App bootstrap
routes.rs — HTTP endpoints
models.rs — Core data structures
store.rs — AppState + session store
prompt.rs — SDR prompt builder
voice.rs — Call orchestration logic
voice_provider.rs — Vapi + Mock provider
llm.rs — Transcript → SDR analysis

Provider-specific logic lives in `voice_provider.rs`.

Application logic lives in `voice.rs`.

---

## Requirements

- Rust 1.70+
- Cargo
- (Optional) OpenAI API key
- (Optional) Vapi account

---

## Setup

### Clone

```bash
git clone https://github.com/myke-awoniran/relay-server.git
cd relay-server
```

### Environment

Create .env:
cp .env.example .env
Example:
PORT=3000
BASE_URL=http://localhost:3000

# Use mock provider locally

MOCK_VOICE=true

# Optional LLM

OPENAI_API_KEY=
OPENAI_MODEL=gpt-4o-mini

# Only required when MOCK_VOICE=false

VAPI_API_KEY=
VAPI_PHONE_NUMBER_ID=

```Running Locally (Mock Voice)
Default mode uses a MockVoiceProvider.
No external services required.
```

````Start server
cargo run
````

