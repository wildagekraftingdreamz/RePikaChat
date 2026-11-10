//! One JSON save file, written atomically, plus the view builder.

use std::fs;
use std::path::PathBuf;

use crate::model::*;

pub struct Store {
    pub studio: Studio,
    pub path: PathBuf,
    pub dir: PathBuf,
}

impl Store {
    pub fn load() -> Store {
        let path = data_file();
        let dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let studio = fs::read_to_string(&path)
            .ok()
            .map(|raw| decode(&raw))
            .unwrap_or_default();
        Store { studio, path, dir }
    }

    /// Writes to a sibling temp file first, so a crash mid-write cannot leave a
    /// half-saved memory file behind.
    pub fn save(&self) {
        let _ = fs::create_dir_all(&self.dir);
        let Ok(raw) = serde_json::to_string_pretty(&self.studio) else {
            return;
        };
        let tmp = self.path.with_extension("json.tmp");
        if fs::write(&tmp, raw).is_ok() {
            let _ = fs::rename(&tmp, &self.path);
        }
    }

    pub fn view(&self) -> View {
        let (mood, mood_note) = mood_of(self.studio.companion.affinity);
        View {
            onboarded: self.studio.onboarded,
            companion: self.studio.companion.clone(),
            settings: self.studio.settings.clone(),
            brain: crate::llm::brain_view(&self.studio.settings),
            memories: self.studio.memories.clone(),
            history: self.studio.history.clone(),
            topics: self.studio.topics.clone(),
            messages: self.studio.messages,
            xp_next: xp_next(self.studio.companion.level),
            mood: mood.to_string(),
            mood_note: mood_note.to_string(),
            data_dir: self.dir.display().to_string(),
        }
    }
}

fn decode(raw: &str) -> Studio {
    match serde_json::from_str::<Studio>(raw) {
        Ok(mut studio) => {
            studio.sanitise();
            studio
        }
        Err(_) => Studio::default(),
    }
}

/// `REPIKACHAT_DATA` is either a directory or a `.json` file path.
pub fn data_file() -> PathBuf {
    if let Ok(raw) = std::env::var("REPIKACHAT_DATA") {
        let p = PathBuf::from(raw);
        if p.extension().map(|e| e == "json").unwrap_or(false) {
            return p;
        }
        return p.join("repikachat.json");
    }
    data_dir().join("repikachat.json")
}

pub fn data_dir() -> PathBuf {
    if let Ok(raw) = std::env::var("REPIKACHAT_DATA") {
        let p = PathBuf::from(raw);
        if p.extension().map(|e| e == "json").unwrap_or(false) {
            return p.parent().map(|x| x.to_path_buf()).unwrap_or(p);
        }
        return p;
    }
    if cfg!(windows) {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.trim().is_empty() {
                return PathBuf::from(local).join("RePikaChat");
            }
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("repikachat");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/repikachat");
    }
    PathBuf::from("data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_corrupt_save_falls_back_to_a_fresh_studio() {
        let studio = decode("{ not json");
        assert_eq!(studio.companion.name, "Aria");
        assert!(studio.history.is_empty());
    }

    #[test]
    fn an_old_save_is_trimmed_and_repaired() {
        let mut studio = Studio::default();
        studio.companion.name = "   ".to_string();
        studio.companion.mode = "enemy".to_string();
        studio.companion.affinity = 400;
        let raw = serde_json::to_string(&studio).unwrap();
        let fixed = decode(&raw);
        assert_eq!(fixed.companion.name, "Aria");
        assert_eq!(fixed.companion.mode, "friend");
        assert_eq!(fixed.companion.affinity, 100);
    }

    #[test]
    fn the_view_reports_mood_and_level_goal() {
        let mut studio = Studio::default();
        studio.companion.affinity = 60;
        let store = Store {
            studio,
            path: PathBuf::from("unused.json"),
            dir: PathBuf::from("."),
        };
        let view = store.view();
        assert_eq!(view.mood, "warm");
        assert_eq!(view.xp_next, 90);
    }
}
