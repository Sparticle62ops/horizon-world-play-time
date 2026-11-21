//! 1:1 port of the playtime attachment of upstream's mixins.
//!
//! `LevelSummaryMixin`/`IWithPlayTime` hold `playTimeTicks`, default `-1`,
//! on each level summary. `LevelStorageSourceMixin.onReturnLevelSummary`
//! reads the level's `stats` directory, sums `minecraft:custom`'s
//! `minecraft:play_time` across every `.json` stats file, and sets the
//! summary's play time only when the total is above zero, ignoring any
//! unreadable file silently. The render mixins (`WorldListEntryMixin`,
//! `OnlineServerEntryMixin`) need entry render hooks the pinned mod API has
//! not got; they are tracked with the render gap.
use serde_json::Value as Json;
use std::path::Path;

/// Upstream `LevelSummaryMixin.worldplaytime$playTimeTicks`' default.
pub const NO_PLAY_TIME: i32 = -1;

/// A summary as the mixins see it: the summary data plus the attached play
/// time (default `NO_PLAY_TIME`).
#[derive(Clone, Debug, PartialEq)]
pub struct LevelSummary {
    pub play_time_ticks: i32,
}

impl Default for LevelSummary {
    fn default() -> Self {
        Self {
            play_time_ticks: NO_PLAY_TIME,
        }
    }
}

impl LevelSummary {
    /// Upstream `IWithPlayTime.setPlayTimeTicks`.
    pub fn set_play_time_ticks(&mut self, play_time_ticks: i32) {
        self.play_time_ticks = play_time_ticks;
    }

    /// Upstream `IWithPlayTime.getPlayTimeTicks`.
    pub fn get_play_time_ticks(&self) -> i32 {
        self.play_time_ticks
    }
}

/// 1:1 with `LevelStorageSourceMixin.onReturnLevelSummary`: sum every stats
/// file's `minecraft:custom`/`minecraft:play_time` under the level's `stats`
/// directory, silently ignoring unreadable or wrong-shaped files, and set it
/// on the summary only when the total is above zero.
pub fn attach_play_time(summary: &mut LevelSummary, level_path: &Path) {
    let stats = match level_path.parent() {
        Some(parent) => parent.join("stats"),
        None => return,
    };
    let Ok(entries) = std::fs::read_dir(&stats) else {
        return;
    };
    let mut total_play_time = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Json>(&text) else {
            continue;
        };
        let play_time = json
            .get("stats")
            .and_then(|stats| stats.get("minecraft:custom"))
            .and_then(|custom| custom.get("minecraft:play_time"))
            .and_then(Json::as_i64);
        if let Some(play_time) = play_time {
            total_play_time += play_time as i32;
        }
    }
    if total_play_time > 0 {
        summary.set_play_time_ticks(total_play_time);
    }
}
