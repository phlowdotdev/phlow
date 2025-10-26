---

sidebar_position: 12
title: OpenAI Module
hide_title: true
---

# OpenAI Module

The OpenAI module provides integration with the OpenAI API for operations such as chat (conversations), completions, embeddings, audio transcription/translation, and image generation/editing. It can be configured via `with` (API key) or using the `OPENAI_API_KEY` environment variable.

## 🚀 Key features

- ✅ Supported actions: `chat`, `completion`, `embeddings`, `audio_transcribe`, `audio_translate`, `image_generate`, `image_edit`
- ✅ Configurable via `with` or `OPENAI_API_KEY` environment variable
- ✅ Simple integration that forwards the request to the OpenAI endpoints and returns the API response as a Phlow `Value`
- ✅ Basic error handling and consistent return as `Value`

## 📋 Module configuration

Example module declaration in a Phlow file:

```phlow
modules:
  - name: openai
    module: openai
    with:
      api_key: "sk-..." # optional - if omitted, OPENAI_API_KEY is used
```

Or via environment variable:

```bash
export OPENAI_API_KEY="sk-..."
```

## 🔧 Parameters (with / input / output)

### Module configuration (`with`)
- `api_key` (string, optional): OpenAI API key. If omitted, the module will attempt to read `OPENAI_API_KEY` from the environment.

### Input (required)
The input must be an object that always defines the `action` property (string). Other fields depend on the chosen action.

Common input fields:
- `action` (string, required): one of the supported actions. Accepted values:
  - `chat` — conversation / chat completions
  - `completion` — classic completions
  - `embeddings` — embeddings generation
  - `audio_transcribe` — audio transcription
  - `audio_translate` — audio translation
  - `image_generate` — image generation from prompt
  - `image_edit` — image editing (requires input image)

Additional fields (dependent on `action`):
- For `chat`:
  - `messages` (array): list of messages in the format `[{"role":"user","content":"..."}, ...]`
  - `model` (string, optional)
- For `completion`:
  - `prompt` (string)
  - `temperature` (number, optional)
- For `embeddings`:
  - `input_text` (string)
- For audio (`audio_transcribe` / `audio_translate`):
  - `audio_base64` (string): audio encoded in base64 (WAV/MP3/etc.)
  - `language` (string, optional): target language code (optional for transcription)
- For images (`image_generate` / `image_edit`):
  - `image_prompt` (string): prompt for image generation
  - `image_base64` (string): base64-encoded image (used for editing)
  - `size` (string, optional): desired image size (e.g. `1024x1024`)

> Note: the module maps the received object to internal types and calls the corresponding OpenAI endpoints. The API response is forwarded as a Phlow `Value` to the calling step.

### Output
The module returns an object (Phlow `Value`) that corresponds to the OpenAI API response for the requested action. In case of errors, the module typically returns a `Value` containing an error message.

General expected shape:
- `success` (boolean) — not always present; the main return is the original API response converted to `Value`.
- `data` / API-specific structure — depends on the action (completions, chat, embeddings, etc.)
- `error` (string) — error message when applicable

## 💻 Usage examples

### 1) Chat (simple example)

```phlow
steps:
  - name: ask_openai
    use: openai
    input:
      action: "chat"
      messages:
        - role: "user"
          content: "Hi, can you summarize what Rust is?"
      model: "gpt-4o-mini"
```

The `ask_openai` step will contain the object returned by the chat API.

### 2) Completion

```phlow
steps:
  - name: completion_example
    use: openai
    input:
      action: "completion"
      prompt: "Write a haiku about programming in Rust"
      temperature: 0.7
```

### 3) Embeddings

```phlow
steps:
  - name: create_embedding
    use: openai
    input:
      action: "embeddings"
      input_text: "Phrases to index and search semantically"
```

### 4) Audio transcription

```phlow
steps:
  - name: transcribe_audio
    use: openai
    input:
      action: "audio_transcribe"
      audio_base64: "<BASE64_ENCODED_AUDIO>"
      language: "en"
```

### 5) Image generation (prompt)

```phlow
steps:
  - name: gen_image
    use: openai
    input:
      action: "image_generate"
      image_prompt: "A futuristic forest at sunset, cyberpunk style"
      size: "1024x1024"
```

## 🔍 Notes and best practices

- Security: never commit API keys to repositories. Prefer environment variables (`OPENAI_API_KEY`) or secret management in your runtime environment.
- Cost: OpenAI model calls may incur costs. Control frequency and prompt size for production use.
- Message size: for `chat` and `completion`, be aware of the model token limits.
- Response format: the module returns the raw API response (converted to `Value`); adapt your pipeline to handle the specific structure returned by OpenAI.

## 🏷️ Tags

- openai
- ai
- nlp
- embeddings
- chat
- image
- audio

---

**Version**: 0.1.0  
**Author**: Phlow Contributors `<dev@phlow.local>`  
**License**: MIT

