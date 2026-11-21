//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.handler.EventHandlerCommon`.
//!
//! Upstream reads the connection and its `ServerData` in `onClientTick`; the
//! port receives the connected server's ip (the game glue's seam) and keeps
//! the exact flow: no connection or no server data is a no-op tick, one play
//! tick per otherwise, an autosave every 6000 play ticks, and a save on
//! leaving the game or the server.
use crate::server_play_time;

/// Upstream `EventHandlerCommon.playTicks`.
static PLAY_TICKS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub fn on_client_tick(connected_server: Option<&str>) {
    let Some(connected_server) = connected_server else {
        return;
    };
    server_play_time::on_play_tick(connected_server);
    let previous = PLAY_TICKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if previous + 1 >= 6000 {
        server_play_time::save_async();
        PLAY_TICKS.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn on_leaving_game() {
    server_play_time::save();
}

pub fn on_leave_server() {
    server_play_time::save();
}
