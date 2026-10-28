# RGE — Rust Game Engine (Bevy)

Initial vibecoded prototype of a game, written in Rust on the
[**Bevy**](https://github.com/bevyengine/bevy) engine.

Current state: a fullscreen **game main menu** that builds and runs on
**five platforms**: Windows, Linux, macOS (native), Web (WebAssembly) and
Android/iOS (native WebView shells around the web build).

## Engine

The game is built on [Bevy](https://github.com/bevyengine/bevy) — a
refreshingly simple, data-driven game engine in Rust (ECS, plugin
architecture, built on `wgpu`: DirectX 12 / Vulkan / Metal / WebGL2).
Rendering, windowing, UI and input all go through Bevy's `DefaultPlugins`;
the game itself contains only menu logic and UI composition.

## Features

- Fullscreen borderless window (desktop) with vsync; canvas-filling page (web)
- Bevy UI screen: title, menu items, gold selection with underline,
  action message, control hints — all Russian UI strings in `config.rs`
- Layout-independent keyboard navigation (`KeyCode` physical keys:
  W/S, ↑/↓, Enter, Esc work on both Latin and Russian layouts)
- Mouse and touch: click/tap a menu item to select and confirm
- Clean startup: no console window (GUI subsystem), window shown after the
  first frames (no white flash)
- File logging (`game_menu.log` next to the executable, native platforms)
  and native MessageBox for critical errors (Windows)
- Embedded font (Fira Sans, OFL license included) — no asset loading at all
- **16 unit tests** covering input mapping, menu/game logic and logging
  (pure logic, no GPU, no windows)

## Platforms

| Platform | How | Build command |
|---|---|---|
| Windows | native (DirectX 12 via wgpu) | `cargo run --release` |
| Linux | native (Vulkan via wgpu) | `cargo run --release` |
| macOS | native (Metal via wgpu) | `cargo run --release` |
| Web | WebAssembly + WebGL2, served/bundled by [Trunk](https://trunkrs.dev) | `trunk serve` / `scripts/build_web_release.sh` |
| Android | native Gradle app (Kotlin) with a system `WebView` hosting the web build | see below |
| iOS | native Xcode app (Swift) with a `WKWebView` hosting the web build | see below |

Bevy does not require a separate codebase per platform: the same Rust code
compiles everywhere. Android and iOS do not have a supported native Rust
toolchain story here, so per the project decision they use **lightweight
native shells** (a WebView in a minimal app) that host the **web build** —
the game logic is written once and never duplicated.

## Size vs performance policy

Release builds are configured **per platform** (see `.cargo/config.toml`
and `Cargo.toml`):

- **Native (Windows/Linux/macOS) — maximum performance**: `opt-level=3`,
  `codegen-units=1`, fat LTO. Debug info is stripped (no runtime impact).
- **Web (wasm32) — minimum size**: `opt-level=z`, same LTO, stripped,
  plus a `wasm-opt -Oz` pass (Binaryen) and pre-compressed `.br` copies
  (Brotli) for serving. The release web bundle is ~16 MB on disk and
  **~3.5 MB over the wire** with Brotli (was 51 MB with default Bevy
  features and no post-processing).
  The used Wasm features (bulk-memory, sign-ext, nontrapping-float-to-int,
  reference-types) are baseline in all modern browsers — Chrome 75+,
  Firefox 79+, Safari 15+ (hence the iOS 15 deployment target).
- Bevy is compiled with a **manual feature set** (`default-features = false`):
  only UI + render + winit — no 3D, audio codecs, gltf/texture loaders,
  scene serialization or picking. This alone cuts the web binary several times.

## Getting started

Prerequisites: [Rust](https://rustup.rs) (latest stable), Visual Studio
C++ Build Tools (Windows); for web builds — [Trunk](https://trunkrs.dev)
(`cargo install --locked trunk`), the `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`), [wasm-opt](https://github.com/WebAssembly/binaryen)
(Binaryen) and Node.js 18+ (Brotli compression + local test server).

```bash
cargo test          # unit tests (pure logic)
cargo run           # native run (debug); release: cargo run --release
```

### Web

```bash
trunk serve                      # dev server on http://localhost:8080
scripts/build_web_release.sh     # min-size release bundle in dist/web/ (~16 MB)
node scripts/serve_web.mjs       # test server on :8088, serves Brotli-compressed files
```

The release script also emits pre-compressed `.br` copies of all files
(Brotli). The test server serves them with `Content-Encoding: br` when the
client announces support — browsers decompress transparently, no extra
client-side work is needed. In production, configure your host the same way
(nginx: `brotli_static on;`, most CDNs do this automatically).
```

### Android

1. Build the min-size web bundle: `scripts/build_web_release.sh`
2. Copy it into the shell assets:
   `cp -r dist/web/* shells/android/app/src/main/assets/web/`
   (create the `web/` directory first; dev alternative — a `trunk serve`
   URL is prepared in `MainActivity.kt`)
3. Open `shells/android/` in Android Studio (or run `gradle assembleDebug`)
   and deploy `app-debug.apk` to a device/emulator.

The shell is a single-activity app (`MainActivity.kt`): a fullscreen
hardware-accelerated `WebView` loading `file:///android_asset/web/index.html`.
`INTERNET` permission is only needed for the dev `trunk serve` mode.

### iOS (build on macOS)

1. Build the min-size web bundle: `scripts/build_web_release.sh`
2. Copy it into the shell resources:
   `mkdir -p shells/ios/Resources/web && cp -r dist/web/* shells/ios/Resources/web/`
3. Generate and open the Xcode project:
   `brew install xcodegen && cd shells/ios && xcodegen && open GameMenu.xcodeproj`
4. Run on a simulator/device.

The shell is a single-file app (`Sources/App.swift`): a fullscreen
`WKWebView` loading the bundled `web/index.html`.

## Architecture

```
├── index.html            web entry for Trunk (canvas fills the page, black bg)
├── Trunk.toml            web build config (dist/web, serve port)
├── assets/fonts/         embedded font source + OFL license (include_bytes!)
├── shells/
│   ├── android/          Gradle shell: WebView + web build in assets
│   └── ios/              XcodeGen shell: WKWebView + web build in resources
└── src/
    ├── main.rs           entry: panic hook, logging init, Bevy App + WindowPlugin
    ├── config.rs         window/UI configuration constants, embedded font
    ├── input.rs          input module: physical key -> Action mapping
    ├── game/             PURE game logic (menu now, scenes later), no Bevy deps
    │   ├── mod.rs        `Game` + `GameView` snapshot for the UI
    │   └── menu.rs       `Menu` selection state machine
    ├── ui/               Bevy UI plugin: build screen, route input -> game,
    │                     reflect GameView -> widgets, exit handling
    └── logging/          file logger (native) + platform fatal-error dialog;
                          no-op on web (Bevy LogPlugin writes to the console)
```

Direction of dependencies: `main → {ui, game, input, logging, config}`;
`game` never depends on Bevy (it is wrapped into a `GameState` resource by
`ui`) — this keeps game logic unit-testable without a GPU or a window.

See `AGENTS.md` for contributor/agent guidance.
