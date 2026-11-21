//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.config.ServerPlayTimeManager`.
//!
//! Upstream persists a Minecraft `CompoundTag` of server ip -> ticks at
//! `data/servers_playtime.dat` through `NbtIo`. The port keeps the same file
//! path and the same per-server integer keys, serialized as the equivalent
//! JSON object (the engine's NBT writer is a game seam the pinned API does
//! not expose), with the same rules: an empty map is never written, a read
//! keeps only positive tick counts, and a save or read failure is logged.
use crate::world_play_time::MOD_ID;
use serde_json::Value as Json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Upstream `ServerPlayTimeManager.file`: `data/servers_playtime.dat`,
/// rooted at the game directory, overridable for tests.
pub fn data_file() -> PathBuf {
    std::env::var_os("WPT_GAME_DIR")
        .map(|d| PathBuf::from(d).join("data").join("servers_playtime.dat"))
        .unwrap_or_else(|| PathBuf::from("data/servers_playtime.dat"))
}

static SERVER_PLAY_TIMES: Mutex<Option<HashMap<String, i32>>> = Mutex::new(None);

fn with_map<R>(f: impl FnOnce(&mut HashMap<String, i32>) -> R) -> R {
    let mut guard = SERVER_PLAY_TIMES.lock().unwrap();
    f(guard.get_or_insert_with(HashMap::new))
}

/// Upstream `save()`: nothing is written for an empty map; the parent
/// directory is created; one entry per server.
pub fn save() {
    let file = data_file();
    let map = with_map(|m| m.clone());
    if map.is_empty() {
        return;
    }
    if let Some(parent) = file.parent() {
        if !parent.is_dir() && std::fs::create_dir_all(parent).is_err() {
            log("Failed to create config/worldplaytime/ directory");
            return;
        }
    }
    let object: serde_json::Map<String, Json> = map
        .into_iter()
        .map(|(ip, ticks)| (ip, Json::from(ticks)))
        .collect();
    if let Err(e) = std::fs::write(&file, Json::Object(object).to_string()) {
        log("Failed to write to servers_playtime.dat");
        let _ = e;
    }
}

/// Upstream `load()`: a missing file returns silently; only positive tick
/// counts are kept (upstream's `if (playTime > 0)` guard).
pub fn load() {
    let map = with_map(|m| m.clear());
    let _ = map;
    let text = match std::fs::read_to_string(data_file()) {
        Ok(text) => text,
        Err(_) => return,
    };
    let Ok(object) = serde_json::from_str::<Json>(&text) else {
        return;
    };
    let Some(entries) = object.as_object() else {
        return;
    };
    with_map(|m| {
        for (ip, value) in entries {
            let ticks = value.as_i64().unwrap_or(0);
            if ticks > 0 {
                m.insert(ip.clone(), ticks as i32);
            }
        }
    });
}

/// Upstream `saveAsync()`: `CompletableFuture.runAsync` over `save`.
pub fn save_async() {
    std::thread::spawn(save);
}

/// Upstream `onPlayTick`: one more tick for that server.
pub fn on_play_tick(server_ip: &str) {
    with_map(|m| *m.entry(server_ip.to_string()).or_insert(0) += 1);
}

/// Upstream `getPlayTime`: 0 for an unknown server.
pub fn get_play_time(server_ip: &str) -> i32 {
    with_map(|m| m.get(server_ip).copied().unwrap_or(0))
}

fn log(message: &str) {
    eprintln!("[{MOD_ID}] {message}");
}
