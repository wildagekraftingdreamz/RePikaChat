# RePikaChat

[![CI](https://github.com/wildagekraftingdreamz/RePikaChat/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wildagekraftingdreamz/RePikaChat/actions/workflows/ci.yml)

A standalone AI companion chat in Rust: one consumer, one bot, one local JSON
file, and a chat engine that learns from what you tell it.

RePikaChat takes the conversation system from [RePika](../RePika) — fact
extraction, auditable memory, affinity, intents, persona composition — and
builds a product around the chat alone. No wardrobe, no 3D stage, no account,
no cloud. It serves its own interface on loopback and, optionally, hands the
sentences to a local language model while keeping the memory and guardrails
in Rust.

```bash
cd D:\SoulApps\RepikaChat
cargo run --release
# open http://127.0.0.1:7719/
```

## Install

Windows, per-user, no administrator rights:

```powershell
cargo build --release
.\target\release\RePikaChat.exe --install
```

That copies the binary to `%LOCALAPPDATA%\Programs\RePikaChat`, adds a Start
Menu shortcut, and leaves `Uninstall.cmd` beside the executable.
`RePikaChat.exe --uninstall` removes the app and shortcut but keeps your
memories; add `--purge` to erase them too.

A polished `setup.exe` is built with Inno Setup when it is installed:

```powershell
powershell -ExecutionPolicy Bypass -File .\installer\build-installer.ps1
```

Without Inno Setup the same script produces `dist\RePikaChat-<version>-portable.zip`
plus an `Install.cmd` inside it, so there is always a working installer path.

## The chat that learns

Every accepted message goes through four local steps (`src/bot.rs`):

1. **Extraction** — pulls structured facts out of natural phrasing: `my name is …`,
   `I love …`, `I hate …`, `I work as …`, `I live in …`, `my goal is …`, `I feel …`,
   `my dog/cat/…`, `remember that …`. Facts are found mid-sentence too, so
   "long day, and I love rainy nights" teaches the same thing as the bare phrase.
2. **Memory** — each fact is stored with a confidence score, a timestamp and the
   sentence it came from. Repeats raise confidence instead of duplicating. Name
   and current feeling are single-slot: a person has one of each.
3. **Mood and affinity** — a small sentiment lexicon nudges affinity (0–100),
   which picks one of four moods and changes how the companion speaks.
4. **Composition** — intent detection (greeting, affection, sadness, anxiety,
   joy, boredom, question, recall, thanks, apology, farewell) picks a reply,
   flavoured by the warmth, playfulness and curiosity you set at setup, then
   decorated with facts it holds.

`what do you know about me` prints the memory back deterministically.
`forget <word>` drops one fact. The memory panel lists every fact with its
confidence and a delete button, because a companion that remembers you should
also be auditable.

Messages earn experience and levels (90 XP for level 2, +45 per level).

## Local models (optional)

The built-in engine always answers: instant, free and predictable. To improve
the *words* without moving any data off the machine, point RePikaChat at a local
server in **Settings → Reply engine**:

| Server | Engine | Endpoint | Models |
| --- | --- | --- | --- |
| [Ollama](https://ollama.com) | `ollama` | `http://127.0.0.1:11434` | `ollama pull llama3.2` |
| llama.cpp server | `openai` | `http://127.0.0.1:8080` | any loaded GGUF |
| LM Studio | `openai` | `http://127.0.0.1:1234` | any loaded model |
| vLLM / LocalAI / text-generation-webui | `openai` | their loopback port | any served model |

The settings dialog can test the connection and list the models the server
reports. Environment overrides for first run:

| Variable | Default | Meaning |
| --- | --- | --- |
| `REPIKACHAT_MODEL` | _(unset)_ | Model name; enables the bridge |
| `REPIKACHAT_MODEL_PROVIDER` | `ollama` when a model is set | `ollama` or `openai` |
| `REPIKACHAT_MODEL_ENDPOINT` | `http://127.0.0.1:11434` | Base URL, loopback only |
| `REPIKACHAT_PORT` | `7719` | Listen port |
| `REPIKACHAT_DATA` | `%LOCALAPPDATA%\RePikaChat` | Save file or directory |
| `REPIKACHAT_NO_OPEN` | _(unset)_ | Do not open a browser on start |

How a turn flows (`api_chat` in `src/main.rs`): `bot::turn` extracts facts,
updates memory/affinity and composes a fallback; `bot::llm_context` builds the
prompt from persona, mood, held memories, newly learned facts and the last 14
messages; the model writes one reply; on success it replaces the composed text.
On *any* failure — daemon down, missing model, HTTP error, timeout, empty output
— the composed reply stands and the reason is reported in the UI.

`what do you know about me`, `forget …` and crisis replies never reach a model.

## Open generative AI selection

**Open models** in the chat header opens a curated catalog of open-weight models
with their lab, parameter count, licence, local runtime and a link to the model
card. The catalog shape — one central model list, a category per entry, a
selector that switches the active model — is inspired by
[Anil-matcha/Open-Generative-AI](https://github.com/anil-matcha/open-generative-ai)
(MIT). Only downloadable open-weight models are listed; that project's hosted
model APIs are deliberately out of scope because RePikaChat never calls a
hosted service.

**Sizing.** Every entry carries a rough memory figure — 4-bit weights for chat,
the smallest common local checkpoint otherwise — and the catalog compares it
with what this machine can actually give a model: system RAM minus an OS
reserve, or dedicated VRAM minus driver headroom when a GPU is present. The
panel shows the detected hardware, badges each model "fits comfortably", "tight
fit" or "needs more memory", offers a **Fits here** filter, and recommends the
largest chat model that fits comfortably. VRAM is read from the display-adapter
registry (NVIDIA, AMD or Intel), falling back to `nvidia-smi`. Detection is
read-only, done once per process, never written to disk and never sent
anywhere.

Each chat entry is also marked **on disk** or **needs pull**: the catalog asks
the local Ollama daemon (`GET /api/tags`, loopback only, sub-second timeout)
which tags are already there, and the action reads **Use for chat** instead of
**Pull & use** when nothing would be downloaded. If the daemon is not running
the catalog simply makes no claim about what is installed. The reply-engine
settings use the same probe: opening Settings lists those tags as clickable
chips beside the model field, so a tag never has to be typed from memory. The
chosen name is also sized against this machine (`POST /api/model/size`), from
the catalog entry when it has one and otherwise from the parameter count in the
tag, so a model too big for the available RAM and VRAM is flagged in Settings
before it is pulled.

Categories are chat, image, video, audio and embedding. **Use for chat** is only
enabled for chat models: it sets the reply engine to Ollama and the model tag in
one action, and tells you the `ollama pull` command to run first. Image, video,
audio and embedding entries are labelled "not a chat engine" instead of
offering a button that cannot work.

The same list is available as JSON at `/api/catalog`, and
`POST /api/catalog/use {id}` performs the switch programmatically (session token
required).

## Public profile page

`/profile` is a server-rendered product page with overview, what-it-does,
support/contact, security notes, docs and an API table. It is generated from the
running build, so facts it does not have are rendered as **Not published**
instead of being invented — see the "Fields without a published value" block.

Machine-readable companions, all actually served:

| Path | Type |
| --- | --- |
| `/api/public` | JSON profile |
| `/openapi.json` | OpenAPI 3.1 |
| `/llms.txt` | text/plain |
| `/security.txt`, `/.well-known/security.txt` | RFC 9116-style |
| `/robots.txt` | text/plain |
| `/healthz` | JSON health |
| `/docs/README.md` | this document as Markdown |

## API

Loopback only. JSON in, JSON out, errors as `{"error":"…"}`. Every POST and
DELETE requires the per-run `x-repikachat-token` header, which is embedded in
the served page.

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/api/state` | Companion, memories, history, settings |
| POST | `/api/chat` | `{text}` → reply, learned facts, engine used, new state |
| POST | `/api/companion` | name, pronouns, mode, traits (also completes setup) |
| POST | `/api/settings` | provider, endpoint, model, timeout, temperature |
| POST | `/api/model/probe` | List models from the configured local server |
| DELETE | `/api/memory/:id` | Drop one memory |
| GET | `/api/export` | Download the whole save file |
| POST | `/api/history/clear` | Clear the transcript (keeps memories, persona and settings) |
| POST | `/api/reset` | Erase memories and history (keeps model settings) |
| POST | `/api/quit` | Stop the process |
| GET | `/api/public` | Public profile JSON |

## Layout

```
src/main.rs     axum server, routes, session token, chat orchestration
src/model.rs    state, persona, settings, view types
src/store.rs    JSON persistence (atomic write), view builder
src/bot.rs      extraction, memory, affinity, intent, composition, prompt
src/llm.rs      Ollama + OpenAI-compatible loopback bridge
src/profile.rs  public profile page, OpenAPI, llms.txt, security.txt
src/install.rs  per-user Windows install/uninstall
web/            index.html, app.css, app.js, profile.css (no build step, no CDN)
installer/      Inno Setup script and build script
scripts/        smoke-test.mjs (end-to-end checks against a live instance)
```

## Deliberate limits

- **Single player.** One consumer, one bot, no accounts. Data lives in one JSON
  file; `--purge` and the erase button are the delete paths.
- **Small model by default.** Replies are composed from intent + memory +
  persona. With a local model the sentences improve, but grounding is limited to
  extracted memory and the prompt forbids inventing facts.
- **No hosted service.** There is no remote API, no telemetry and no sync. The
  installer is a local desktop install.
- **No crisis service.** The app answers explicit self-harm messages with a
  short, fixed reply pointing to local emergency numbers and crisis lines. It is
  not a substitute for professional help.
- **Memory is capped.** 140 facts, 12 topics and 300 transcript messages, so the
  save file cannot grow without bound.

## Checks

```bash
cargo test
node scripts/smoke-test.mjs        # against a running instance on port 7719
```

MIT licensed. Replika is a product of Luka, Inc.; this project is independent.
