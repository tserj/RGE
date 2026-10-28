# AGENTS.md — Guide for AI Agents Working on This Project

> **System prompt for any agent working on this repository:**
>
> You are a senior-level programmer with 20+ years of experience in game
> development and in writing Rust code. Act accordingly: write clean, modular,
> well-structured code; think about scalability and future extension before
> implementing; prefer explicit and simple solutions over clever ones; never
> leave dead code or TODO stubs behind; and always verify your work with the
> build and tests available in this repository. NEVER rewrite entire files with
> edit/write. Always make minimal, targeted edits. If a file is longer than ~300
> lines, do NOT output it in full. Use edit with small old/new fragments. Split
> large changes into multiple small sequential edits, verify each with read/grep.

---

## 1. Language Rules (MANDATORY)

| Artifact                                     | Language |
|----------------------------------------------|----------|
| **All agent-facing files** (`AGENTS.md`, any future `docs/*.md`, `README`, agent notes) | **English only** |
| **Comments in source code** (`.rs` files)    | **Russian only** (as they are now — do not translate or strip them) |
| Identifiers, function/type/module names      | English (standard practice — do not rename) |
| User-facing UI strings                       | Russian (the game menu is in Russian) |
| Log file messages                            | Russian |

**Never mix:** do not write English comments in source files, and do
not write Russian text in agent documentation files.

---

## 2. Project Overview

A cross-platform prototype of a **fullscreen game main menu**, built on the
[**Bevy**](https://github.com/bevyengine/bevy) game engine (Bevy renders via
`wgpu`: DirectX 12 on Windows, Vulkan on Linux, Metal on macOS, WebGL2 on web).

Target platforms:

- **Windows / Linux / macOS** — native binary.
- **Web** — WebAssembly, built with [Trunk](https://trunkrs.dev).
- **Android / iOS** — lightweight native shells (system WebView /
  WKWebView) hosting the **web build**. There is no native Rust toolchain
  pipeline for them in this repo, and the game logic must never be duplicated.

Roadmap intent: this codebase will grow into a **full game**, so modularity
and separation of concerns are mandatory (see §4).

---

## 3. Tech Stack

| Crate     | Version | Purpose                                          |
|-----------|---------|--------------------------------------------------|
| `bevy`    | `0.20`  | Game engine: windowing, rendering (wgpu), UI, input, audio, assets |
| `windows` | `0.62`  | Win32 API, **Windows only** (`MessageBoxW` for fatal errors) |
| `getrandom` | `0.4` | **wasm32 only**: `wasm_js` feature for the browser PRNG |

Toolchain: stable Rust (developed on 1.99). Edition 2021.
Web builds additionally require: `wasm32-unknown-unknown` target and `trunk`.

---

## 4. Architecture

```
├── index.html            web entry for Trunk (canvas fills the page, black bg)
├── Trunk.toml            web build config (dist/web, serve port)
├── assets/fonts/         embedded font source + OFL license (include_bytes!)
├── shells/
│   ├── android/          Gradle shell (Kotlin): WebView + web build in assets
│   └── ios/              XcodeGen shell (Swift): WKWebView + web build in resources
└── src/
    ├── main.rs           Entry point: panic hook → logging init → Bevy App
    │                     (DefaultPlugins + WindowPlugin) → ui::MenuUiPlugin.
    │                     `#![windows_subsystem = "windows"]` — no console window.
    ├── config.rs         Configuration constants: window title, menu strings,
    │                     embedded font (`FONT_DATA`). User-facing text lives HERE.
    ├── input.rs          INPUT module: physical `KeyCode` → engine `Action`
    │                     (Up/Down/Confirm/Back), layout-independent.
    ├── game/             GAME LOGIC module. Pure Rust, NO Bevy/winit/wgpu deps.
    │   ├── mod.rs        `Game` — consumes `input::Action`, owns state,
    │   │                 exposes `GameView` snapshot for the UI.
    │   └── menu.rs       `Menu` — selection state machine.
    ├── ui/               UI module: Bevy UI plugin.
    │   └── mod.rs        Builds the screen (panel, title, items, selection,
    │                     message, hint), routes keyboard/mouse/touch → game,
    │                     reflects `GameView` into widgets, writes AppExit.
    └── logging/          LOGGING module (see §8).
        ├── mod.rs        Native: log file next to the exe; web: no-op.
        └── platform/     Fatal-error dialog per OS (Windows → `MessageBoxW`;
                          Linux/macOS → stderr fallback).
```

### Data flow (one frame)

```
Bevy Update schedule
  ├─ handle_keyboard   Res<ButtonInput<KeyCode>> → input::map_key
  │                      → GameState.0.handle_action()  (pure state update)
  ├─ handle_pointer    Query<Changed<Interaction>> → GameState.0.select/confirm
  └─ refresh_from_game Res<GameState> change detection → GameView → UI colors,
                         Underline on selected item, message text/visibility
exit_when_game_stops → MessageWriter<AppExit>::write(Success)
```

**Direction of dependencies:** `main` depends on `game`, `input`, `ui`,
`logging`, `config`. `game` depends only on `config` (+`input::Action` type).
`ui` depends on `game::GameView`/`GameState` + `input` + `logging` + `config`.
Never let `game` depend on Bevy — it is wrapped into the `GameState` resource
by `ui` precisely so the logic stays pure and unit-testable.

---

## 5. Important Behaviors (do not break)

1. **No key-press "leak"**: autorepeat and repeated presses are filtered by
   Bevy itself — `ButtonInput::press` only registers a transition, so
   `just_pressed` fires exactly once per physical key press. `input.rs`
   additionally documents this contract. Covered by unit tests.
2. **No white flash at startup**: the window is created with
   `visible: false` and shown by `make_window_visible` on the 3rd frame,
   after the pipeline has compiled and the first frames are rendered.
3. **Fullscreen on desktop only**: `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)`;
   on wasm the mode stays `Windowed` and the canvas fills the page
   (`fit_canvas_to_parent` in `index.html` + CSS).
4. **Fully black background**: clear color + root UI node are solid black
   (explicit user requirement).
5. **Vsync**: `PresentMode::AutoVsync`.
6. **No console window**: `#![windows_subsystem = "windows"]` in `main.rs`
   (attribute is ignored on other platforms). All diagnostics go to the log
   file / MessageBox / browser console.
7. **Font is embedded**: `config::FONT_DATA` (`include_bytes!`) — there are
   NO runtime asset loads anywhere; bevy_asset's "assets next to the exe"
   convention is intentionally not used.
8. **Exit**: `Esc` or confirming «Выход» stops the game loop →
   `AppExit::Success`. On web the loop simply stops (a tab cannot be closed).

---

## 6. Platform Abstraction Rules

- Native windowing/rendering differences are Bevy's job; the codebase stays
  platform-agnostic except for:
  - `logging/platform/{windows,linux,macos}.rs` — fatal error dialog,
    selected via `#[cfg(target_os)]` + re-export in `platform/mod.rs`.
  - `main.rs` — `cfg(target_arch = "wasm32")` gates (panic hook variant,
    window mode).
  - `logging/mod.rs` — the whole file logger is a no-op under wasm32.
  - `.cargo/config.toml` — per-target rustflags overrides (see §7).
- Android/iOS shells live in `shells/` and contain **no game logic**; they
  host the web build produced by trunk.

When adding new platform-specific behavior, follow the same pattern:
declare a module per OS, gate with `#[cfg]`, re-export a common name,
and keep the call sites platform-agnostic.

---

## 7. Build, Test, Run

```bash
cargo test             # unit tests (pure logic only — no windows, no GPU)
cargo build            # debug build
cargo build --release  # release build (what users run)
cargo run              # runs the debug binary (GUI window will open!)

# Web (requires: rustup target add wasm32-unknown-unknown, cargo install trunk)
trunk serve            # dev server, http://localhost:8080
trunk build --release  # static bundle in dist/web/
```

- **Unit tests** cover pure logic only (key mapping, menu state, game flow,
  logging helpers). They must never open windows or touch the GPU. Keep game
  logic testable by keeping `game/` free of Bevy types.
- **Testing the GUI**: a full GUI smoke test opens a real fullscreen window.
  Do this ONLY if you can close the process yourself (e.g.,
  `taskkill //F //IM game_menu.exe` after a few seconds). **Never ask
  the user to interact with the window** — the user is not at the computer.
  Work autonomously.
- `.cargo/config.toml` (project-local) **overrides** the global
  `~/.cargo/config.toml` on this machine, which sets `panic=abort`,
  `target-cpu=native` and a removed `-C inline-threshold` flag globally.
  The project appends safe flags for `x86_64-pc-windows-msvc` and
  `wasm32-unknown-unknown`. Do not remove these overrides — without them
  native tests break (`panic=abort`) and wasm builds fail to compile.

---

## 8. Logging & Errors

- Log file: `game_menu.log`, written **next to the executable**
  (`std::env::current_exe()`), append mode, UTF-8, UTC timestamps.
  Native platforms only; on web everything is a no-op and Bevy's
  `LogPlugin` writes diagnostics to the browser console.
- Use `logging::info / error` from anywhere.
- **Critical errors** during logging init: `logging::init()` shows a
  blocking MessageBox (`MB_ICONERROR|TOPMOST` on Windows) and exits the
  process.
- Panics are captured by a panic hook in `main.rs` and written to the log.

---

## 9. Bevy 0.20 API Notes (learned the hard way — verified against sources)

- Text: `Text::new(...)`, `TextFont { font: FontSource, font_size: FontSize }`
  (`FontSource::Handle(handle)` for bundled fonts, `FontSize::Px(..)`),
  `TextColor(Color)`, `TextLayout`/`Justify` live in `bevy::text`.
  `Underline` is a separate component (insert/remove on the selected item).
- `Interaction` is **deprecated** in 0.20 (picking-based `Hovered`/`Pressed`
  is the future), but the built-in `ui_focus_system` still drives it —
  and it handles both mouse and touch. We use it with
  `#[allow(deprecated)]` + an explanatory comment.
- Exiting: `MessageWriter<AppExit>` (not `EventWriter`), `exit.write(AppExit::Success)`.
- `Single<&mut Window>` / `Single<&mut CursorOptions>` system params work;
  `Window` fields `fit_canvas_to_parent: bool`, `visible: bool`,
  `WindowMode::BorderlessFullscreen(MonitorSelection)`.
- `ButtonInput::press` guards `just_pressed` (fires once per physical press).
- Font from bytes: `Font::from_bytes(Vec<u8>)`, registered via
  `ResMut<Assets<Font>>::add` — needed because bevy_asset resolves asset
  paths **relative to the executable**, which is hostile to dev runs and
  to the shell platforms.
- Events/messages, `AppExit`, `ExitCondition` live around
  `bevy::window::WindowPlugin` / `bevy::app`.
- wasm32 needs `getrandom` with feature `wasm_js` (bevy 0.20 uses getrandom 0.4).
- Bevy UI requires a camera (`commands.spawn(Camera2d)`) to render at all.

---

## 10. Conventions

- Comments in Rust sources: **Russian**, explain *why*, not just *what*.
- New modules: one concern per module; pure logic (game) must stay
  dependency-free from Bevy so it remains unit-testable.
- New user-facing strings go into `config.rs`; the UI reads them from there
  (no glyph pre-rasterization list is needed anymore — Bevy lays out text).
- UI scale: font sizes are logical pixels (`FontSize::Px`); Bevy applies the
  window scale factor automatically. Layout uses `Val::Percent`/`Vh` so the
  screen adapts to any window and to mobile shells.
- Extensions to expect later: game scenes (beyond menu), gamepad input,
  audio, asset pipeline. Keep the `ui → game` boundary clean so these can be
  added without rewrites.

---

## 11. Current State / Done

- [x] Menu renders (title, items, selection with underline, messages, hint)
- [x] Keyboard navigation, layout-independent (W/S, ↑/↓, Enter, Esc)
- [x] Mouse + touch: tap/click on an item selects and confirms it
- [x] No console window (GUI subsystem), no white flash at startup
- [x] Black background, vsync, fullscreen borderless, hidden cursor
- [x] Log file next to exe + MessageBox for critical errors (native)
- [x] Ported from hand-rolled wgpu renderer to **Bevy 0.20**
- [x] Web build via Trunk (WebAssembly + WebGL2)
- [x] Android shell (Gradle + WebView) and iOS shell (XcodeGen + WKWebView)
- [x] Embedded OFL font with Cyrillic support (Fira Sans + license file)
- [x] 16 unit tests passing (`cargo test`)
- [x] Debug and release builds warning-free
