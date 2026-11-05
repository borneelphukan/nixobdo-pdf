# 08 — AI and LLM

This document describes the AI summarization and chatbot features, including
endpoint normalization, preset models, request format, and session management.

## 1. LLM Configuration

### Settings file

- LLM settings live in `dirs::config_dir()/nixobdo-pdf/settings.json`.
- Fields: `llm_api_key` (plaintext), `llm_model`, `llm_endpoint_url`,
  `llm_configured` (bool), `ai_chat_sessions` (Vec<AiChatSession>),
  `ai_active_session_id` (Option<String>), plus recent files.

### Endpoint normalization

The effective API endpoint is determined at startup (in `src/app/mod.rs::default`)

| Condition | Effective endpoint |
|---|---|
| API key **non‑empty** AND endpoint is `localhost`, `127.0.0.1`, or **empty** | `https://api.openai.com/v1` |
| Otherwise | The endpoint string as‑provided by the user |

This means a first‑time user who leaves the endpoint empty and provides an API key
will automatically point at OpenAI's production API.

### Preset models

| Preset display name | Model ID |
|---|---|
| Llama 3.2 3B | `llama-3.2-3b` |
| Llama 3.1 8B | `llama-3.1-8b` |
| Mistral 7B | `mistral-7b` |
| Gemma 2 9B | `gemma-2-9b` |
| Phi‑3 Mini | `phi-3-mini` |
| Qwen 2.5 7B | `qwen-2.5-7b` |
| Qwen 2.5 14B | `qwen-2.5-14b` |
| DeepSeek‑V2 | `deepseek-v2` |
| **Custom** | free‑form string |

The preset list is defined in `src/ui/dialogs/llm_settings.rs::LLM_PRESETS` and
used by `init_llm_preset_from_model()` to map a stored model ID back to a preset
index.

### Custom model

If the user types a model ID not in the preset list, `llm_is_custom_model` is set
to `true` and the raw string is stored in `llm_custom_model`. The preset index
drops to `LLM_PRESETS.len()` (the "Custom" slot).

## 2. AI Request Format

All AI calls are POST requests to the normalized endpoint.

- **URL:** `POST <effective_endpoint>/chat/completions`
- **Headers:** `Authorization: Bearer <api_key>`
- **Body (JSON):**
  ```json
  {
    "model": "<model_id>",
    "temperature": 0.3,
    "max_tokens": 1500,
    "stream": false,
    "messages": [...]
  }
  ```
- The `messages` array depends on the context:
  - **Summary:** system prompt "You are a helpful assistant…" + user prompt
    "Please summarize the following document:\n\n<full_text>".
  - **Chatbot:** per‑session message history; a system prompt is included in the
    initial message.

## 3. AI Summary (stand‑alone, not in chatbot)

- **Trigger:** `AI → Summarize with AI` menu item (or via toolbar if AI is
  configured).
- **Flow:**
  1. Gathers all page text: `tab.page_texts.join("\n")`.
  2. Sends `PdfWorkerTask::AiSummarize { is_chatbot: false, messages, endpoint_url, model, api_key, ctx }`.
  3. Worker sends the POST request via `ureq` (blocking on the worker thread).
  4. On success: `AiSummaryResult { is_chatbot: false, success: true, text }`
     → `ai_summary_full_text` stored, `ai_summary_text` cleared,
     `ai_summary_display_len = 0`, toast "Summary ready".
  5. On failure: `AiSummaryResult { is_chatbot: false, success: false, error }`
     → `ai_summary_error` stored, shows error toast.
- The UI shows a loading state (`ai_summary_loading = true`) while the worker
  handles the HTTP request.
- **Error handling:** If the API key is invalid or the endpoint is unreachable,
  the error string is displayed in a toast.

## 4. AI Chatbot Panel

- **Toggle:** `AI → Summarize with AI` (creates a new session) or the AI panel
  button in the toolbar.
- **Session lifecycle:**
  1. A new `AiChatSession` is created with a timestamp‑based ID and a name like
     "Summary – HH:MM".
  2. The session is pushed onto `ai_chat_sessions`; `ai_active_session_id` is set.
  3. User types a message and presses Enter; the message is sent to the LLM.
  4. On successful reply, the assistant's message is appended to the session's
     `messages` vector; `save_settings()` persists the chat history.
  5. On error, `ai_chat_error` is set and displayed in the UI.
- **UI state:** `ai_chatbot_open`, `ai_chat_loading`, `ai_chat_error`,
  `ai_chat_input`, `ai_chat_display_len`, `ai_chat_start_time`.
- **No LLM configured:** shows "No LLM Service Available" with "Open Settings"
  button.
- **No document open:** shows "No Document Opened" prompt.

## 5. ChatMessage structure

```rust
pub struct ChatMessage {
    pub role: String,      // "system", "user", "assistant"
    pub content: String,
}
```

Sessions persist across app launches via `settings.json` (serialized with
serde). The `ai_chat_sessions` Vec and `ai_active_session_id` are part of the
settings struct in `src/app/state.rs::save_settings()`.