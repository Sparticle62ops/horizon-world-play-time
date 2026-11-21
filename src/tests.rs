//! Tests for the 1:1 core: config round-trip and fallbacks, color save
//! forms, server playtime persistence and guards, the tick handler's
//! autosave cadence, the formatter's two shapes, and the stats reader.
use super::color::Color;
use super::config;
use super::handler;
use super::level_summary;
use super::renderer;
use super::render_pos::{ServerEntryRenderPos, WorldEntryRenderPos};
use super::server_play_time;

#[test]
fn config_defaults_match_upstreams_field_initializers() {
    let config = config::Config::default();
    assert!(!config.show_server_play_time);
    assert_eq!(config.server_play_time_position, ServerEntryRenderPos::AfterName);
    assert_eq!(config.server_play_time_color, Color::new(128, 128, 128, 255));
    assert!(config.show_world_play_time);
    assert_eq!(config.world_play_time_position, WorldEntryRenderPos::TopRight);
    assert_eq!(config.world_play_time_color, Color::new(128, 128, 128, 255));
}

#[test]
fn config_round_trips_the_upstream_json_shape() {
    let dir = std::env::temp_dir().join("wpt-test-roundtrip");
    std::fs::remove_dir_all(&dir).ok();
    let path = dir.join("worldplaytime").join("config.json");
    let mut config = config::Config::default();
    config.show_server_play_time = true;
    config.server_play_time_position = ServerEntryRenderPos::Right;
    config.server_play_time_color = Color::new(255, 0, 0, 127);
    config.show_world_play_time = false;
    config.world_play_time_position = WorldEntryRenderPos::Left;
    config.world_play_time_color = Color::new(10, 20, 30, 200);
    config::set_active(config.clone());
    config::save_to(&path);
    let read_back = config::load(&path);
    assert_eq!(read_back, config);

    // the file keys are upstream's field names
    let text = std::fs::read_to_string(&path).unwrap();
    for key in [
        "showServerPlayTime",
        "serverPlayTimePosition",
        "serverPlayTimeColor",
        "showWorldPlayTime",
        "worldPlayTimePosition",
        "worldPlayTimeColor",
    ] {
        assert!(text.contains(&format!("\"{key}\"")), "{key} missing");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn config_read_falls_back_like_the_value_wrappers() {
    let dir = std::env::temp_dir().join("wpt-test-fallback");
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.json");
    // enums read case-insensitively and fall back on an unknown name; colors
    // fall back on an unparseable string; missing keys keep defaults
    std::fs::write(
        &path,
        r#"{"serverPlayTimePosition": "LEFT", "serverPlayTimeColor": "(1, 2, 3, -128)",
            "worldPlayTimePosition": "nonsense", "worldPlayTimeColor": "blue"}"#,
    )
    .unwrap();
    let config = config::load(&path);
    assert_eq!(config.server_play_time_position, ServerEntryRenderPos::Left);
    assert_eq!(config.server_play_time_color, Color::new(1, 2, 3, 255));
    assert_eq!(config.world_play_time_position, WorldEntryRenderPos::TopRight);
    assert_eq!(config.world_play_time_color, Color::new(128, 128, 128, 255));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn server_and_handler_and_init_work_together_under_temp_dirs() {
    // one combined test: the three games the temp dirs simulate share the
    // process env, so a parallel run would race the overrides
    let config_dir = std::env::temp_dir().join("wpt-test-env");
    let game_dir = std::env::temp_dir().join("wpt-test-env-game");
    std::fs::remove_dir_all(&config_dir).ok();
    std::fs::remove_dir_all(&game_dir).ok();
    unsafe {
        std::env::set_var("WPT_CONFIG_DIR", &config_dir);
        std::env::set_var("WPT_GAME_DIR", &game_dir);
    }

    // -- init: the missing config file is written with the defaults
    super::world_play_time::init();
    let path = config_dir.join("worldplaytime").join("config.json");
    assert!(path.exists());
    assert_eq!(config::active(), config::Config::default());

    // -- server play time: accumulate, persist, reload positive-only
    assert_eq!(server_play_time::get_play_time("1.2.3.4"), 0);
    server_play_time::on_play_tick("1.2.3.4");
    server_play_time::on_play_tick("1.2.3.4");
    server_play_time::on_play_tick("5.6.7.8");
    assert_eq!(server_play_time::get_play_time("1.2.3.4"), 2);
    assert_eq!(server_play_time::get_play_time("5.6.7.8"), 1);
    server_play_time::save();
    server_play_time::load();
    assert_eq!(server_play_time::get_play_time("1.2.3.4"), 2);
    assert_eq!(server_play_time::get_play_time("5.6.7.8"), 1);
    // a server driven back to 0 is dropped on the next load
    std::fs::write(
        server_play_time::data_file(),
        serde_json::json!({"1.2.3.4": 0, "5.6.7.8": 1}).to_string(),
    )
    .unwrap();
    server_play_time::load();
    assert_eq!(server_play_time::get_play_time("1.2.3.4"), 0);
    assert_eq!(server_play_time::get_play_time("5.6.7.8"), 1);

    // -- handler: one tick per connected tick, autosave every 6000
    handler::on_client_tick(None);
    assert_eq!(server_play_time::get_play_time("mc.example.net"), 0);
    for _ in 0..5999 {
        handler::on_client_tick(Some("mc.example.net"));
    }
    assert_eq!(server_play_time::get_play_time("mc.example.net"), 5999);
    handler::on_client_tick(Some("mc.example.net"));
    assert_eq!(server_play_time::get_play_time("mc.example.net"), 6000);
    assert!(server_play_time::data_file().exists());
    handler::on_client_tick(Some("mc.example.net"));
    assert_eq!(server_play_time::get_play_time("mc.example.net"), 6001);

    std::fs::remove_dir_all(&config_dir).ok();
    std::fs::remove_dir_all(&game_dir).ok();
    unsafe {
        std::env::remove_var("WPT_CONFIG_DIR");
        std::env::remove_var("WPT_GAME_DIR");
    }
}
#[test]
fn color_save_form_and_negative_alpha_match_upstream() {
    let color = Color::new(128, 128, 128, 255);
    assert_eq!(color.to_save_string(), "(128, 128, 128, 255)");
    // the saved regex matches whole: only exactly four fields parse
    assert_eq!(Color::from_save_string("(128, 128, 128, 255)"), Some(color));
    assert_eq!(Color::from_save_string("(1, 2, 3)"), None);
    assert_eq!(Color::from_save_string("blue"), None);
    // upstream's `a < 0 ? 127 - a : a` rule
    assert_eq!(
        Color::from_save_string("(1, 2, 3, -128)"),
        Some(Color::new(1, 2, 3, 255))
    );
    assert_eq!(
        Color::from_save_string("(1, 2, 3, -100)"),
        Some(Color::new(1, 2, 3, 227))
    );
}

#[test]
fn color_argb_round_trips() {
    let color = Color::from_argb(0xFF40A0FFu32 as i32);
    assert_eq!(color, Color::new(0x40, 0xA0, 0xFF, 0xFF));
    assert_eq!(color.to_argb(), 0xFF40A0FFu32 as i32);
}

#[test]
fn render_positions_round_trip_lowercase_and_read_case_insensitive() {
    assert_eq!(ServerEntryRenderPos::from_name("behind_count"), Some(ServerEntryRenderPos::BehindCount));
    assert_eq!(ServerEntryRenderPos::from_name("AFTER_NAME"), Some(ServerEntryRenderPos::AfterName));
    assert_eq!(ServerEntryRenderPos::BehindCount.to_json(), "behind_count");
    assert_eq!(WorldEntryRenderPos::from_name("TOP_RIGHT"), Some(WorldEntryRenderPos::TopRight));
    assert_eq!(WorldEntryRenderPos::Right.to_json(), "right");
    assert_eq!(WorldEntryRenderPos::from_name("nonsense"), None);
}





#[test]
fn renderer_formats_whole_hours_from_100_up_and_one_decimal_under() {
    assert_eq!(renderer::get_play_time_text(0), None);
    assert_eq!(renderer::get_play_time_text(-5), None);
    assert_eq!(renderer::get_play_time_text(20 * 3600), Some("1.0h".into()));
    assert_eq!(renderer::get_play_time_text(20 * 3600 * 3 / 10), Some("0.3h".into()));
    // 123.4 hours is also whole: the integer form covers 100 and up
    assert_eq!(
        renderer::get_play_time_text(20 * 3600 * 123 + 20 * 3600 * 4 / 10),
        Some("123h".into())
    );
    // from 100 up: the integer form
    assert_eq!(renderer::get_play_time_text(20 * 3600 * 100), Some("100h".into()));
    assert_eq!(renderer::get_play_time_text(20 * 3600 * 234), Some("234h".into()));
}

#[test]
fn renderer_whole_width_adds_the_icon_constant() {
    assert_eq!(renderer::get_whole_width(0, 40), None);
    assert_eq!(renderer::get_whole_width(20 * 3600, 40), Some(51));
}

#[test]
fn level_summary_defaults_to_no_play_time() {
    let summary = level_summary::LevelSummary::default();
    assert_eq!(summary.get_play_time_ticks(), -1);
    let mut summary = summary;
    summary.set_play_time_ticks(72000);
    assert_eq!(summary.get_play_time_ticks(), 72000);
}

#[test]
fn stats_reader_sums_across_files_and_ignores_bad_ones() {
    let level = std::env::temp_dir().join("wpt-test-level").join("world");
    std::fs::remove_dir_all(&level).ok();
    let stats = level.join("stats");
    std::fs::create_dir_all(&stats).unwrap();
    std::fs::write(
        stats.join("aaaaaaaa-1111-2222-3333-444444444444.json"),
        r#"{"stats": {"minecraft:custom": {"minecraft:play_time": 72000, "minecraft:mob_kills": 3}}}"#,
    )
    .unwrap();
    std::fs::write(
        stats.join("bbbbbbbb-1111-2222-3333-444444444444.json"),
        r#"{"stats": {"minecraft:custom": {"minecraft:play_time": 18000}}}"#,
    )
    .unwrap();
    // unreadable and wrong-shaped files are silently skipped, as upstream
    std::fs::write(stats.join("broken.json"), "{not json").unwrap();
    std::fs::write(stats.join("empty.json"), r#"{}"#).unwrap();

    // the glue passes the level's data path; the reader looks at its parent's
    // stats directory, 1:1 with upstream's `path.getParent().resolve("stats")`
    let mut summary = level_summary::LevelSummary::default();
    level_summary::attach_play_time(&mut summary, &level.join("level.dat"));
    assert_eq!(summary.get_play_time_ticks(), 90000);

    // a total of zero leaves the summary at its -1 default
    std::fs::remove_dir_all(&stats).unwrap();
    let mut fresh = level_summary::LevelSummary::default();
    level_summary::attach_play_time(&mut fresh, &level.join("level.dat"));
    assert_eq!(fresh.get_play_time_ticks(), -1);
    std::fs::remove_dir_all(&level).ok();
}


