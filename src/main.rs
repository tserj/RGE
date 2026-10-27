//! Точка входа: инициализация логирования, запуск приложения.
//!
//! Вся архитектура:
//!  * `app`    — оркестратор (события окна, маршрутизация ввода/логики/рендера);
//!  * `game`   — чистая игровая логика (меню, в будущем — сцены игры);
//!  * `gfx`    — модуль графики: весь рендер (DX12 / Vulkan / Metal по ОС);
//!  * `input`  — модуль ввода (фильтр autorepeat, защита от «проскока» клавиш);
//!  * `logging`— лог-файл рядом с exe + MessageBox для критических ошибок;
//!  * `config` — конфигурация (окно, строки интерфейса).

// GUI-подсистема Windows: при запуске exe консольное окно не создаётся —
// только окно игры.
#![windows_subsystem = "windows"]

mod app;
mod config;
mod game;
mod gfx;
mod input;
mod logging;

use logging::Level;

fn main() {
    // Паники тоже попадают в лог.
    std::panic::set_hook(Box::new(|info| {
        logging::error(&format!("паника: {info}"));
        let prev = std::panic::take_hook();
        prev(info);
    }));

    if let Err(e) = logging::init() {
        // Лог недоступен — это критично: показываем диалог и выходим.
        logging::platform::show_error_box("Критическая ошибка", &e);
        std::process::exit(1);
    }

    logging::log(Level::Info, "старт приложения");

    let mut builder = winit::event_loop::EventLoop::builder();
    let event_loop = match builder.build() {
        Ok(el) => el,
        Err(e) => logging::critical(&format!("не удалось создать event loop: {e}")),
    };

    let mut application = app::App::new();
    if let Err(e) = event_loop.run_app(&mut application) {
        logging::critical(&format!("ошибка event loop: {e}"));
    }

    logging::log(Level::Info, "приложение завершено");
}
