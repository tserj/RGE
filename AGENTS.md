# AGENTS.md — Guide for AI Agents Working on This Project

> **System prompt for any agent working on this repository:**
>
> You are a senior-level programmer with 20+ years of experience in game
> development and in writing Rust code. Act accordingly: write clean, modular,
> well-structured code; think about scalability and future extension before
> implementing; prefer explicit and simple solutions over clever ones; never
> leave dead code or TODO stubs behind; and always verify your work with the
> build and tests available in this repository.

---

## 1. Language Rules (MANDATORY)

| Artifact                                     | Language |
|----------------------------------------------|----------|
| **All agent-facing files** (`AGENTS.md`, any future `docs/*.md`, `README`, agent notes) | **English only** |
| **Comments in source code** (`.rs` files)    | **Russian only** (as they are now — do not translate or strip them) |
| Identifiers, function/type/module names      | English (standard practice — do not rename) |
| User-facing UI strings                       | Russian (the game menu is in Russian) |
| Log file messages                            | Russian |

**Never mix:** do not write English comments in source files, and do not
write Russian text in agent documentation files.

---

## 2. Project Overview

A Windows-first (but cross-platform) prototype of a **fullscreen game main
menu**, rendered with **wgpu** on top of native GPU backends:

- **Windows → DirectX 12** (`wgpu::Backends::DX12`)
- **Linux → Vulkan** (`wgpu::Backends::VULKAN`)
- **macOS → Metal** (`wgpu::Backends::METAL`)

The selected backend is **enforced at startup** and verified against the
actual adapter; a mismatch is a critical error (see Logging below).

Current features: fullscreen borderless window, animated vsync render loop,
text rendering (font rasterized on CPU into an R8 glyph atlas), keyboard
navigation (W/S, arrows, layout-independent), Enter to activate menu items,
Esc to exit, log file, MessageBox for fatal errors.

Roadmap intent: this codebase will grow into a **full game**, so modularity
and separation of concerns are mandatory (see §4).

---

## 3. Tech Stack

| Crate     | Version | Purpose                                          |
|-----------|---------|--------------------------------------------------|
| `wgpu`    | `30`    | Graphics API (WebGPU-style; DX12/Vulkan/Metal)    |
| `winit`   | `0.30`  | Window creation + event loop (`ApplicationHandler`) |
| `pollster`| `0.4`   | Blocking on wgpu async calls (`request_adapter`, `request_device`) |
| `fontdue` | `0.9`   | CPU-side font rasterization (glyph bitmaps + metrics) |
| `windows` | `0.62`  | Win32 API, **Windows only** (`MessageBoxW` for fatal errors) |

Toolchain: stable Rust (developed on 1.99). Edition 2021.

---

## 4. Architecture

```
src/
├── main.rs              Entry point: panic hook → logging init → run app.
│                        `#![windows_subsystem = "windows"]` — no console window.
├── config.rs            Configuration constants: window title/size, menu strings,
│                        log file name. User-facing text lives HERE.
├── app.rs               ORCHESTRATOR. Implements `winit::application::ApplicationHandler`.
│                        Contains NO game logic and NO rendering code — only
│                        routing: window events → input → game → view → renderer.
│                        Also: hidden-window startup (render first frame BEFORE
│                        `set_visible(true)` to avoid a white flash at launch).
├── input.rs             INPUT module. Translates winit `KeyEvent` into engine
│                        `Action` values (Up/Down/Confirm/Back), layout-independent
│                        (Latin + Russian). Contains anti-repeat protection (§5).
├── game/                GAME LOGIC module. Pure Rust, no graphics/winit deps.
│   ├── mod.rs           `Game` — consumes `input::Action`, owns state,
│   │                    exposes `GameView` snapshot for rendering.
│   └── menu.rs          `Menu` — selection state machine.
├── gfx/                 GRAPHICS module. Owns ALL rendering.
│   ├── mod.rs           `Renderer` facade: instance/surface/device/queue setup,
│   │                    swapchain config, per-frame render, atlas upload.
│   ├── platform/        Backend selection per OS (§6).
│   │   ├── windows.rs   → DX12
│   │   ├── linux.rs     → Vulkan
│   │   └── macos.rs     → Metal
│   ├── common.rs        Vertex format, quad/draw-list primitives, ortho MVP.
│   ├── pipeline.rs      WGSL shader + render pipeline + bind group layouts.
│   ├── text.rs          Text: fontdue rasterizer, shelf-packed R8 atlas
│   │                    (2048×2048), glyph cache keyed by (char, px),
│   │                    font file discovery per OS.
│   └── frame.rs         Frame composition: `GameView` → draw list
│                        (panel, title, items, selection, message, hint).
└── logging/             LOGGING module.
    ├── mod.rs           Levels (INFO/WARN/ERROR), UTC timestamps, log file
    │                    next to the exe (`dx12_menu.log`, append mode),
    │                    `critical()` = log + MessageBox + exit(1).
    └── platform/        Fatal-error dialog per OS (Windows → `MessageBoxW`;
                        Linux/macOS → stderr fallback).
```

### Data flow (one frame)

```
winit event loop (app.rs)
  └─ WindowEvent::KeyboardInput
       └─ input::InputState::process()   → Option<Action>   (filters repeats)
            └─ game::Game::handle_action()  (pure state update)
                 └─ about_to_wait → window.request_redraw()
                      └─ RedrawRequested:
                           game.view() → GameView snapshot
                             └─ gfx::frame::build_menu_frame(view) → DrawList
                                  └─ gfx::Renderer::render() → GPU → Present
```

**Direction of dependencies:** `app` depends on `game`, `input`, `gfx`,
`logging`, `config`. `game` depends only on `config` (+`input::Action` type).
`gfx` depends only on `game::GameView` (read-only snapshot) + `logging` +
`config`. Never let `game` depend on `gfx` or on winit/wgpu types.

---

## 5. Important Behaviors (do not break)

1. **No key-press "leak"**: `input.rs` ignores autorepeat events
   (`KeyEvent.repeat == true`) and latches keys until `Released`. A second
   `Pressed` without a `Released` is dropped. This is intentional — it
   prevents a held Enter from firing twice (e.g., into a future game scene).
   Covered by unit tests.
2. **No white flash at startup**: the window is created with
   `.with_visible(false)`, the first frame (including DX12 pipeline
   compilation) is rendered while hidden, and only then
   `set_visible(true)` + `request_redraw()` are called.
3. **Backend verification**: `gfx/platform` picks the backend for the OS;
   `Renderer::new` compares the adapter's actual backend and fails with a
   critical error on mismatch.
4. **Fully black background**: clear color is black and the background quad
   is solid black (explicit user requirement).
5. **Vsync**: `PresentMode::Fifo`, continuous redraw via
   `about_to_wait → request_redraw`.
6. **No console window**: `#![windows_subsystem = "windows"]` in `main.rs`.
   All diagnostics go to the log file / MessageBox, never to stdout.

---

## 6. Platform Abstraction Rules

Anything OS-specific must live in a `platform` submodule selected via
`#[cfg(target_os = "...")]` + re-export:

- `gfx/platform/{windows,linux,macos}.rs` — backend selection.
- `logging/platform/{windows,linux,macos}.rs` — fatal error dialog.
- `gfx/text.rs` — font directory/file lists per OS (`cfg`-gated consts).

When adding new platform-specific behavior, follow the same pattern:
declare a module per OS, gate with `#[cfg]`, re-export a common name,
and keep the call sites platform-agnostic.

---

## 7. Build, Test, Run

```bash
cargo build            # debug build
cargo build --release  # release build (what users run)
cargo test             # unit tests (pure logic only — no windows, no GPU)
cargo run              # runs the debug binary (GUI window will open!)
```

- **Unit tests** cover pure logic only (input latch, menu state, logging
  helpers). They must never open windows or touch the GPU. Keep game logic
  testable by keeping `game/` free of winit/wgpu.
- **Testing the GUI**: a full GUI smoke test opens a real fullscreen window.
  Do this ONLY if you can close the process yourself (e.g.,
  `Stop-Process -Name dx12_menu -Force` after a few seconds). **Never ask
  the user to interact with the window** — the user is not at the computer.
  Work autonomously.
- `.cargo/config.toml` (project-local) **overrides** the global
  `~/.cargo/config.toml` on this machine, which sets `panic=abort` and
  `target-cpu=native` globally. The project appends
  `-C panic=unwind -C target-cpu=generic` because `panic=abort` breaks
  `cargo test`. Do not remove these overrides.

---

## 8. Logging & Errors

- Log file: `dx12_menu.log`, written **next to the executable**
  (`std::env::current_exe()`), append mode, UTF-8, UTC timestamps.
- Use `logging::info / warn / error` from anywhere.
- **Critical errors** (window creation failure, GPU/backend mismatch, font
  not found, event loop failure, log file unwritable): call
  `logging::critical(msg)` — it writes to the log, shows a blocking
  MessageBox (`MB_ICONERROR|TOPMOST` on Windows) and exits the process.
- Panics are captured by a panic hook in `main.rs` and written to the log.

---

## 9. wgpu 30 API Notes (learned the hard way — verified against sources)

- `Instance::new(InstanceDescriptor)` — descriptor by value; `InstanceDescriptor`
  has **no `Default`**; construct all fields (`backends`, `flags`,
  `memory_budget_thresholds`, `backend_options`, `display`).
- `instance.request_adapter(&opts)` → `impl Future<Output = Result<Adapter, _>>`;
  `RequestAdapterOptions` has an `apply_limit_buckets` field.
- `adapter.request_device(&DeviceDescriptor)` → future of `(Device, Queue)`;
  descriptor fields: `label, required_features, required_limits,
  experimental_features, memory_hints, trace`.
- `surface.get_current_texture()` returns the enum
  `wgpu::CurrentSurfaceTexture` (`Success / Suboptimal / Outdated / Lost /
  Timeout / Occluded / Validation`) — **not** a `Result`.
- Present via **`queue.present(frame)`** — `SurfaceTexture::present()` no
  longer exists.
- `PipelineLayoutDescriptor`: `bind_group_layouts: &[Option<&BindGroupLayout>]`,
  `immediate_size` (no `push_constant_ranges` field).
- `RenderPipelineDescriptor`: has `cache: Option<&PipelineCache>` and
  `multiview_mask`; `VertexState.buffers: &[Option<VertexBufferLayout>]`.
- `RenderPassColorAttachment` requires `depth_slice: None` (2D case).
- `queue.submit` takes an iterator; `write_texture` uses
  `TexelCopyTextureInfo` / `TexelCopyBufferLayout` (renamed from ImageCopy*).
- `MemoryHints` and `Trace` have `Default`; `ExperimentalFeatures::disabled()`.

---

## 10. Conventions

- Comments in Rust sources: **Russian**, explain *why*, not just *what*.
- New modules: one concern per module; pure logic (game) must stay
  dependency-free from winit/wgpu so it remains unit-testable.
- New user-facing strings go into `config.rs` **and** must be added to the
  pre-rasterization string list in `gfx/mod.rs::menu_strings()` (glyphs are
  rasterized at startup; unknown glyphs are skipped at runtime).
- UI scale: all font sizes are multiplied by `window.scale_factor()`
  (HiDPI); keep it that way.
- Extensions to expect later: game scenes (beyond menu), gamepad input,
  audio, asset pipeline. Keep the `app → game / gfx` boundaries clean so
  these can be added without rewrites.

---

## 11. Current State / Done

- [x] Menu renders (title, items, selection with underline, messages, hint)
- [x] Keyboard navigation, layout-independent (W/S, ↑/↓, Enter, Esc)
- [x] Anti-repeat input protection (no Enter "leak" on scene entry)
- [x] No console window (GUI subsystem), no white flash at startup
- [x] Black background, vsync, fullscreen borderless, hidden cursor
- [x] Log file next to exe + MessageBox for critical errors
- [x] 12 unit tests passing (`cargo test`)
- [x] Debug and release builds warning-free
