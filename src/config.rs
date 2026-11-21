//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.config`
//! (`WPTConfigValues`, `WPTConfig`).
//!
//! Upstream keeps six annotated static entries and round-trips them through
//! Gson at `config/worldplaytime/config.json`: booleans are booleans, enums
//! write their lowercase name and read case-insensitively (falling back to
//! the default), colors write `(r, g, b, a)` and read with `Color.fromString`
//! (falling back to the default). `init()` writes the defaults when the file
//! is missing and loads it otherwise. The port keeps the same file, the same
//! keys and the same fallbacks, hand-rolled over `serde_json::Value` so the
//! JSON shapes stay identical.
use crate::color::Color;
use crate::render_pos::{ServerEntryRenderPos, WorldEntryRenderPos};
use crate::world_play_time::MOD_ID;
use serde_json::Value as Json;
use std::path::PathBuf;
use std::sync::Mutex;

/// Upstream `WPTConfig.file`: `config/worldplaytime/config.json`, overridable
/// for tests.
pub fn config_file() -> PathBuf {
    std::env::var_os("WPT_CONFIG_DIR")
        .map(|d| PathBuf::from(d).join("worldplaytime").join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config/worldplaytime/config.json"))
}

/// The original's six entries, same keys as upstream's fields (which are also
/// the JSON file's keys).
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub show_server_play_time: bool,
    pub server_play_time_position: ServerEntryRenderPos,
    pub server_play_time_color: Color,
    pub show_world_play_time: bool,
    pub world_play_time_position: WorldEntryRenderPos,
    pub world_play_time_color: Color,
}

impl Default for Config {
    /// Upstream's field initializers.
    fn default() -> Self {
        Self {
            show_server_play_time: false,
            server_play_time_position: ServerEntryRenderPos::DEFAULT,
            server_play_time_color: Color::new(128, 128, 128, 255),
            show_world_play_time: true,
            world_play_time_position: WorldEntryRenderPos::DEFAULT,
            world_play_time_color: Color::new(128, 128, 128, 255),
        }
    }
}

/// The live config cells, 1:1 with upstream's statics.
static ACTIVE: Mutex<Option<Config>> = Mutex::new(None);

pub fn set_active(config: Config) {
    *ACTIVE.lock().unwrap() = Some(config);
}

pub fn active() -> Config {
    ACTIVE.lock().unwrap().clone().unwrap_or_default()
}

/// Syncs a boolean setting into the live cell (the mod API's Options > Mods
/// page holds the two boolean entries; the rest stay file-held).
pub fn sync_bool(key: &str, value: bool) {
    let mut guard = ACTIVE.lock().unwrap();
    let config = guard.get_or_insert_with(Config::default);
    if key == "showServerPlayTime" {
        config.show_server_play_time = value;
    } else if key == "showWorldPlayTime" {
        config.show_world_play_time = value;
    }
}

/// Upstream `WPTConfig.init()`: write the defaults when the file is missing,
/// load it otherwise (a broken file is logged and keeps the defaults).
pub fn init() {
    if !config_file().exists() {
        save_to(&config_file());
        set_active(Config::default());
    } else {
        set_active(load(&config_file()));
    }
}

/// Upstream `WPTConfig.save()`: the parent directory is created, each entry
/// written under its field name.
pub fn save_to(path: &std::path::Path) {
    let config = active();
    let object = serde_json::json!({
        "showServerPlayTime": config.show_server_play_time,
        "serverPlayTimePosition": config.server_play_time_position.to_json(),
        "serverPlayTimeColor": config.server_play_time_color.to_save_string(),
        "showWorldPlayTime": config.show_world_play_time,
        "worldPlayTimePosition": config.world_play_time_position.to_json(),
        "worldPlayTimeColor": config.world_play_time_color.to_save_string(),
    });
    if let Some(parent) = path.parent() {
        if !parent.is_dir() && std::fs::create_dir_all(parent).is_err() {
            log(&format!("Failed to create the {} directory", parent.display()));
            return;
        }
    }
    if let Err(e) = std::fs::write(
        path,
        serde_json::to_string_pretty(&object).expect("json object serializes"),
    ) {
        log(&format!("Failed to save the World Play Time config: {e}"));
    }
}

/// Upstream `WPTConfig.load()`: missing file keeps the defaults; each entry
/// is read only when its key is present, each reader falling back to the
/// entry's default on a wrong shape.
pub fn load(path: &std::path::Path) -> Config {
    let mut config = Config::default();
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            log(&format!("Failed to read the World Play Time config: {e}"));
            return config;
        }
    };
    let Ok(object) = serde_json::from_str::<Json>(&text) else {
        return config;
    };

    if let Some(value) = object.get("showServerPlayTime").and_then(Json::as_bool) {
        config.show_server_play_time = value;
    }
    if let Some(name) = object.get("serverPlayTimePosition").and_then(Json::as_str) {
        if let Some(pos) = ServerEntryRenderPos::from_name(name) {
            config.server_play_time_position = pos;
        }
    }
    if let Some(string) = object.get("serverPlayTimeColor").and_then(Json::as_str) {
        if let Some(color) = Color::from_save_string(string) {
            config.server_play_time_color = color;
        }
    }
    if let Some(value) = object.get("showWorldPlayTime").and_then(Json::as_bool) {
        config.show_world_play_time = value;
    }
    if let Some(name) = object.get("worldPlayTimePosition").and_then(Json::as_str) {
        if let Some(pos) = WorldEntryRenderPos::from_name(name) {
            config.world_play_time_position = pos;
        }
    }
    if let Some(string) = object.get("worldPlayTimeColor").and_then(Json::as_str) {
        if let Some(color) = Color::from_save_string(string) {
            config.world_play_time_color = color;
        }
    }
    config
}

fn log(message: &str) {
    eprintln!("[{MOD_ID}] {message}");
}
