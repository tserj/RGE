//! Оркестратор приложения: событийный цикл winit, связывающий
//! ввод, игровую логику и графику.
//!
//! Здесь нет ни игровой логики, ни деталей рендера — только маршрутизация:
//! события окна -> ввод -> игра -> снимок состояния -> рендер.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window};

use crate::config;
use crate::gfx::Renderer;
use crate::game::Game;
use crate::input::InputState;
use crate::logging;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: Game,
    input: InputState,
    running: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            game: Game::new(),
            input: InputState::new(),
            running: true,
        }
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        self.running = false;
        event_loop.exit();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(config::WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                config::WINDOW_SIZE_LOGICAL.0,
                config::WINDOW_SIZE_LOGICAL.1,
            ))
            // Окно показываем только после отрисовки первого кадра —
            // иначе на мгновение виден белый фон неотрисованного окна.
            .with_visible(false);

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => logging::critical(&format!("не удалось создать окно: {e}")),
        };

        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        window.set_cursor_visible(false);

        match Renderer::new(&window) {
            Ok(mut renderer) => {
                logging::info(&format!("рендер готов: {}", renderer.adapter_info()));
                // Прогрев: первый кадр рисуем, пока окно скрыто — DX12
                // успевает скомпилировать пайплайны без вспышки на экране.
                renderer.render(&self.game.view(), &window);
                self.renderer = Some(renderer);
                self.window = Some(window.clone());
                window.set_visible(true);
                window.request_redraw();
            }
            Err(e) => logging::critical(&format!("ошибка инициализации графики: {e}")),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.exit(event_loop),

            WindowEvent::KeyboardInput { event: k, .. } => {
                if let Some(action) = self.input.process(&k.logical_key, k.state, k.repeat) {
                    self.game.handle_action(action);
                    if !self.game.is_running() {
                        self.exit(event_loop);
                    }
                }
            }

            WindowEvent::Resized(_size) => {
                // Повторная конфигурация свопчейна выполняется в render().
            }

            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    let view = self.game.view();
                    renderer.render(&view, window);
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.running {
            event_loop.exit();
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw(); // постоянный цикл отрисовки (vsync ограничивает FPS)
        }
    }
}
