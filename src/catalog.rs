//! Curated open generative AI catalog.
//!
//! The shape (one central model list, a category per entry, a selector that
//! switches the active model) is inspired by Anil-matcha/Open-Generative-AI,
//! an MIT-licensed image/video studio built around a single `models.js`.
//!
//! RePikaChat adds no hosted APIs and no cloud calls, so this catalog only
//! lists downloadable open-weight models. Chat entries can be set as the reply
//! engine through a local Ollama daemon. Image, video, audio and embedding
//! entries are listed for reference and are explicitly marked as not usable as
//! a chat engine, because this build only writes text.
//!
//! Every entry carries a rough memory estimate, and the catalog sizes those
//! against the memory this machine can actually give a model (see
//! [`crate::hardware`]), so a 70B suggestion never arrives without a warning.

use serde::Serialize;

pub const ATTRIBUTION_NAME: &str = "Open Generative AI";
pub const ATTRIBUTION_URL: &str = "https://github.com/anil-matcha/open-generative-ai";
pub const ATTRIBUTION_NOTE: &str =
    "Catalog shape inspired by Open Generative AI (MIT). Only downloadable open-weight models are listed; hosted model APIs are out of scope for a local-only app.";

#[derive(Serialize, Clone)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub org: &'static str,
    /// chat | image | video | audio | embedding
    pub category: &'static str,
    pub parameters: &'static str,
    pub license: &'static str,
    /// How you would run it locally: ollama, llama.cpp, diffusers, sd.cpp, ComfyUI, Wan2GP, transformers.
    pub runtime: &'static str,
    /// Reply-engine provider this entry can drive: ollama, openai or none.
    pub engine: &'static str,
    /// Ollama tag for chat entries; empty otherwise.
    pub model_tag: &'static str,
    pub summary: &'static str,
    pub source: &'static str,
    pub chat_capable: bool,
    pub setup_note: &'static str,
}

const PULL: &str =
    "Pull it first with `ollama pull {tag}`. If it is missing, the built-in engine still answers the turn.";

pub static ENTRY_LIST: &[CatalogEntry] = &[
    // ── chat / text ──────────────────────────────────────────
    CatalogEntry {
        id: "llama32-3b",
        name: "Llama 3.2 3B Instruct",
        org: "Meta",
        category: "chat",
        parameters: "3B",
        license: "Llama 3.2 Community Licence",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "llama3.2:3b",
        summary: "Small general chat model; the default pick for a companion on modest hardware.",
        source: "https://huggingface.co/meta-llama/Llama-3.2-3B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "llama32-1b",
        name: "Llama 3.2 1B Instruct",
        org: "Meta",
        category: "chat",
        parameters: "1B",
        license: "Llama 3.2 Community Licence",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "llama3.2:1b",
        summary: "Fastest option here; shallow but responsive, good for a low-power machine.",
        source: "https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "qwen25-7b",
        name: "Qwen2.5 7B Instruct",
        org: "Alibaba",
        category: "chat",
        parameters: "7B",
        license: "Apache-2.0",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "qwen2.5:7b-instruct",
        summary: "Strong instruction following and warm conversational tone; the model used to verify this build.",
        source: "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "qwen3-8b",
        name: "Qwen3 8B",
        org: "Alibaba",
        category: "chat",
        parameters: "8B",
        license: "Apache-2.0",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "qwen3:8b",
        summary: "Reasoning-capable sibling of Qwen2.5; may produce thinking traces, so expect terser replies.",
        source: "https://huggingface.co/Qwen/Qwen3-8B",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "gemma3-4b",
        name: "Gemma 3 4B IT",
        org: "Google DeepMind",
        category: "chat",
        parameters: "4B",
        license: "Gemma Terms of Use",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "gemma3:4b",
        summary: "Compact instruct model with a gentle register; a good middle ground under 7B.",
        source: "https://huggingface.co/google/gemma-3-4b-it",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "mistral-7b",
        name: "Mistral 7B Instruct v0.3",
        org: "Mistral AI",
        category: "chat",
        parameters: "7B",
        license: "Apache-2.0",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "mistral:7b",
        summary: "Plain, direct prose without much flourish; useful when you want less personality.",
        source: "https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.3",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "phi4-14b",
        name: "Phi-4 14B",
        org: "Microsoft",
        category: "chat",
        parameters: "14B",
        license: "MIT",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "phi4:14b",
        summary: "Performs above its size on reasoning; needs a roomy GPU to stay quick.",
        source: "https://huggingface.co/microsoft/phi-4",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "deepseek-r1-1.5b",
        name: "DeepSeek-R1 Distill 1.5B",
        org: "DeepSeek",
        category: "chat",
        parameters: "1.5B",
        license: "MIT",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "deepseek-r1:1.5b",
        summary: "Tiny reasoning distill; verbose internal traces can leak into chat replies.",
        source: "https://huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "llama33-70b",
        name: "Llama 3.3 70B Instruct",
        org: "Meta",
        category: "chat",
        parameters: "70B",
        license: "Llama 3.3 Community Licence",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "llama3.3:70b",
        summary: "The strongest writer here if the machine can hold it; slow on consumer hardware.",
        source: "https://huggingface.co/meta-llama/Llama-3.3-70B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "smollm2-1.7b",
        name: "SmolLM2 1.7B Instruct",
        org: "Hugging Face",
        category: "chat",
        parameters: "1.7B",
        license: "Apache-2.0",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "smollm2:1.7b",
        summary: "Very small and permissive; charming for short turns, weak at long context.",
        source: "https://huggingface.co/HuggingFaceTB/SmolLM2-1.7B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    CatalogEntry {
        id: "olmo2-7b",
        name: "OLMo 2 7B Instruct",
        org: "Allen Institute for AI",
        category: "chat",
        parameters: "7B",
        license: "Apache-2.0",
        runtime: "ollama",
        engine: "ollama",
        model_tag: "olmo2:7b",
        summary: "Fully open model with published training data; a good auditability pick.",
        source: "https://huggingface.co/allenai/OLMo-2-1124-7B-Instruct",
        chat_capable: true,
        setup_note: PULL,
    },
    // ── image ────────────────────────────────────────────────
    CatalogEntry {
        id: "flux-schnell",
        name: "FLUX.1 [schnell]",
        org: "Black Forest Labs",
        category: "image",
        parameters: "12B",
        license: "Apache-2.0",
        runtime: "diffusers / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Four-step text-to-image model; the permissively licensed FLUX variant.",
        source: "https://huggingface.co/black-forest-labs/FLUX.1-schnell",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "flux-dev",
        name: "FLUX.1 [dev]",
        org: "Black Forest Labs",
        category: "image",
        parameters: "12B",
        license: "FLUX.1 dev Non-Commercial Licence",
        runtime: "diffusers / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Higher-fidelity FLUX variant; weights are non-commercial.",
        source: "https://huggingface.co/black-forest-labs/FLUX.1-dev",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "sdxl-base",
        name: "Stable Diffusion XL Base 1.0",
        org: "Stability AI",
        category: "image",
        parameters: "3.5B UNet",
        license: "CreativeML OpenRAIL++-M",
        runtime: "sd.cpp / diffusers",
        engine: "none",
        model_tag: "",
        summary: "Widely supported SDXL checkpoint; the usual first local image test.",
        source: "https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "sd35-large",
        name: "Stable Diffusion 3.5 Large",
        org: "Stability AI",
        category: "image",
        parameters: "8B",
        license: "Stability AI Community Licence",
        runtime: "ComfyUI / diffusers",
        engine: "none",
        model_tag: "",
        summary: "MMDiT image model with stronger prompt adherence than SDXL.",
        source: "https://huggingface.co/stabilityai/stable-diffusion-3.5-large",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "qwen-image",
        name: "Qwen-Image",
        org: "Alibaba",
        category: "image",
        parameters: "20B",
        license: "Apache-2.0",
        runtime: "diffusers / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Apache-licensed image model that is also strong at in-image text rendering.",
        source: "https://huggingface.co/Qwen/Qwen-Image",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "dreamshaper-8",
        name: "DreamShaper 8",
        org: "Lykon",
        category: "image",
        parameters: "~2 GB SD1.5",
        license: "CreativeML OpenRAIL-M",
        runtime: "sd.cpp / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Small SD1.5 finetune; the fastest way to check a local image pipeline works.",
        source: "https://huggingface.co/Lykon/DreamShaper",
        chat_capable: false,
        setup_note: "",
    },
    // ── video ────────────────────────────────────────────────
    CatalogEntry {
        id: "wan21-1.3b",
        name: "Wan 2.1 T2V 1.3B",
        org: "Alibaba",
        category: "video",
        parameters: "1.3B",
        license: "Apache-2.0",
        runtime: "Wan2GP / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Lightest open text-to-video model; 480p output on a consumer GPU.",
        source: "https://huggingface.co/Wan-AI/Wan2.1-T2V-1.3B",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "hunyuanvideo",
        name: "HunyuanVideo",
        org: "Tencent",
        category: "video",
        parameters: "13B",
        license: "Tencent Hunyuan Community Licence",
        runtime: "Wan2GP / ComfyUI",
        engine: "none",
        model_tag: "",
        summary: "Large text-to-video model; needs serious VRAM and patience.",
        source: "https://huggingface.co/tencent/HunyuanVideo",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "ltx-video",
        name: "LTX-Video",
        org: "Lightricks",
        category: "video",
        parameters: "2B / 13B",
        license: "Lightricks licence (see model card)",
        runtime: "ComfyUI / diffusers",
        engine: "none",
        model_tag: "",
        summary: "Fast DiT video model designed for short clips at low step counts.",
        source: "https://huggingface.co/Lightricks/LTX-Video",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "mochi-1",
        name: "Mochi 1 Preview",
        org: "Genmo",
        category: "video",
        parameters: "10B",
        license: "Apache-2.0",
        runtime: "ComfyUI / diffusers",
        engine: "none",
        model_tag: "",
        summary: "Asymmetric DiT video model released under Apache-2.0.",
        source: "https://huggingface.co/genmo/mochi-1-preview",
        chat_capable: false,
        setup_note: "",
    },
    // ── audio ────────────────────────────────────────────────
    CatalogEntry {
        id: "whisper-large-v3",
        name: "Whisper large-v3",
        org: "OpenAI",
        category: "audio",
        parameters: "1.55B",
        license: "MIT",
        runtime: "whisper.cpp / transformers",
        engine: "none",
        model_tag: "",
        summary: "Speech-to-text model covering many languages; useful for voice input later.",
        source: "https://huggingface.co/openai/whisper-large-v3",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "kokoro-82m",
        name: "Kokoro 82M",
        org: "hexgrad",
        category: "audio",
        parameters: "82M",
        license: "Apache-2.0",
        runtime: "transformers / ONNX",
        engine: "none",
        model_tag: "",
        summary: "Small text-to-speech model; light enough to run beside the chat engine.",
        source: "https://huggingface.co/hexgrad/Kokoro-82M",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "musicgen-small",
        name: "MusicGen Small",
        org: "Meta",
        category: "audio",
        parameters: "300M",
        license: "CC-BY-NC 4.0 (weights)",
        runtime: "transformers / audiocraft",
        engine: "none",
        model_tag: "",
        summary: "Text-to-music generation; weights are non-commercial.",
        source: "https://huggingface.co/facebook/musicgen-small",
        chat_capable: false,
        setup_note: "",
    },
    // ── embeddings ───────────────────────────────────────────
    CatalogEntry {
        id: "bge-m3",
        name: "BGE-M3",
        org: "BAAI",
        category: "embedding",
        parameters: "568M",
        license: "MIT",
        runtime: "transformers / Ollama",
        engine: "none",
        model_tag: "",
        summary: "Multilingual embedding model; the obvious base for searching old memories.",
        source: "https://huggingface.co/BAAI/bge-m3",
        chat_capable: false,
        setup_note: "",
    },
    CatalogEntry {
        id: "nomic-embed-text",
        name: "Nomic Embed Text v1.5",
        org: "Nomic AI",
        category: "embedding",
        parameters: "137M",
        license: "Apache-2.0",
        runtime: "transformers / Ollama",
        engine: "none",
        model_tag: "",
        summary: "Compact long-context embedding model for local retrieval.",
        source: "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5",
        chat_capable: false,
        setup_note: "",
    },
];

pub fn entries() -> &'static [CatalogEntry] {
    ENTRY_LIST
}

pub fn find(id: &str) -> Option<&'static CatalogEntry> {
    entries().iter().find(|e| e.id == id)
}

/// Rough memory a local run needs, in MB.
///
/// Chat weights assume a 4-bit quantisation (the usual Ollama default); the
/// generative entries assume the smallest common local checkpoint. These are
/// sizing estimates for choosing between models, not benchmarks.
pub fn estimate_mb(entry: &CatalogEntry) -> Option<u64> {
    let (per_billion, overhead) = match entry.category {
        "chat" => (600, 1200),
        "image" => (1000, 2000),
        "video" => (1100, 3500),
        "audio" => (700, 700),
        "embedding" => (700, 300),
        _ => return None,
    };
    if let Some(gb) = gigabytes(&entry.parameters) {
        return Some(gb * 1024 + 1024);
    }
    let billions = params_billion(&entry.parameters)?;
    Some((billions * per_billion as f32).round() as u64 + overhead)
}

/// Some entries state their footprint outright, such as `~2 GB SD1.5`.
fn gigabytes(parameters: &str) -> Option<u64> {
    let head = parameters[..parameters.find("GB")?].trim_end();
    let number: String = head
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();
    number.parse::<f32>().ok().map(|gb| gb.round() as u64)
}

/// First `<number>B` or `<number>M` in a parameter label: `7B`, `1.5B`, `568M`,
/// `2B / 13B`, `3.5B UNet`.
fn params_billion(parameters: &str) -> Option<f32> {
    let bytes = parameters.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
            i += 1;
        }
        let number: f32 = parameters[start..i].parse().ok()?;
        let mut suffix = i;
        while suffix < bytes.len() && bytes[suffix] == b' ' {
            suffix += 1;
        }
        let ends_word = |at: usize| at + 1 >= bytes.len() || !bytes[at + 1].is_ascii_alphabetic();
        match bytes.get(suffix) {
            Some(b'B') if ends_word(suffix) => return Some(number),
            Some(b'M') if ends_word(suffix) => return Some(number / 1000.0),
            _ => {}
        }
        i = suffix.max(i);
    }
    None
}

/// Sizing for a model name that may not be in the catalog: the curated entry
/// wins, otherwise the parameter count in the tag itself (`:16b`, `:7b`) gets
/// the chat formula. Returns the estimate and the catalog entry it matched.
pub fn size_for(tag: &str) -> (Option<u64>, Option<&'static str>) {
    let tag = tag.trim().to_lowercase();
    if tag.is_empty() {
        return (None, None);
    }
    if let Some(entry) = entries()
        .iter()
        .find(|e| e.chat_capable && is_installed(e.model_tag, &[tag.clone()]))
    {
        return (estimate_mb(entry), Some(entry.id));
    }
    let estimate = tag_billions(&tag).map(|b| (b * 600.0).round() as u64 + 1200);
    (estimate, None)
}

/// The parameter count at the end of an Ollama tag: `qwen2.5:7b-instruct` → 7,
/// `llama3.2:1b` → 1, `phi3:mini` → None.
fn tag_billions(tag: &str) -> Option<f32> {
    let tail = tag.rsplit(':').next()?;
    let mut number = String::new();
    for c in tail.chars() {
        if c.is_ascii_digit() || c == '.' {
            number.push(c);
        } else {
            break;
        }
    }
    let value: f32 = number.parse().ok()?;
    let unit = tail[number.len()..].chars().next()?.to_ascii_lowercase();
    match unit {
        'b' => Some(value),
        'm' => Some(value / 1000.0),
        _ => None,
    }
}

/// How an estimate compares with what the machine can hold: `good` (fits with
/// room to spare), `tight` (runs, but with little context or partial offload),
/// `heavy` (needs more memory than this machine can offer) or `unknown`.
pub fn fit_for(estimate_mb: Option<u64>, budget_mb: u64) -> &'static str {
    match estimate_mb {
        Some(need) if budget_mb > 0 => {
            if need * 100 <= budget_mb * 85 {
                "good"
            } else if need * 100 <= budget_mb * 135 {
                "tight"
            } else {
                "heavy"
            }
        }
        _ => "unknown",
    }
}

pub fn fit_label(fit: &str) -> &'static str {
    match fit {
        "good" => "fits comfortably",
        "tight" => "tight fit",
        "heavy" => "needs more memory",
        _ => "size unknown",
    }
}

pub struct Pick {
    pub id: &'static str,
    pub model_tag: &'static str,
    pub estimate_mb: u64,
    pub note: String,
}

/// The largest chat model that fits comfortably, or the smallest one when
/// nothing does. `None` when the machine could not be measured.
pub fn recommend(budget_mb: u64) -> Option<Pick> {
    if budget_mb == 0 {
        return None;
    }
    let sized: Vec<(&CatalogEntry, u64)> = entries()
        .iter()
        .filter(|e| e.chat_capable)
        .filter_map(|e| estimate_mb(e).map(|mb| (e, mb)))
        .collect();

    // Largest first, keeping catalog order among models of the same size, so a
    // tie goes to the earlier entry rather than the last one listed.
    let mut fitting: Vec<&(&CatalogEntry, u64)> = sized
        .iter()
        .filter(|(_, mb)| mb * 100 <= budget_mb * 85)
        .collect();
    fitting.sort_by_key(|(_, mb)| std::cmp::Reverse(*mb));

    let (entry, estimate) = match fitting.first() {
        Some(found) => **found,
        None => *sized.iter().min_by_key(|(_, mb)| *mb)?,
    };
    let gb = |mb: u64| mb as f64 / 1024.0;
    let fits = estimate * 100 <= budget_mb * 85;
    let note = if fits {
        format!(
            "{} is the largest chat model that fits here: about {:.1} GB of the {:.1} GB this machine can give a model.",
            entry.name,
            gb(estimate),
            gb(budget_mb)
        )
    } else {
        format!(
            "Nothing here fits comfortably. {} is the smallest option at about {:.1} GB, and it will lean on the CPU.",
            entry.name,
            gb(estimate)
        )
    };
    Some(Pick {
        id: entry.id,
        model_tag: entry.model_tag,
        estimate_mb: estimate,
        note,
    })
}

fn category_label(id: &str) -> &'static str {
    match id {
        "chat" => "Chat & text",
        "image" => "Image",
        "video" => "Video",
        "audio" => "Audio",
        "embedding" => "Embeddings",
        _ => "Other",
    }
}

const CATEGORY_ORDER: [&str; 5] = ["chat", "image", "video", "audio", "embedding"];

/// Ollama reports pulled models as `name` or `name:latest`; both count.
pub fn is_installed(tag: &str, installed: &[String]) -> bool {
    if tag.trim().is_empty() {
        return false;
    }
    let tag = tag.trim().to_lowercase();
    let latest = format!("{tag}:latest");
    installed.iter().any(|name| {
        let name = name.trim().to_lowercase();
        name == tag || name == latest
    })
}

/// Adds the no-download hint when the recommendation is already pulled.
fn disk_note(note: &str, tag: &str, installed: Option<&[String]>) -> String {
    match installed {
        Some(list) if is_installed(tag, list) => {
            format!("{note} It is already on disk, so no download is needed.")
        }
        _ => note.to_string(),
    }
}

/// The catalog as the API serves it. `installed` is the model names the local
/// runtime reported; `None` when it could not be asked, and no entry then
/// claims to be on disk.
pub fn json(installed: Option<&[String]>) -> serde_json::Value {
    let hardware = crate::hardware::detect();
    let budget = hardware.budget_mb();

    let categories: Vec<serde_json::Value> = CATEGORY_ORDER
        .iter()
        .map(|id| {
            let count = entries().iter().filter(|e| e.category == *id).count();
            serde_json::json!({
                "id": id,
                "label": category_label(id),
                "count": count,
            })
        })
        .collect();

    let listed: Vec<serde_json::Value> = entries()
        .iter()
        .map(|entry| {
            let estimate = estimate_mb(entry);
            let fit = fit_for(estimate, budget);
            let mut value = serde_json::to_value(entry).expect("catalog entries serialise");
            value["estimate_mb"] = serde_json::json!(estimate);
            value["fit"] = serde_json::json!(fit);
            value["fit_label"] = serde_json::json!(fit_label(fit));
            value["installed"] = match (entry.chat_capable, installed) {
                (true, Some(list)) => serde_json::json!(is_installed(entry.model_tag, list)),
                _ => serde_json::Value::Null,
            };
            value
        })
        .collect();

    let recommendation = recommend(budget).map(|pick| {
        serde_json::json!({
            "id": pick.id,
            "model_tag": pick.model_tag,
            "estimate_mb": pick.estimate_mb,
            "installed": match installed {
                Some(list) => serde_json::json!(is_installed(pick.model_tag, list)),
                None => serde_json::Value::Null,
            },
            "note": disk_note(&pick.note, pick.model_tag, installed),
        })
    });

    serde_json::json!({
        "attribution": {
            "name": ATTRIBUTION_NAME,
            "url": ATTRIBUTION_URL,
            "note": ATTRIBUTION_NOTE,
            "license": "MIT",
        },
        "count": entries().len(),
        "chat_capable": entries().iter().filter(|e| e.chat_capable).count(),
        "categories": categories,
        "hardware": hardware.json(),
        "recommendation": recommendation,
        "disk": {
            "checked": installed.is_some(),
            "models": installed.map(|list| list.len()).unwrap_or(0),
        },
        "entries": listed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_is_unique_and_known() {
        let mut seen = std::collections::HashSet::new();
        for e in entries() {
            assert!(seen.insert(e.id), "duplicate catalog id {}", e.id);
            assert!(find(e.id).is_some());
        }
    }

    #[test]
    fn every_chat_entry_can_drive_the_engine() {
        for e in entries().iter().filter(|e| e.chat_capable) {
            assert_eq!(e.category, "chat", "{} is chat-capable but not in chat", e.id);
            assert!(!e.model_tag.is_empty(), "{} has no model tag", e.id);
            assert!(
                matches!(e.engine, "ollama" | "openai"),
                "{} has no usable engine",
                e.id
            );
            assert!(e.setup_note.contains("{tag}"));
        }
    }

    #[test]
    fn non_chat_entries_are_not_offered_as_engines() {
        for e in entries().iter().filter(|e| !e.chat_capable) {
            assert_eq!(e.engine, "none", "{} should not claim an engine", e.id);
            assert!(e.model_tag.is_empty(), "{} should not carry an ollama tag", e.id);
        }
    }

    #[test]
    fn every_entry_links_to_a_model_card() {
        for e in entries() {
            assert!(e.source.starts_with("https://"), "{} has no source", e.id);
            assert!(!e.license.is_empty(), "{} has no licence", e.id);
        }
    }

    #[test]
    fn the_json_counts_match_the_catalog() {
        let value = json(None);
        assert_eq!(value["count"], entries().len());
        assert_eq!(
            value["chat_capable"],
            entries().iter().filter(|e| e.chat_capable).count()
        );
        let total: u64 = value["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["count"].as_u64().unwrap())
            .sum();
        assert_eq!(total as usize, entries().len());
    }

    #[test]
    fn every_entry_has_a_size_estimate() {
        for e in entries() {
            let mb = estimate_mb(e).unwrap_or_else(|| panic!("{} has no estimate", e.id));
            assert!(mb > 0, "{} estimates to nothing", e.id);
        }
    }

    #[test]
    fn a_bigger_model_estimates_larger() {
        let small = estimate_mb(find("llama32-1b").unwrap()).unwrap();
        let big = estimate_mb(find("llama33-70b").unwrap()).unwrap();
        assert!(big > small * 10, "{big} MB vs {small} MB");
        assert_eq!(fit_for(Some(big), 6451), "heavy");
    }

    #[test]
    #[test]
    fn any_tag_can_be_sized() {
        assert_eq!(
            size_for("qwen2.5:7b-instruct"),
            (Some(5400), Some("qwen25-7b")),
            "a catalog tag uses the curated entry"
        );
        assert_eq!(
            size_for("QWEN2.5:7B-INSTRUCT"),
            (Some(5400), Some("qwen25-7b")),
            "matching ignores case"
        );
        assert_eq!(size_for("deepseek-coder-v2:16b"), (Some(10800), None));
        assert_eq!(size_for("some-model:7b"), (Some(5400), None));
        assert_eq!(size_for("phi4:latest"), (None, None));
        assert_eq!(size_for("mystery"), (None, None));
        assert_eq!(size_for("   "), (None, None));
        assert_eq!(fit_for(size_for("deepseek-coder-v2:16b").0, 6451), "heavy");
        assert_eq!(fit_for(size_for("some-model:1b").0, 6451), "good");
    }

    #[test]
    fn fit_tiers_follow_the_budget() {
        assert_eq!(fit_for(Some(3000), 8000), "good");
        assert_eq!(fit_for(Some(7000), 8000), "tight");
        assert_eq!(fit_for(Some(20000), 8000), "heavy");
        assert_eq!(fit_for(Some(3000), 0), "unknown");
        assert_eq!(fit_for(None, 8000), "unknown");
        assert_eq!(fit_label("good"), "fits comfortably");
    }

    #[test]
    fn the_recommendation_is_the_largest_chat_model_that_fits() {
        let pick = recommend(8000).expect("a budget yields a pick");
        assert_eq!(pick.id, "qwen3-8b");
        assert!(pick.note.contains("largest chat model that fits"), "{}", pick.note);
        let fallback = recommend(900).expect("a small budget still gets a pick");
        assert_eq!(fallback.id, "llama32-1b");
        assert!(fallback.note.contains("smallest"), "{}", fallback.note);
        assert!(recommend(0).is_none());

        // 6451 MB is this machine's GPU budget: the three 7B entries tie, and
        // the tie goes to the first in catalog order.
        let tied = recommend(6451).expect("a GPU budget yields a pick");
        assert_eq!(tied.id, "qwen25-7b", "ties keep catalog order");
    }

    #[test]
    fn the_json_reports_hardware_and_fits() {
        let value = json(None);
        assert!(value["hardware"]["budget_mb"].is_u64());
        assert!(value["hardware"]["summary"].is_string());
        let listed = value["entries"].as_array().unwrap();
        assert_eq!(listed.len(), entries().len());
        assert!(listed.iter().all(|e| e["fit"].is_string()));
        assert_eq!(
            listed[0]["estimate_mb"],
            estimate_mb(&entries()[0]).unwrap()
        );
    }

    #[test]
    fn pulled_models_are_marked_on_disk() {
        let on_disk = vec![
            "qwen2.5:7b-instruct".to_string(),
            "llama3.2:3b:latest".to_string(),
        ];
        assert!(is_installed("qwen2.5:7b-instruct", &on_disk));
        assert!(is_installed("llama3.2:3b", &on_disk), "the :latest alias counts");
        assert!(
            is_installed("QWEN2.5:7B-INSTRUCT", &on_disk),
            "tags compare case-insensitively"
        );
        assert!(!is_installed("qwen3:8b", &on_disk));
        assert!(!is_installed("", &on_disk));

        let value = json(Some(&on_disk));
        assert!(value["disk"]["checked"].as_bool().unwrap());
        assert_eq!(value["disk"]["models"], on_disk.len());
        let listed = value["entries"].as_array().unwrap();
        for (entry, row) in entries().iter().zip(listed) {
            if entry.chat_capable {
                assert_eq!(
                    row["installed"],
                    serde_json::json!(is_installed(entry.model_tag, &on_disk)),
                    "{}",
                    entry.id
                );
            } else {
                assert_eq!(row["installed"], serde_json::Value::Null, "{}", entry.id);
            }
        }
        assert!(listed.iter().any(|r| r["installed"] == true));
        assert!(listed.iter().any(|r| r["installed"] == false));

        assert!(!json(None)["disk"]["checked"].as_bool().unwrap());

        let pick = recommend(6451).expect("a budget yields a pick");
        assert!(disk_note(&pick.note, pick.model_tag, Some(&on_disk)).contains("already on disk"));
        assert!(!disk_note(&pick.note, pick.model_tag, None).contains("already on disk"));
    }
}
