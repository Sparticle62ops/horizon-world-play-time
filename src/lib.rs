//! World Play Time for Horizon: the mod API wiring of the 1:1 core.
//!
//! The 1:1 logic lives beside the entry: `config` (the original's annotated
//! `WPTConfig` with its six `WPTConfigValues` entries and its JSON file at
//! `config/worldplaytime/config.json`), `server_play_time` (the original's
//! `ServerPlayTimeManager`: per-server tick counts, autosaved), `handler`
//! (the original's `EventHandlerCommon`: one tick per connected tick, save
//! every 6000 ticks and on leaving), `renderer` (the original's
//! `PlayTimeRenderer` formatter), `color` and `render_pos` (the original's
//! `Color` and render-position enums) and `level_summary` (the original's
//! `LevelStorageSourceMixin` stats reader).
//!
//! The two boolean entries are registered as settings so Horizon's
//! Options > Mods page holds them, same defaults as upstream; the position
//! and color entries have no matching setting kind in the pinned API, so
//! they live in the config file exactly as upstream's file holds them.
//!
//! Gaps, tracked: the on-screen render half (the entry render hooks for the
//! world and server lists, the icon blit and the font width) needs a render
//! and list-entry surface the pinned mod API has not got.
use mod_api::{ClientSetupEvent, ClientHost, RegistrationContext, Registrar, SettingSpec};

mod color;
mod config;
mod handler;
mod level_summary;
mod renderer;
mod render_pos;
mod server_play_time;
mod world_play_time;

/// The original's boolean config fields registered as settings, same
/// defaults as the annotated config file: (key, label, default). The keys
/// are upstream's field names, which are also the JSON file's keys.
const SETTINGS: &[(&str, &str, bool)] = &[
    ("showServerPlayTime", "worldplaytime.config.showServerPlayTime", false),
    ("showWorldPlayTime", "worldplaytime.config.showWorldPlayTime", true),
];

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// The original's boolean config fields registered as settings, same
    /// defaults.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        for &(key, label, default) in SETTINGS {
            let _ = registrar.register_setting(SettingSpec::toggle(key, label, default));
        }
    }

    /// 1:1 of the original's mod init (`WorldPlayTime.init()`): load the
    /// config (defaults written when missing, as `WPTConfig.init()` keeps),
    /// load the server play times, then sync the boolean settings into the
    /// live config cells.
    fn client_setup(&self, host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        world_play_time::init();
        for (key, _, default) in SETTINGS {
            let value = host.setting_bool(key).unwrap_or(*default);
            config::sync_bool(key, value);
        }
    }
}
mod tests;
