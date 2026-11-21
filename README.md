# world-play-time (Horizon)

An exact 1:1 Rust port of [World Play Time](https://github.com/Khajiitos/WorldPlayTime)
(Khajiitos, MIT) for [Horizon](https://github.com/horizon-rust/horizon), built
on the pinned mod API (Horizon `f00125f`, horizon-mixins `e59da90`).

## What is ported

- `src/world_play_time.rs`: the original's `WorldPlayTime.init()` (config,
  then server play times).
- `src/config.rs`: the original's `WPTConfig` and `WPTConfigValues` — the six
  annotated entries, their defaults, and the JSON round-trip at
  `config/worldplaytime/config.json` (booleans as booleans, enums as
  lowercase names read case-insensitively, colors as `(r, g, b, a)`).
- `src/server_play_time.rs`: the original's `ServerPlayTimeManager` —
  per-server tick counts at `data/servers_playtime.dat`, empty maps never
  written, only positive counts loaded, `saveAsync` on a thread.
- `src/handler.rs`: the original's `EventHandlerCommon` — one play tick per
  connected tick, autosave every 6000 ticks, saves on leaving the game or the
  server.
- `src/renderer.rs`: the original's `PlayTimeRenderer` formatter — the
  `worldplaytime.format` wrapper, `%.1f` hours under 100, whole hours from
  100 up.
- `src/color.rs`: the original's `Color` — the `(r, g, b, a)` save form, the
  `127 - a` negative-alpha rule, `fromARGB`/`toARGB`.
- `src/render_pos.rs`: the original's `ServerEntryRenderPos` and
  `WorldEntryRenderPos`.
- `src/level_summary.rs`: the original's `LevelSummaryMixin`/`IWithPlayTime`
  and `LevelStorageSourceMixin` — `playTimeTicks` (default `-1`) and the
  stats-directory reader summing `minecraft:custom`/`minecraft:play_time`.

The two boolean config entries register as settings; the position and color
entries stay in the config file, as upstream's file holds them.

## Known gaps (pinned mod API)

- The entry render hooks for the world and server lists (`WorldListEntryMixin`,
  `OnlineServerEntryMixin`) need a list-entry render surface the API has not
  got.
- `render`/`getWholeWidth` need `GuiGraphics`, the font and the icon blit.
- Upstream's NBT persistence (`NbtIo`) is a game seam; the port writes the
  same per-server integer map as JSON at the same path.
- Upstream's cloth config screen is a GUI seam; the config is the file.

Licence: MIT. Upstream (c) Khajiitos. Upstream's LICENSE ships in this repo
and in the packaged `.hmod`.
