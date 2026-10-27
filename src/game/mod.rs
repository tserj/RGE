//! Игровая логика: сцены, обновление, состояние.
//!
//! Модуль полностью отделён от графики и ввода: получает действия
//! (`input::Action`), отдаёт снимок состояния (`GameView`) для рендера.
//! В будущем здесь появятся сцены собственно игры.

pub mod menu;

use crate::config;
use crate::input::Action;

/// Снимок игрового состояния для отрисовки (графика не знает
/// внутреннего устройства игры).
pub struct GameView {
    pub title: &'static str,
    pub items: &'static [&'static str],
    pub selected: usize,
    pub message: Option<String>,
    pub hint: &'static str,
}

pub struct Game {
    menu: menu::Menu,
    running: bool,
}

impl Game {
    pub fn new() -> Self {
        Self { menu: menu::Menu::new(), running: true }
    }

    /// Передаёт действие в игровую логику.
    pub fn handle_action(&mut self, action: Action) {
        if !self.running {
            return;
        }
        match action {
            Action::Up => self.menu.up(),
            Action::Down => self.menu.down(),
            Action::Confirm => {
                if self.menu.confirm() {
                    self.running = false;
                }
            }
            Action::Back => self.running = false,
        }
    }

    /// Продолжает ли работу приложение.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Снимок состояния для рендера.
    pub fn view(&self) -> GameView {
        GameView {
            title: config::GAME_TITLE,
            items: &config::MENU_ITEMS,
            selected: self.menu.selected,
            message: self.menu.message.clone(),
            hint: config::HINT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_flow() {
        let mut game = Game::new();
        game.handle_action(Action::Down);
        game.handle_action(Action::Down);
        game.handle_action(Action::Down); // «Выход»
        game.handle_action(Action::Confirm);
        assert!(!game.is_running());
    }

    #[test]
    fn back_exits_immediately() {
        let mut game = Game::new();
        game.handle_action(Action::Back);
        assert!(!game.is_running());
    }

    #[test]
    fn view_snapshot() {
        let game = Game::new();
        let view = game.view();
        assert_eq!(view.items.len(), config::MENU_ITEMS.len());
        assert_eq!(view.selected, 0);
        assert!(view.message.is_none());
    }
}
