//! The public profile page and the machine-readable files behind it.
//!
//! Everything here is generated from facts this build actually knows. Fields
//! without a published value are represented as `None` and rendered as
//! "Not published" instead of being invented, so the page never looks broken.

use serde::Serialize;

use crate::llm::BrainView;

pub const NAME: &str = "RePikaChat";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SUMMARY: &str = "RePikaChat is a standalone AI companion that runs on your own computer. \
It holds one conversation with one person, learns the facts you choose to share — your name, what you \
love, what you do, how you feel — and keeps them in a single local file you can read, export and erase. \
Replies come from a small built-in engine, or from a local model you point it at, with no account and no cloud.";

/// A real, served file. `available` is false only when the build genuinely does
/// not publish it; the page shows that honestly instead of linking to a 404.
#[derive(Serialize, Clone)]
pub struct DocLink {
    pub title: &'static str,
    pub path: &'static str,
    pub kind: &'static str,
    pub description: &'static str,
    pub available: bool,
}

#[derive(Serialize)]
pub struct Endpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    /// Mutating routes require the per-run `x-repikachat-token` header.
    pub token: bool,
}

#[derive(Serialize)]
pub struct Gap {
    pub label: &'static str,
    pub detail: &'static str,
}

#[derive(Serialize)]
pub struct PublicProfile {
    pub name: &'static str,
    pub version: &'static str,
    pub tagline: &'static str,
    pub summary: &'static str,
    pub product_type: &'static str,
    pub runtime: &'static str,
    pub storage: &'static str,
    pub network: &'static str,
    pub pricing: &'static str,
    pub license: Option<&'static str>,
    pub support_email: Option<&'static str>,
    pub security_contact: Option<&'static str>,
    pub docs: Vec<DocLink>,
    pub gaps: Vec<Gap>,
    pub endpoints: Vec<Endpoint>,
    pub model: BrainView,
    pub data_dir: String,
    pub uptime_ms: u64,
    pub generated_at_ms: u64,
}

pub fn endpoints() -> Vec<Endpoint> {
    vec![
        Endpoint { method: "GET", path: "/api/public", summary: "This profile as JSON.", token: false },
        Endpoint { method: "GET", path: "/api/catalog", summary: "Curated open generative AI models with licence, runtime and chat support.", token: false },
        Endpoint { method: "POST", path: "/api/catalog/use", summary: "Point the reply engine at a chat-capable catalog model.", token: true },
        Endpoint { method: "GET", path: "/api/state", summary: "Companion, memories, history and settings.", token: false },
        Endpoint { method: "POST", path: "/api/chat", summary: "Send one message; returns the reply, what was learned and the new state.", token: true },
        Endpoint { method: "POST", path: "/api/companion", summary: "Create or edit the companion's name, pronouns, mode and traits.", token: true },
        Endpoint { method: "POST", path: "/api/settings", summary: "Choose the reply engine and local model endpoint.", token: true },
        Endpoint { method: "POST", path: "/api/model/probe", summary: "Ask a local model server which models it serves.", token: true },
        Endpoint { method: "POST", path: "/api/model/size", summary: "Estimate a model's memory need against this machine's RAM and VRAM.", token: true },
        Endpoint { method: "DELETE", path: "/api/memory/:id", summary: "Forget exactly one stored fact.", token: true },
        Endpoint { method: "GET", path: "/api/export", summary: "Download the entire save file.", token: false },
        Endpoint { method: "POST", path: "/api/reset", summary: "Erase memories and history, keeping model settings.", token: true },
        Endpoint { method: "POST", path: "/api/quit", summary: "Stop this RePikaChat process.", token: true },
        Endpoint { method: "GET", path: "/healthz", summary: "Liveness and engine status as JSON.", token: false },
    ]
}

pub fn docs() -> Vec<DocLink> {
    vec![
        DocLink {
            title: "Public profile (JSON)",
            path: "/api/public",
            kind: "JSON",
            description: "The same facts this page renders, for tooling and directories.",
            available: true,
        },
        DocLink {
            title: "Open model catalog",
            path: "/api/catalog",
            kind: "JSON",
            description: "Open-weight chat, image, video, audio and embedding models with licence, runtime and an honest chat-capable flag.",
            available: true,
        },
        DocLink {
            title: "OpenAPI description",
            path: "/openapi.json",
            kind: "OpenAPI 3.1",
            description: "Every local HTTP route with its method, body and responses.",
            available: true,
        },
        DocLink {
            title: "llms.txt",
            path: "/llms.txt",
            kind: "text/plain",
            description: "A compact summary written for language models and crawlers.",
            available: true,
        },
        DocLink {
            title: "security.txt",
            path: "/security.txt",
            kind: "RFC 9116",
            description: "Security policy pointer and expiry. Also at /.well-known/security.txt.",
            available: true,
        },
        DocLink {
            title: "robots.txt",
            path: "/robots.txt",
            kind: "text/plain",
            description: "Crawler rules for this local service.",
            available: true,
        },
        DocLink {
            title: "README",
            path: "/docs/README.md",
            kind: "Markdown",
            description: "Install, usage, local model setup and the deliberate limits.",
            available: true,
        },
        DocLink {
            title: "Health check",
            path: "/healthz",
            kind: "JSON",
            description: "Uptime, memory count and which reply engine is active.",
            available: true,
        },
        DocLink {
            title: "Hosted help centre",
            path: "#",
            kind: "—",
            description: "No hosted help desk exists for this build; a local README is the whole support surface.",
            available: false,
        },
    ]
}

fn gaps() -> Vec<Gap> {
    vec![
        Gap {
            label: "Support email",
            detail: "Not published. This build ships without a support desk or ticketing address.",
        },
        Gap {
            label: "Security contact",
            detail: "Not published. security.txt states the same rather than pointing at an address nobody reads.",
        },
        Gap {
            label: "Company address",
            detail: "Not applicable. RePikaChat is a local application, not a hosted service with an office.",
        },
    ]
}

pub fn build(brain: BrainView, data_dir: String, uptime_ms: u64) -> PublicProfile {
    PublicProfile {
        name: NAME,
        version: VERSION,
        tagline: "A local-first AI companion you can read the source of.",
        summary: SUMMARY,
        product_type: "Desktop AI companion application",
        runtime: "Single Rust binary (axum), embedded web UI, no runtime dependencies",
        storage: "One JSON save file on this computer",
        network: "Loopback only. No telemetry, no cloud calls; optional model traffic stays on 127.0.0.1",
        pricing: "Free",
        license: Some("MIT"),
        support_email: None,
        security_contact: None,
        docs: docs(),
        gaps: gaps(),
        endpoints: endpoints(),
        model: brain,
        data_dir,
        uptime_ms,
        generated_at_ms: crate::model::now_ms(),
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn engine_label(brain: &BrainView) -> String {
    if brain.active {
        let provider = match brain.provider.as_str() {
            "openai" => "OpenAI-compatible local server",
            _ => "Ollama",
        };
        format!("model · {} ({provider})", brain.model)
    } else {
        "built-in engine".to_string()
    }
}

fn doc_rows(docs: &[DocLink]) -> String {
    let mut out = String::new();
    for d in docs {
        if d.available {
            out.push_str(&format!(
                "<a class=\"doc\" href=\"{path}\"><span class=\"doc-kind\">{kind}</span>\
                 <span class=\"doc-body\"><strong>{title}</strong><code>{path}</code>\
                 <span class=\"doc-note\">{desc}</span></span><span class=\"doc-open\">open →</span></a>",
                path = esc(d.path),
                kind = esc(d.kind),
                title = esc(d.title),
                desc = esc(d.description),
            ));
        } else {
            out.push_str(&format!(
                "<div class=\"doc unavailable\"><span class=\"doc-kind\">{kind}</span>\
                 <span class=\"doc-body\"><strong>{title}</strong><span class=\"doc-note\">{desc}</span></span>\
                 <span class=\"doc-open muted\">not published</span></div>",
                kind = esc(d.kind),
                title = esc(d.title),
                desc = esc(d.description),
            ));
        }
    }
    out
}

fn endpoint_rows(endpoints: &[Endpoint]) -> String {
    let mut out = String::new();
    for e in endpoints {
        out.push_str(&format!(
            "<tr><td><span class=\"method {cls}\">{method}</span></td><td><code>{path}</code></td>\
             <td>{summary}</td><td class=\"auth\">{auth}</td></tr>",
            cls = e.method.to_ascii_lowercase(),
            method = e.method,
            path = esc(e.path),
            summary = esc(e.summary),
            auth = if e.token { "session token" } else { "local read" },
        ));
    }
    out
}

fn gap_rows(gaps: &[Gap]) -> String {
    let mut out = String::new();
    for g in gaps {
        out.push_str(&format!(
            "<div class=\"gap\"><span class=\"gap-label\">{label}</span>\
             <span class=\"gap-detail\">{detail}</span></div>",
            label = esc(g.label),
            detail = esc(g.detail),
        ));
    }
    out
}

/// Server-rendered so the page is complete before any script runs, and so a
/// missing field is handled by the same code path as a present one.
pub fn render_html(p: &PublicProfile) -> String {
    let uptime = p.uptime_ms / 1000;
    let uptime_label = if uptime >= 3600 {
        format!("{}h {}m", uptime / 3600, (uptime % 3600) / 60)
    } else if uptime >= 60 {
        format!("{}m {}s", uptime / 60, uptime % 60)
    } else {
        format!("{uptime}s")
    };
    let model_line = engine_label(&p.model);
    let model_status = if p.model.active {
        format!(
            "{} at {} (unverified until the next message)",
            esc(&p.model.model),
            esc(&p.model.endpoint)
        )
    } else {
        "No local model configured — the built-in engine answers deterministically.".to_string()
    };
    let license = p.license.map(esc).unwrap_or_else(|| "Not published".to_string());

    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>RePikaChat — public profile</title>
<meta name="description" content="RePikaChat is a standalone, local-first AI companion in Rust: a chat that learns from what you tell it, with a built-in engine and optional local models." />
<link rel="stylesheet" href="/profile.css" />
</head>
<body>
<header class="topbar">
  <div class="wrap topbar-in">
    <a class="brand" href="/profile">
      <span class="mark">R</span>
      <span class="brand-text"><strong>RePikaChat</strong><small>public profile</small></span>
    </a>
    <nav class="nav">
      <a href="#overview">Overview</a>
      <a href="#product">Product</a>
      <a href="#support">Support</a>
      <a href="#security">Security</a>
      <a href="#docs">Docs</a>
      <a class="nav-cta" href="/">Open app</a>
    </nav>
  </div>
</header>

<main>
  <section class="hero wrap">
    <p class="eyebrow">AI companion · local-first · Rust</p>
    <h1>RePikaChat</h1>
    <p class="tagline">{tagline}</p>
    <div class="chips">
      <span class="chip live"><i></i> running on this computer</span>
      <span class="chip">v{version}</span>
      <span class="chip">{model_line}</span>
      <span class="chip">MIT</span>
    </div>
    <p class="lede">{summary}</p>
    <div class="actions">
      <a class="btn primary" href="/">Open the chat</a>
      <a class="btn" href="/docs/README.md">Read the README</a>
      <a class="btn ghost" href="/api/public">JSON profile</a>
    </div>
  </section>

  <section id="overview" class="wrap section split">
    <div>
      <h2>Overview</h2>
      <p>Most companion apps are a thin client over someone else's server: you sign up, your conversations
      leave the machine, and the memory that makes the product feel personal is held somewhere you cannot
      read it. RePikaChat inverts that. One binary serves its own interface on loopback, keeps one companion
      and one transcript, and writes every fact it learns to a JSON file you own.</p>
      <p>The conversation engine is deliberately modest and fully inspectable. It extracts facts from what
      you type, scores their confidence, tracks a mood from the sentiment of your messages, and composes a
      reply in the persona you set up. Point it at a local model and the sentences improve; the facts, the
      memory and the guardrails do not change hands.</p>
      <p class="note">This is a single-user desktop application, not a hosted service. Every fact on this page
      comes from the running build, and fields that are not published say so.</p>
    </div>
    <aside class="facts">
      <dl>
        <div><dt>Product</dt><dd>{name}</dd></div>
        <div><dt>Type</dt><dd>{product_type}</dd></div>
        <div><dt>Version</dt><dd>{version}</dd></div>
        <div><dt>Runtime</dt><dd>{runtime}</dd></div>
        <div><dt>Storage</dt><dd>{storage}</dd></div>
        <div><dt>Network</dt><dd>{network}</dd></div>
        <div><dt>Model</dt><dd>{model_status}</dd></div>
        <div><dt>Price</dt><dd>{pricing}</dd></div>
        <div><dt>Licence</dt><dd>{license}</dd></div>
        <div><dt>Status</dt><dd><span class="ok">up {uptime_label}</span></dd></div>
      </dl>
    </aside>
  </section>

  <section id="product" class="wrap section">
    <h2>What it does</h2>
    <div class="cards">
      <article class="card">
        <h3>Remembers what you tell it</h3>
        <p>Nine kinds of fact are pulled out of ordinary phrasing — name, loves, dislikes, work, home, goals,
        feelings, pets, and anything after “remember that”. Repeats raise confidence instead of piling up.</p>
      </article>
      <article class="card">
        <h3>Shows its memory</h3>
        <p>Each stored fact is listed with a confidence bar, the sentence it came from, and a delete control.
        Ask “what do you know about me” and the answer is deterministic, never a guess.</p>
      </article>
      <article class="card">
        <h3>Grows with the conversation</h3>
        <p>Sentiment moves affinity between guarded and glowing, which changes how the companion speaks.
        Messages earn experience; levels raise the mood and mark the milestones in the transcript.</p>
      </article>
      <article class="card">
        <h3>Borrows a local model, optionally</h3>
        <p>Ollama and any OpenAI-compatible loopback server (llama.cpp, LM Studio, vLLM, LocalAI) can write the
        sentences. Timeouts, missing models and dead daemons fall back to the built-in engine mid-turn.</p>
      </article>
    </div>
  </section>

  <section id="replika" class="wrap section split">
    <div>
      <h2>About Replika</h2>
      <p>Replika is the commercial AI companion app made by Luka, Inc. (San Francisco), created by Eugenia
      Kuyda and available since 2017 on iOS, Android and the web. It is designed as an empathetic companion:
      it chats, keeps memories of what you share, and offers relationship modes, voice calls and avatar
      customisation.</p>
      <p>RePikaChat is an independent, local project built in that spirit. It is not affiliated with, endorsed
      by, or connected to Luka, Inc., and it uses no Replika code, models or data.</p>
    </div>
    <aside class="refs">
      <h3>Official Replika references</h3>
      <ul>
        <li><a href="https://replika.com/" rel="noopener">replika.com</a> — product site</li>
        <li><a href="https://help.replika.com/" rel="noopener">help.replika.com</a> — help centre</li>
        <li><a href="https://replika.com/legal/privacy" rel="noopener">Privacy policy</a></li>
      </ul>
      <p class="note">Links go to the official product. Support for Replika itself is handled by Luka, Inc.,
      not by this application.</p>
    </aside>
  </section>

  <section id="support" class="wrap section">
    <h2>Support &amp; contact</h2>
    <p>This build has no hosted support desk, so it does not pretend to have one. These are the channels that
    genuinely exist:</p>
    <div class="rows">
      <div class="row"><span>Documentation</span><span><a href="/docs/README.md">README</a> — install, usage, local models, limits</span></div>
      <div class="row"><span>API reference</span><span><a href="/openapi.json">openapi.json</a> and the <a href="#api">route table</a> below</span></div>
      <div class="row"><span>Diagnostics</span><span><a href="/healthz">/healthz</a> — uptime, engine, memory count</span></div>
      <div class="row"><span>Data location</span><span><code>{data_dir}</code> — one JSON file you can read, copy or delete</span></div>
      <div class="row"><span>Support email</span><span class="muted">Not published</span></div>
    </div>
    <h3 class="sub-head">Fields without a published value</h3>
    <div class="gaps">{gap_rows}</div>
  </section>

  <section id="security" class="wrap section">
    <h2>Security notes</h2>
    <ul class="checks">
      <li><strong>Loopback only.</strong> The server binds 127.0.0.1. Nothing is exposed to your network, and
      there is no remote access mode to misconfigure.</li>
      <li><strong>No telemetry, no accounts, no cloud.</strong> No analytics, crash reporting or update checks
      are sent anywhere. The only outbound sockets go to a loopback model endpoint you configure.</li>
      <li><strong>Model traffic stays local.</strong> Endpoints are validated before use: 127.0.0.1, ::1 or
      localhost only, plain HTTP. A remote host is refused rather than silently used.</li>
      <li><strong>Mutations need a session token.</strong> Every POST and DELETE requires the per-run
      <code>x-repikachat-token</code> header that only the served page knows, so another site cannot make your
      companion forget or reset things.</li>
      <li><strong>Your data is inspectable and erasable.</strong> Memories are listed with their source
      sentence; a single fact can be deleted from the panel, and the whole file can be exported or erased in
      one action.</li>
      <li><strong>Deterministic answers stay deterministic.</strong> “what do you know about me”, “forget …” and
      crisis replies never reach a model, so they cannot be embellished.</li>
      <li><strong>Installer is per-user.</strong> The Windows installer writes to
      <code>%LOCALAPPDATA%\\Programs\\RePikaChat</code> and asks for no administrator rights.</li>
    </ul>
    <div class="rows">
      <div class="row"><span>Security contact</span><span class="muted">Not published</span></div>
      <div class="row"><span>Disclosure policy</span><span class="muted">Not published</span></div>
      <div class="row"><span>security.txt</span><span><a href="/security.txt">/security.txt</a> and <a href="/.well-known/security.txt">/.well-known/security.txt</a></span></div>
    </div>
    <p class="note">No diagnosis, no medical claims, and no framing of the companion as a replacement for
    people. If you are in danger, contact your local emergency number or a crisis line.</p>
  </section>

  <section id="docs" class="wrap section">
    <h2>Public docs &amp; machine-readable files</h2>
    <p>These are served by this running instance, not aspirational links. Anything this build does not
    publish is labelled instead of hidden.</p>
    <div class="docs">{doc_rows}</div>
  </section>

  <section id="api" class="wrap section">
    <h2>Local HTTP API</h2>
    <p>The same API the interface uses. JSON in, JSON out, errors as <code>{{"error":"…"}}</code>. It listens on
    loopback only; there is no hosted API and no key to obtain.</p>
    <div class="table-wrap">
      <table class="api">
        <thead><tr><th>Method</th><th>Path</th><th>Purpose</th><th>Auth</th></tr></thead>
        <tbody>{endpoint_rows}</tbody>
      </table>
    </div>
  </section>
</main>

<footer class="footer">
  <div class="wrap footer-in">
    <span>{name} v{version} · MIT · one consumer, one bot, one JSON file.</span>
    <span class="muted">Generated from this running build. Replika is a product of Luka, Inc.; this project is independent.</span>
  </div>
</footer>
</body>
</html>"##,
        tagline = esc(p.tagline),
        version = esc(p.version),
        model_line = esc(&model_line),
        summary = esc(p.summary),
        name = esc(p.name),
        product_type = esc(p.product_type),
        runtime = esc(p.runtime),
        storage = esc(p.storage),
        network = esc(p.network),
        model_status = model_status,
        pricing = esc(p.pricing),
        license = license,
        uptime_label = uptime_label,
        data_dir = esc(&p.data_dir),
        gap_rows = gap_rows(&p.gaps),
        doc_rows = doc_rows(&p.docs),
        endpoint_rows = endpoint_rows(&p.endpoints),
    )
}

pub fn openapi(origin: &str) -> serde_json::Value {
    let mut paths = serde_json::Map::new();
    for e in endpoints() {
        let path = e.path.replace(":id", "{id}");
        let mut op = serde_json::json!({
            "summary": e.summary,
            "responses": {
                "200": { "description": "Success" },
                "400": { "description": "Invalid input" },
                "403": { "description": "Missing or stale session token" }
            }
        });
        if e.token {
            if let Some(obj) = op.as_object_mut() {
                obj.insert("security".to_string(), serde_json::json!([{ "sessionToken": [] }]));
            }
        }
        let mut entry = serde_json::Map::new();
        entry.insert(e.method.to_ascii_lowercase(), op);
        paths.insert(path, serde_json::Value::Object(entry));
    }

    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "RePikaChat local API",
            "version": VERSION,
            "description": "Loopback-only HTTP API for a single-user companion chat. No authentication service; mutating routes need the per-run session token injected into the served page.",
            "license": { "name": "MIT" }
        },
        "servers": [{ "url": origin, "description": "This machine" }],
        "paths": paths,
        "components": {
            "securitySchemes": {
                "sessionToken": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-repikachat-token"
                }
            }
        }
    })
}

pub fn llms_txt(p: &PublicProfile) -> String {
    let mut out = String::new();
    out.push_str("# RePikaChat\n\n");
    out.push_str(&format!("> {}\n\n", p.summary));
    out.push_str(&format!("Version: {}\n", p.version));
    out.push_str(&format!("Type: {}\n", p.product_type));
    out.push_str(&format!("Runtime: {}\n", p.runtime));
    out.push_str(&format!("Storage: {}\n", p.storage));
    out.push_str(&format!("Network: {}\n", p.network));
    out.push_str(&format!("Price: {}\n", p.pricing));
    out.push_str(&format!("License: {}\n\n", p.license.unwrap_or("not published")));

    out.push_str("## What it does\n\n");
    out.push_str("- Learns nine kinds of personal fact from natural phrasing, with confidence and source sentence.\n");
    out.push_str("- Shows every memory and deletes one on request; recall is deterministic.\n");
    out.push_str("- Tracks affinity and levels from conversation sentiment and length.\n");
    out.push_str("- Optional local model: Ollama or any OpenAI-compatible loopback server.\n");
    out.push_str("- Single user, single save file, loopback only, no telemetry.\n\n");

    out.push_str("## Machine-readable files\n\n");
    for d in p.docs.iter().filter(|d| d.available) {
        out.push_str(&format!("- {}: {} ({})\n", d.title, d.path, d.kind));
    }
    out.push_str("\n## API\n\n");
    for e in &p.endpoints {
        out.push_str(&format!("- {} {} — {}\n", e.method, e.path, e.summary));
    }
    out.push_str("\n## Relationship to Replika\n\n");
    out.push_str("Replika is a commercial AI companion by Luka, Inc. RePikaChat is an independent local project inspired by it and is not affiliated with Luka, Inc.\n");
    out.push_str("\n## Support and security contacts\n\n");
    out.push_str("Not published for this build. See /profile#support and /security.txt.\n");
    out
}

pub fn robots_txt() -> String {
    "User-agent: *\nAllow: /profile\nAllow: /llms.txt\nAllow: /openapi.json\nDisallow: /api/\nDisallow: /app.js\nDisallow: /app.css\n".to_string()
}

fn ymd_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn security_txt(origin: &str) -> String {
    let now_days = (crate::model::now_ms() / 1000 / 86_400) as i64;
    let (y, m, d) = ymd_from_days(now_days + 365);
    format!(
        "# RePikaChat security policy\n\
         # No security contact address has been published for this standalone build.\n\
         # If you found this binary through a distributor, report issues to them privately.\n\
         Canonical: {origin}/security.txt\n\
         Policy: {origin}/profile#security\n\
         Preferred-Languages: en\n\
         Expires: {y:04}-{m:02}-{d:02}T00:00:00Z\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_published_doc_has_a_real_path() {
        for d in docs().iter().filter(|d| d.available) {
            assert!(d.path.starts_with('/'), "{} has no path", d.title);
            assert_ne!(d.path, "/", "{} points at the app root", d.title);
        }
    }

    #[test]
    fn openapi_covers_every_endpoint() {
        let spec = openapi("http://127.0.0.1:7719");
        let paths = spec["paths"].as_object().unwrap();
        assert!(paths.contains_key("/api/chat"));
        assert!(paths.contains_key("/api/memory/{id}"));
        assert_eq!(paths.len(), endpoints().len());
    }

    #[test]
    fn the_page_never_renders_a_broken_link_for_missing_docs() {
        let profile = build(
            crate::llm::brain_view(&crate::model::ModelSettings::default()),
            "C:/tmp".to_string(),
            61_000,
        );
        let html = render_html(&profile);
        assert!(html.contains("not published"));
        assert!(html.contains("Not published"));
        assert!(html.contains("01:01") || html.contains("1m"));
    }

    #[test]
    fn security_txt_has_a_future_expiry() {
        let text = security_txt("http://127.0.0.1:7719");
        assert!(text.contains("Expires: 20"));
        assert!(text.contains("Canonical: http://127.0.0.1:7719/security.txt"));
    }
}
