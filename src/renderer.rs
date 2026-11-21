//! 1:1 port of upstream `me.khajiitos.worldplaytime.common.util.PlayTimeRenderer`,
//! formatter half.
//!
//! `getPlayTimeComponent` returns `null` at 0 ticks or below and otherwise
//! renders the `worldplaytime.format` string (`%sh`) over the hours: whole
//! hours from 100 up, one decimal under. The on-screen half (`getWholeWidth`
//! needs the font's width, `render` needs `GuiGraphics` and the icon blit)
//! waits on a render surface the pinned mod API has not got; the port keeps
//! the `+ 11` icon-and-gap constant and takes the font width as a parameter
//! for the game glue.

/// Upstream `getWholeWidth`'s `+ 11` (the 9x9 time icon plus its gap).
pub const INDICATOR_WIDTH_EXTRA: i32 = 11;

/// 1:1 with `getPlayTimeComponent`'s text, with the `worldplaytime.format`
/// wrapper applied: `Some("12.3h")`.
pub fn get_play_time_text(ticks: i32) -> Option<String> {
    if ticks <= 0 {
        return None;
    }
    let hours = (ticks as f64) / 20.0 / 3600.0;
    let body = if hours >= 100.0 {
        format!("{}", hours as i64)
    } else {
        format!("{:.1}", hours)
    };
    Some(format!("{body}h"))
}

/// 1:1 with `getWholeWidth`'s arithmetic (`font.width(component) + 11`),
/// with the font's text width passed in by the game glue.
pub fn get_whole_width(ticks: i32, text_width: i32) -> Option<i32> {
    get_play_time_text(ticks).map(|_| text_width + INDICATOR_WIDTH_EXTRA)
}
