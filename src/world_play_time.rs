//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.WorldPlayTime`.
use crate::{config, server_play_time};

/// Upstream `WorldPlayTime.MOD_ID`.
pub const MOD_ID: &str = "worldplaytime";

/// 1:1 with `WorldPlayTime.init()`: the config, then the server play times.
pub fn init() {
    config::init();
    server_play_time::load();
}
