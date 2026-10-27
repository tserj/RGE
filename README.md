# RGE — Rust Game Engine

Initial vibecoded template of Game Engine written in Rust.

Current state: a fullscreen **game main menu** prototype built on `wgpu`
(DirectX 12 on Windows, Vulkan on Linux, Metal on macOS), with modular
architecture ready to grow into a full game.

## Features

- Fullscreen borderless window, vsync render loop (`wgpu 30` + `winit 0.30`)
- Forced graphics backend per OS with startup verification:
  **Windows → DirectX 12**, **Linux → Vulkan**, **macOS → Metal**
- Text rendering: font rasterized on CPU (`fontdue`) into an R8 glyph atlas,
  drawn with a custom wgpu pipeline (Cyrillic + arrows supported)
- Layout-independent keyboard navigation (W/S, ↑/↓, Enter, Esc — Latin and
  Russian layouts), with autorepeat/press-leak protection
- Clean startup: no console window, no white flash (first frame rendered
  before the window is shown)
- File logging (`dx12_menu.log` next to the executable) and native
  MessageBox for critical errors
- 12 unit tests covering input/menu/logging logic

## Architecture

```
src/
├── main.rs      entry point (bootstrap, panic hook)
├── config.rs    window/UI configuration constants
├── app.rs       orchestrator: winit event loop → input → game → gfx
├── input.rs     input module: actions, autorepeat filtering, key latching
├── game/        pure game logic (menu now, scenes later), no gfx deps
├── gfx/         ALL rendering: renderer facade, pipeline (WGSL), text atlas,
│                frame composition, platform backends (dx12/vulkan/metal)
└── logging/     file logger + platform fatal-error dialog (MessageBox)
```

Direction of dependencies: `app → {game, input, gfx, logging, config}`;
`game` never depends on `gfx` (renderer consumes a `GameView` snapshot) —
this keeps game logic unit-testable without a GPU.

## Build

```bash
cargo build --release   # executable: target/release/dx12_menu.exe
cargo test              # pure-logic unit tests (no GPU, no windows)
```

Requirements: stable Rust (MSVC toolchain on Windows), a DX12-capable GPU
on Windows.

## Controls

| Key                | Action            |
|--------------------|-------------------|
| W / ↑              | move up           |
| S / ↓              | move down         |
| Enter              | activate item     |
| Esc                | quit              |

Both Latin and Russian keyboard layouts work.

## Notes

- `AGENTS.md` contains the full guide for AI agents working on this repo
  (architecture details, conventions, language rules, verified wgpu 30 API
  notes). Read it before making changes.
- Comments in source files are intentionally in Russian; agent-facing
  documentation is English-only.
