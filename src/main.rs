//! Точка входа: инициализация логирования и запуск Bevy-приложения.
//!
//! Архитектура:
//!  * `config`  — конфигурация (окно, строки интерфейса, ресурсы);
//!  * `input`   — преобразование клавиш Bevy в игровые действия;
//!  * `game`    — чистая игровая логика (меню), без зависимостей от Bevy;
//!  * `ui`      — модуль интерфейса: Bevy UI, маршрутизация ввода в игру;
//!  * `logging` — лог-файл рядом с exe + MessageBox для критических ошибок.
//!
//! Платформы: Windows/Linux/macOS — нативно; web — WebAssembly (trunk);
//! Android/iOS — нативные оболочки с WebView, показывающие web-сборку
//! (см. `shells/` и README).

// GUI-подсистема Windows: при запуске exe консольное окно не создаётся.
// На других платформах атрибут игнорируется.
#![windows_subsystem = "windows"]

mod config;
mod game;
mod input;
mod logging;
mod ui;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode, PresentMode};

fn main() {
    install_panic_hook();

    // Лог-файл (на wasm — no-op): при невозможности открыть показывает
    // диалог и завершает процесс.
    logging::init();
    logging::info("старт приложения");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: config::WINDOW_TITLE.to_string(),
                mode: window_mode(),
                present_mode: PresentMode::AutoVsync,
                // WebGL: холст растягивается на всю страницу.
                fit_canvas_to_parent: true,
                // Окно показываем позже (см. `make_window_visible`):
                // пока пайплайн компилируется, белой вспышки не будет.
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ui::MenuUiPlugin)
        .run();

    logging::info("приложение завершено");
}

/// Полноэкранный режим — только на десктопе; в браузере размер задаёт холст.
fn window_mode() -> WindowMode {
    if cfg!(target_arch = "wasm32") {
        WindowMode::Windowed
    } else {
        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
    }
}

/// Паники попадают в лог (только на нативных платформах; в браузере
/// трассировку паники выводит среда исполнения).
fn install_panic_hook() {
    #[cfg(not(target_arch = "wasm32"))]
    std::panic::set_hook(Box::new(|info| {
        logging::error(&format!("паника: {info}"));
        let prev = std::panic::take_hook();
        prev(info);
    }));
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(|info| {
        // Трассировку паники в консоль браузера выводит среда исполнения;
        // здесь только единообразное логирование.
        logging::error(&format!("паника: {info}"));
    }));
}
