//! State, persona, memory records and view types for RePikaChat.
//!
//! One consumer, one bot, one JSON file. Everything the app knows lives in
//! `Studio`; everything the browser needs is assembled in `View`.

use serde::{Deserialize, Serialize};

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", now_ms())
}

/// 90 XP for level 2, then +45 per level.
pub fn xp_next(level: u32) -> u32 {
    90 + level.saturating_sub(1) * 45
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Traits {
    pub warmth: u8,
    pub playfulness: u8,
    pub curiosity: u8,
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            warmth: 72,
            playfulness: 58,
            curiosity: 66,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Companion {
    pub name: String,
    pub pronouns: String,
    /// friend | mentor | romantic
    pub mode: String,
    pub traits: Traits,
    pub level: u32,
    pub xp: u32,
    /// 0-100. Derived mood bands come from this, never set directly by the UI.
    pub affinity: i32,
}

impl Default for Companion {
    fn default() -> Self {
        Self {
            name: "Aria".to_string(),
            pronouns: "she/her".to_string(),
            mode: "friend".to_string(),
            traits: Traits::default(),
            level: 1,
            xp: 0,
            affinity: 42,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Memory {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub confidence: u8,
    pub at_ms: u64,
    pub snippet: String,
}

impl Memory {
    pub fn label(&self) -> &'static str {
        match self.kind.as_str() {
            "name" => "Name",
            "like" => "Loves",
            "dislike" => "Avoids",
            "job" => "Work",
            "pet" => "Pet",
            "home" => "Home",
            "goal" => "Goal",
            "feeling" => "Last feeling",
            _ => "Fact",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Msg {
    /// user | bot
    pub role: String,
    pub text: String,
    pub at_ms: u64,
    /// Which movement/expression the companion answered in.
    pub pose: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TopicStat {
    pub word: String,
    pub count: u32,
}

/// Where reply text may come from. `off` keeps the deterministic engine.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelSettings {
    /// off | ollama | openai
    pub provider: String,
    /// Plain-http loopback base, e.g. http://127.0.0.1:11434
    pub endpoint: String,
    pub model: String,
    pub timeout_ms: u64,
    pub temperature: f32,
}

impl Default for ModelSettings {
    fn default() -> Self {
        let model = std::env::var("REPIKACHAT_MODEL")
            .unwrap_or_default()
            .trim()
            .to_string();
        let provider = std::env::var("REPIKACHAT_MODEL_PROVIDER")
            .ok()
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| matches!(p.as_str(), "off" | "ollama" | "openai"))
            .unwrap_or_else(|| {
                if model.is_empty() {
                    "off".to_string()
                } else {
                    "ollama".to_string()
                }
            });
        let endpoint = std::env::var("REPIKACHAT_MODEL_ENDPOINT")
            .ok()
            .filter(|e| !e.trim().is_empty())
            .unwrap_or_else(|| {
                if provider == "openai" {
                    "http://127.0.0.1:1234".to_string()
                } else {
                    "http://127.0.0.1:11434".to_string()
                }
            });
        Self {
            provider,
            endpoint,
            model: if model.is_empty() {
                "llama3.2".to_string()
            } else {
                model
            },
            timeout_ms: 20_000,
            temperature: 0.85,
        }
    }
}

impl ModelSettings {
    pub fn sanitise(&mut self) {
        if !matches!(self.provider.as_str(), "off" | "ollama" | "openai") {
            self.provider = "off".to_string();
        }
        if self.endpoint.trim().is_empty() {
            self.endpoint = "http://127.0.0.1:11434".to_string();
        }
        if self.model.chars().count() > 80 {
            self.model = self.model.chars().take(80).collect();
        }
        self.timeout_ms = self.timeout_ms.clamp(2_000, 120_000);
        if !self.temperature.is_finite() {
            self.temperature = 0.85;
        }
        self.temperature = self.temperature.clamp(0.0, 1.5);
    }
}

/// Everything the app stores. One save file, one consumer, one companion.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Studio {
    #[serde(default = "default_version")]
    pub version: u32,
    pub onboarded: bool,
    pub companion: Companion,
    #[serde(default)]
    pub settings: ModelSettings,
    pub memories: Vec<Memory>,
    pub history: Vec<Msg>,
    pub topics: Vec<TopicStat>,
    /// Consumer messages accepted so far.
    pub messages: u32,
    pub created_ms: u64,
}

fn default_version() -> u32 {
    1
}

impl Default for Studio {
    fn default() -> Self {
        Self {
            version: 1,
            onboarded: false,
            companion: Companion::default(),
            settings: ModelSettings::default(),
            memories: Vec::new(),
            history: Vec::new(),
            topics: Vec::new(),
            messages: 0,
            created_ms: now_ms(),
        }
    }
}

impl Studio {
    pub fn add_xp(&mut self, amount: u32) {
        self.companion.xp += amount;
        loop {
            let need = xp_next(self.companion.level);
            if self.companion.xp < need {
                break;
            }
            self.companion.xp -= need;
            self.companion.level += 1;
            self.companion.affinity = (self.companion.affinity + 2).clamp(0, 100);
        }
    }

    /// Repairs anything a hand edit or an older file could have left broken.
    pub fn sanitise(&mut self) {
        self.version = 1;
        if self.companion.name.trim().is_empty() {
            self.companion.name = "Aria".to_string();
        }
        if self.companion.name.chars().count() > 24 {
            self.companion.name = self.companion.name.chars().take(24).collect();
        }
        if !matches!(
            self.companion.mode.as_str(),
            "friend" | "mentor" | "romantic"
        ) {
            self.companion.mode = "friend".to_string();
        }
        self.companion.affinity = self.companion.affinity.clamp(0, 100);
        self.settings.sanitise();
        if self.memories.len() > 140 {
            let cut = self.memories.len() - 140;
            self.memories.drain(0..cut);
        }
        if self.history.len() > 300 {
            let cut = self.history.len() - 300;
            self.history.drain(0..cut);
        }
        if self.topics.len() > 12 {
            self.topics.truncate(12);
        }
    }
}

pub fn mood_of(affinity: i32) -> (&'static str, &'static str) {
    match affinity {
        0..=24 => ("guarded", "Still reading you. Small talk first."),
        25..=49 => ("steady", "Comfortable, curious, a little careful."),
        50..=74 => ("warm", "Genuinely glad you are here."),
        _ => ("glowing", "Fully attached. Possibly showing off."),
    }
}

/// What the chat view needs. Memory is returned in full because the consumer
/// must be able to audit and delete every fact.
#[derive(Serialize)]
pub struct View {
    pub onboarded: bool,
    pub companion: Companion,
    pub settings: ModelSettings,
    pub brain: crate::llm::BrainView,
    pub memories: Vec<Memory>,
    pub history: Vec<Msg>,
    pub topics: Vec<TopicStat>,
    pub messages: u32,
    pub xp_next: u32,
    pub mood: String,
    pub mood_note: String,
    pub data_dir: String,
}
