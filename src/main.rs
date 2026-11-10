//! RePikaChat — a standalone, local-first Replika-style companion chat.
//!
//! One binary serves the chat interface, the public profile page and the
//! machine-readable files on loopback. All state lives in one JSON file.

mod bot;
mod catalog;
mod hardware;
mod install;
mod llm;
mod model;
mod profile;
mod store;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

use crate::model::*;
use crate::store::Store;

type App = Arc<Mutex<Store>>;
type ApiErr = (StatusCode, Json<serde_json::Value>);

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INDEX_HTML: &str = include_str!("../web/index.html");
const APP_CSS: &str = include_str!("../web/app.css");
const APP_JS: &str = include_str!("../web/app.js");
const PROFILE_CSS: &str = include_str!("../web/profile.css");
const README_MD: &str = include_str!("../README.md");

#[derive(Clone)]
struct Ctx {
    token: String,
    origin: String,
    started_ms: u64,
}

struct Options {
    port: u16,
    open_browser: bool,
}

enum Cli {
    Run(Options),
    Install,
    Uninstall { purge: bool },
    Help,
    Version,
}

#[tokio::main]
async fn main() {
    let cli = match parse_cli() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    match cli {
        Cli::Help => print_help(),
        Cli::Version => println!("RePikaChat {VERSION}"),
        Cli::Install => match install::install() {
            Ok(lines) => {
                println!("\n  RePikaChat is installed.\n");
                for line in lines {
                    println!("  {line}");
                }
                println!("\n  Launch it from the Start Menu, or run the installed RePikaChat.exe.\n");
            }
            Err(e) => {
                eprintln!("Could not install: {e}");
                std::process::exit(1);
            }
        },
        Cli::Uninstall { purge } => match install::uninstall(purge) {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
            }
            Err(e) => {
                eprintln!("Could not uninstall: {e}");
                std::process::exit(1);
            }
        },
        Cli::Run(opts) => run(opts).await,
    }
}

fn print_help() {
    println!(
        "\
RePikaChat {VERSION} — a standalone local AI companion

USAGE
  RePikaChat [--port N] [--no-open] [--data PATH]
  RePikaChat --install
  RePikaChat --uninstall [--purge]

OPTIONS
  --port N        Port to serve on (default 7719, env REPIKACHAT_PORT)
  --no-open       Do not open the browser on start
  --data PATH     Save file or directory (env REPIKACHAT_DATA)
  --install       Install for this user in %LOCALAPPDATA%\\Programs\\RePikaChat
  --uninstall     Remove the app and its Start Menu shortcut
  --purge         With --uninstall, also erase the memory file
  -h, --help      This text
  -V, --version   Print the version

MODEL
  Replies come from the built-in engine unless you configure a local model in
  the app's settings, or set REPIKACHAT_MODEL (and optionally
  REPIKACHAT_MODEL_PROVIDER=ollama|openai, REPIKACHAT_MODEL_ENDPOINT).
  Only loopback endpoints are accepted; nothing is sent off this machine.
"
    );
}

fn parse_cli() -> Result<Cli, String> {
    let mut args = std::env::args().skip(1);
    let env_port = std::env::var("REPIKACHAT_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    let mut port = env_port.unwrap_or(7719);
    let mut open_browser = std::env::var("REPIKACHAT_NO_OPEN").is_err();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--install" => return Ok(Cli::Install),
            "--uninstall" => {
                let purge = args.any(|a| a == "--purge");
                return Ok(Cli::Uninstall { purge });
            }
            "--purge" => return Ok(Cli::Uninstall { purge: true }),
            "--help" | "-h" => return Ok(Cli::Help),
            "--version" | "-V" => return Ok(Cli::Version),
            "--no-open" => open_browser = false,
            "--port" => {
                let v = args.next().ok_or("--port needs a number.")?;
                port = v
                    .parse::<u16>()
                    .map_err(|_| format!("`{v}` is not a port."))?;
            }
            other => {
                if let Some(v) = other.strip_prefix("--port=") {
                    port = v
                        .parse::<u16>()
                        .map_err(|_| format!("`{v}` is not a port."))?;
                } else if let Some(v) = other.strip_prefix("--data=") {
                    std::env::set_var("REPIKACHAT_DATA", v);
                } else {
                    return Err(format!("Unknown option `{other}`. Try --help."));
                }
            }
        }
    }

    Ok(Cli::Run(Options {
        port,
        open_browser,
    }))
}

async fn run(opts: Options) {
    let store = Store::load();
    let save_path = store.path.display().to_string();
    let view = store.view();
    let brain = view.brain;
    let app: App = Arc::new(Mutex::new(store));

    let ctx = Ctx {
        token: make_token(),
        origin: format!("http://127.0.0.1:{}", opts.port),
        started_ms: now_ms(),
    };

    let router = Router::new()
        .route("/", get(index))
        .route("/app.css", get(css))
        .route("/app.js", get(js))
        .route("/favicon.ico", get(favicon))
        .route("/profile", get(profile_page))
        .route("/profile.css", get(profile_css))
        .route("/healthz", get(healthz))
        .route("/robots.txt", get(robots))
        .route("/llms.txt", get(llms))
        .route("/security.txt", get(security))
        .route("/.well-known/security.txt", get(security))
        .route("/openapi.json", get(openapi))
        .route("/docs/README.md", get(docs_readme))
        .route("/api/public", get(api_public))
        .route("/api/catalog", get(api_catalog))
        .route("/api/catalog/use", post(api_catalog_use))
        .route("/api/state", get(api_state))
        .route("/api/chat", post(api_chat))
        .route("/api/companion", post(api_companion))
        .route("/api/settings", post(api_settings))
        .route("/api/model/probe", post(api_probe))
        .route("/api/model/size", post(api_model_size))
        .route("/api/export", get(api_export))
        .route("/api/memory/:id", delete(api_memory_delete))
        .route("/api/history/clear", post(api_history_clear))
        .route("/api/reset", post(api_reset))
        .route("/api/quit", post(api_quit))
        .with_state(app)
        .layer(Extension(ctx.clone()));

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "Could not bind {addr}: {e}\nIs RePikaChat already running on port {}?",
                opts.port
            );
            std::process::exit(1);
        }
    };

    println!();
    println!("  RePikaChat {VERSION} - a local AI companion");
    println!("  -----------------------------------------");
    println!("  Open:      {}/", ctx.origin);
    println!("  Profile:   {}/profile", ctx.origin);
    println!("  Save file: {save_path}");
    if brain.active {
        println!(
            "  Replies:   model · {} via {} at {}",
            brain.model, brain.provider, brain.endpoint
        );
        println!("             (falls back to the built-in engine on any error)");
    } else {
        println!("  Replies:   built-in engine (configure a local model in settings)");
    }
    println!("  Install:   RePikaChat.exe --install");
    println!("  Stop:      Ctrl+C, or the quit button in settings");
    println!();

    if opts.open_browser {
        open_browser(&ctx.origin);
    }

    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("Server error: {e}");
    }
}

/// Local CSRF token for mutating routes. Not a password: it only has to be
/// unguessable by another page in the same browser session.
fn make_token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}{:x}", nanos, std::process::id())
}

fn open_browser(url: &str) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

fn api_err(code: StatusCode, message: &str) -> ApiErr {
    (code, Json(json!({ "error": message })))
}

fn guard(headers: &HeaderMap, ctx: &Ctx) -> Result<(), ApiErr> {
    let ok = headers
        .get("x-repikachat-token")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == ctx.token)
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(api_err(
            StatusCode::FORBIDDEN,
            "Missing or stale session token. Reload the page.",
        ))
    }
}

fn no_store(body: impl IntoResponse, content_type: &'static str) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

async fn index(Extension(ctx): Extension<Ctx>) -> Response {
    let html = INDEX_HTML.replace("__SESSION_TOKEN__", &ctx.token);
    no_store(html, "text/html; charset=utf-8")
}

async fn css() -> Response {
    no_store(APP_CSS, "text/css; charset=utf-8")
}

async fn js() -> Response {
    no_store(APP_JS, "text/javascript; charset=utf-8")
}

async fn profile_css() -> Response {
    no_store(PROFILE_CSS, "text/css; charset=utf-8")
}

async fn favicon() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn docs_readme() -> Response {
    no_store(README_MD, "text/markdown; charset=utf-8")
}

async fn healthz(State(app): State<App>, Extension(ctx): Extension<Ctx>) -> Json<serde_json::Value> {
    let s = app.lock().await;
    let brain = crate::llm::brain_view(&s.studio.settings);
    Json(json!({
        "status": "ok",
        "version": VERSION,
        "uptime_ms": now_ms().saturating_sub(ctx.started_ms),
        "engine": if brain.active { "model" } else { "built-in" },
        "model": brain.model,
        "memories": s.studio.memories.len(),
        "messages": s.studio.messages,
    }))
}

async fn robots() -> Response {
    no_store(profile::robots_txt(), "text/plain; charset=utf-8")
}

async fn llms(State(app): State<App>, Extension(ctx): Extension<Ctx>) -> Response {
    let view = {
        let s = app.lock().await;
        s.view()
    };
    let uptime = now_ms().saturating_sub(ctx.started_ms);
    let profile = profile::build(view.brain, view.data_dir, uptime);
    no_store(profile::llms_txt(&profile), "text/plain; charset=utf-8")
}

async fn security(Extension(ctx): Extension<Ctx>) -> Response {
    no_store(
        profile::security_txt(&ctx.origin),
        "text/plain; charset=utf-8",
    )
}

async fn openapi(Extension(ctx): Extension<Ctx>) -> Response {
    no_store(
        serde_json::to_string_pretty(&profile::openapi(&ctx.origin)).unwrap_or_else(|_| "{}".into()),
        "application/json; charset=utf-8",
    )
}

async fn build_profile(app: App, ctx: &Ctx) -> profile::PublicProfile {
    let view = {
        let s = app.lock().await;
        s.view()
    };
    let uptime = now_ms().saturating_sub(ctx.started_ms);
    profile::build(view.brain, view.data_dir, uptime)
}

async fn profile_page(State(app): State<App>, Extension(ctx): Extension<Ctx>) -> Response {
    let profile = build_profile(app, &ctx).await;
    no_store(profile::render_html(&profile), "text/html; charset=utf-8")
}

async fn api_public(State(app): State<App>, Extension(ctx): Extension<Ctx>) -> Json<profile::PublicProfile> {
    Json(build_profile(app, &ctx).await)
}

async fn api_catalog(State(app): State<App>) -> Json<serde_json::Value> {
    let (endpoint, provider) = {
        let s = app.lock().await;
        runtime_endpoint(&s.studio.settings.endpoint, &s.studio.settings.provider)
    };
    let installed = llm::Client::installed_models(&endpoint, &provider).await;
    Json(catalog::json(installed.as_deref()))
}

/// The local runtime the catalog asks about pulled models: the configured
/// endpoint when it is a model server, otherwise the stock Ollama port, so the
/// "on disk" marks still work with the built-in engine selected.
fn runtime_endpoint(endpoint: &str, provider: &str) -> (String, String) {
    let configured = endpoint.trim();
    if !configured.is_empty() && matches!(provider, "ollama" | "openai") {
        (configured.to_string(), provider.to_string())
    } else {
        ("http://127.0.0.1:11434".to_string(), "ollama".to_string())
    }
}

#[derive(Deserialize)]
struct CatalogUseIn {
    id: String,
}

#[derive(Serialize)]
struct CatalogUseOut {
    note: String,
    state: View,
}

/// Points the reply engine at a catalog entry. Only chat models qualify; the
/// rest are reference entries and say so rather than silently doing nothing.
async fn api_catalog_use(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(body): Json<CatalogUseIn>,
) -> Result<Json<CatalogUseOut>, ApiErr> {
    guard(&headers, &ctx)?;

    let entry = catalog::find(&body.id)
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "Unknown model id."))?;
    if !entry.chat_capable {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "That model generates images, video, audio or embeddings. RePikaChat only writes chat replies, so it cannot drive the conversation.",
        ));
    }

    let (endpoint, provider) = {
        let s = app.lock().await;
        runtime_endpoint(&s.studio.settings.endpoint, &s.studio.settings.provider)
    };
    let on_disk = provider == "ollama"
        && llm::Client::installed_models(&endpoint, &provider)
            .await
            .map(|list| catalog::is_installed(entry.model_tag, &list))
            .unwrap_or(false);

    let mut s = app.lock().await;
    let mut settings = s.studio.settings.clone();
    settings.provider = entry.engine.to_string();
    settings.model = entry.model_tag.to_string();
    if settings.provider == "ollama" && settings.endpoint.trim().is_empty() {
        settings.endpoint = "http://127.0.0.1:11434".to_string();
    }
    settings.sanitise();
    s.studio.settings = settings;
    s.save();

    let note = if on_disk {
        format!(
            "{} is already on disk — the engine is using it now, so there is nothing to download.",
            entry.name
        )
    } else {
        entry.setup_note.replace("{tag}", entry.model_tag)
    };
    Ok(Json(CatalogUseOut {
        note,
        state: s.view(),
    }))
}

async fn api_state(State(app): State<App>) -> Json<View> {
    let s = app.lock().await;
    Json(s.view())
}

#[derive(Deserialize)]
struct ChatIn {
    text: String,
}

#[derive(Serialize)]
struct ChatOut {
    reply: String,
    pose: String,
    learned: Vec<Memory>,
    /// "model" when the words came from the local model, "local" when the
    /// built-in engine wrote them (also used when the model failed).
    source: String,
    model_error: Option<String>,
    state: View,
}

async fn api_chat(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(body): Json<ChatIn>,
) -> Result<Json<ChatOut>, ApiErr> {
    guard(&headers, &ctx)?;

    if body.text.trim().is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "Message is empty."));
    }
    if body.text.chars().count() > 1200 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "That message is long even for me. Try under 1200 characters.",
        ));
    }

    // Step 1: the deterministic half. Extraction, memory, affinity, expression
    // and the fallback reply all happen here, under the lock, before any model.
    let mut pending: Option<(ModelSettings, bot::LlmContext, u64)> = None;
    let out = {
        let mut s = app.lock().await;
        let out = bot::turn(&mut s.studio, &body.text);
        let settings = s.studio.settings.clone();
        if !out.deterministic && settings.provider != "off" && !settings.model.trim().is_empty() {
            let context = bot::llm_context(&s.studio, &body.text, &out.learned);
            let at = s.studio.history.last().map(|m| m.at_ms).unwrap_or(0);
            pending = Some((settings, context, at));
        }
        s.save();
        out
    };

    // Step 2: let the local model write the words. The lock is released while it
    // thinks, and any failure leaves the composed reply untouched.
    let mut reply = out.reply;
    let mut source = "local".to_string();
    let mut model_error = None;

    if let Some((settings, context, bot_at)) = pending {
        match llm::Client::from_settings(&settings) {
            Ok(client) => match client.reply(&context.system, &context.turns).await {
                Ok(text) => {
                    let mut s = app.lock().await;
                    if let Some(last) = s.studio.history.last_mut() {
                        if last.role == "bot" && last.at_ms == bot_at {
                            last.text = text.clone();
                        }
                    }
                    s.save();
                    reply = text;
                    source = "model".to_string();
                }
                Err(e) => {
                    eprintln!("[repikachat] model unavailable, answered locally: {e}");
                    model_error = Some(e);
                }
            },
            Err(e) => model_error = Some(e),
        }
    }

    let state = {
        let s = app.lock().await;
        s.view()
    };

    Ok(Json(ChatOut {
        reply,
        pose: out.pose,
        learned: out.learned,
        source,
        model_error,
        state,
    }))
}

#[derive(Deserialize)]
struct CompanionIn {
    name: String,
    pronouns: Option<String>,
    mode: Option<String>,
    warmth: Option<u8>,
    playfulness: Option<u8>,
    curiosity: Option<u8>,
}

async fn api_companion(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(body): Json<CompanionIn>,
) -> Result<Json<View>, ApiErr> {
    guard(&headers, &ctx)?;

    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "Give your companion a name."));
    }
    if name.chars().count() > 24 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "24 characters is plenty for a name.",
        ));
    }
    let mode = body.mode.unwrap_or_else(|| "friend".to_string());
    if !["friend", "mentor", "romantic"].contains(&mode.as_str()) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Unknown relationship mode."));
    }

    let mut s = app.lock().await;
    {
        let st = &mut s.studio;
        let first_run = !st.onboarded;
        st.companion.name = name;
        if let Some(p) = body.pronouns {
            let p = p.trim().to_string();
            if !p.is_empty() && p.chars().count() <= 20 {
                st.companion.pronouns = p;
            }
        }
        st.companion.mode = mode;
        if let Some(v) = body.warmth {
            st.companion.traits.warmth = v.min(100);
        }
        if let Some(v) = body.playfulness {
            st.companion.traits.playfulness = v.min(100);
        }
        if let Some(v) = body.curiosity {
            st.companion.traits.curiosity = v.min(100);
        }
        st.onboarded = true;

        if first_run && st.history.is_empty() {
            let (text, pose) = bot::greeting(st);
            st.history.push(Msg {
                role: "bot".to_string(),
                text,
                at_ms: now_ms(),
                pose,
            });
        }
    }
    s.save();
    Ok(Json(s.view()))
}

#[derive(Deserialize)]
struct SettingsIn {
    provider: String,
    endpoint: String,
    model: String,
    timeout_ms: Option<u64>,
    temperature: Option<f32>,
}

fn validated_settings(input: SettingsIn, current: &ModelSettings) -> Result<ModelSettings, String> {
    let provider = input.provider.trim().to_ascii_lowercase();
    if !["off", "ollama", "openai"].contains(&provider.as_str()) {
        return Err("Unknown engine. Choose off, ollama or openai.".to_string());
    }
    let mut next = current.clone();
    next.provider = provider;
    next.endpoint = input.endpoint.trim().to_string();
    next.model = input.model.trim().to_string();
    if let Some(t) = input.timeout_ms {
        next.timeout_ms = t;
    }
    if let Some(temp) = input.temperature {
        next.temperature = temp;
    }

    if next.provider != "off" {
        llm::parse_endpoint(&next.endpoint)?;
        if next.model.is_empty() {
            return Err("Set a model name, for example llama3.2.".to_string());
        }
        if next.model.chars().count() > 80 {
            return Err("That model name is too long.".to_string());
        }
    }
    next.sanitise();
    Ok(next)
}

async fn api_settings(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(body): Json<SettingsIn>,
) -> Result<Json<View>, ApiErr> {
    guard(&headers, &ctx)?;
    let mut s = app.lock().await;
    let next = validated_settings(body, &s.studio.settings)
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, &e))?;
    s.studio.settings = next;
    s.save();
    Ok(Json(s.view()))
}

#[derive(Serialize)]
struct ProbeOut {
    ok: bool,
    models: Vec<String>,
    error: Option<String>,
}

async fn api_probe(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(mut body): Json<SettingsIn>,
) -> Result<Json<ProbeOut>, ApiErr> {
    guard(&headers, &ctx)?;
    if body.provider.trim() != "off" && body.model.trim().is_empty() {
        // Listing what a server has on disk does not need a chat model name.
        body.model = "probe".to_string();
    }
    let current = {
        let s = app.lock().await;
        s.studio.settings.clone()
    };
    let settings = match validated_settings(body, &current) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Json(ProbeOut {
                ok: false,
                models: Vec::new(),
                error: Some(e),
            }))
        }
    };
    if settings.provider == "off" {
        return Ok(Json(ProbeOut {
            ok: false,
            models: Vec::new(),
            error: Some("Choose Ollama or an OpenAI-compatible server first.".to_string()),
        }));
    }

    match llm::Client::from_settings(&settings) {
        Ok(client) => match client.probe().await {
            Ok(models) => Ok(Json(ProbeOut {
                ok: true,
                models,
                error: None,
            })),
            Err(e) => Ok(Json(ProbeOut {
                ok: false,
                models: Vec::new(),
                error: Some(e),
            })),
        },
        Err(e) => Ok(Json(ProbeOut {
            ok: false,
            models: Vec::new(),
            error: Some(e),
        })),
    }
}

#[derive(Deserialize)]
struct SizeIn {
    model: String,
}

#[derive(Serialize)]
struct SizeOut {
    model: String,
    estimate_mb: Option<u64>,
    budget_mb: u64,
    fit: String,
    fit_label: String,
    matched_id: Option<&'static str>,
}

/// Sizes any model name against this machine: the curated catalog entry when
/// the tag matches one, otherwise the parameter count in the tag itself.
async fn api_model_size(
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    Json(body): Json<SizeIn>,
) -> Result<Json<SizeOut>, ApiErr> {
    guard(&headers, &ctx)?;
    let model = body.model.trim().to_string();
    if model.chars().count() > 80 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "That model name is too long.",
        ));
    }
    let (estimate, matched_id) = catalog::size_for(&model);
    let budget_mb = crate::hardware::detect().budget_mb();
    let fit = catalog::fit_for(estimate, budget_mb);
    Ok(Json(SizeOut {
        model,
        estimate_mb: estimate,
        budget_mb,
        fit: fit.to_string(),
        fit_label: catalog::fit_label(fit).to_string(),
        matched_id,
    }))
}

async fn api_memory_delete(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<View>, ApiErr> {
    guard(&headers, &ctx)?;
    let mut s = app.lock().await;
    s.studio.memories.retain(|m| m.id != id);
    s.save();
    Ok(Json(s.view()))
}

/// Clears the transcript but keeps everything that makes the companion itself:
/// learned memories, persona, traits, level and model settings.
async fn api_history_clear(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
) -> Result<Json<View>, ApiErr> {
    guard(&headers, &ctx)?;
    let mut s = app.lock().await;
    s.studio.history.clear();
    s.save();
    Ok(Json(s.view()))
}

async fn api_reset(
    State(app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
) -> Result<Json<View>, ApiErr> {
    guard(&headers, &ctx)?;
    let mut s = app.lock().await;
    let settings = s.studio.settings.clone();
    s.studio = Studio::default();
    s.studio.settings = settings;
    s.save();
    Ok(Json(s.view()))
}

async fn api_export(State(app): State<App>) -> Response {
    let body = {
        let s = app.lock().await;
        serde_json::to_string_pretty(&s.studio).unwrap_or_else(|_| "{}".to_string())
    };
    (
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"repikachat-memory.json\"",
            ),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

async fn api_quit(
    State(_app): State<App>,
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiErr> {
    guard(&headers, &ctx)?;
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        std::process::exit(0);
    });
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(provider: &str) -> SettingsIn {
        SettingsIn {
            provider: provider.to_string(),
            endpoint: "http://127.0.0.1:11434".to_string(),
            model: "llama3.2".to_string(),
            timeout_ms: None,
            temperature: None,
        }
    }

    #[test]
    fn refuses_a_remote_model_endpoint() {
        let current = ModelSettings::default();
        let mut input = settings("ollama");
        input.endpoint = "http://10.0.0.5:11434".to_string();
        assert!(validated_settings(input, &current).is_err());
    }

    #[test]
    fn accepts_a_local_model_and_clamps_timeout() {
        let current = ModelSettings::default();
        let mut input = settings("ollama");
        input.timeout_ms = Some(10);
        let next = validated_settings(input, &current).unwrap();
        assert_eq!(next.provider, "ollama");
        assert_eq!(next.timeout_ms, 2_000);
    }

    #[test]
    fn off_needs_no_endpoint_or_model_name() {
        let current = ModelSettings::default();
        let mut input = settings("off");
        input.endpoint = String::new();
        input.model = String::new();
        let next = validated_settings(input, &current).unwrap();
        assert_eq!(next.provider, "off");
    }

    #[test]
    fn a_bad_engine_name_is_rejected() {
        let current = ModelSettings::default();
        assert!(validated_settings(settings("gpt-in-the-cloud"), &current).is_err());
    }
}
