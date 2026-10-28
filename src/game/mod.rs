//! Игровая логика: сцены, обновление, состояние.
//!
//! Модуль полностью чистый: не зависит ни от Bevy, ни от графики.
//! Получает действия (`input::Action`), отдаёт снимок состояния
//! (`GameView`) для интерфейса. В будущем здесь появятся сцены игры.

pub mod menu;

use crate::input::Action;

/// Снимок изменчивой части игрового состояния для интерфейса (UI не знает
/// внутреннего устройства игры; статичные строки UI берёт из `config`).
pub struct GameView {
    pub selected: usize,
    pub message: Option<String>,
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

    /// Прямой выбор пункта (клик/тап по пункту меню).
    pub fn select(&mut self, index: usize) {
        if self.running {
            self.menu.select(index);
        }
    }

    /// Продолжает ли работу приложение.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Снимок состояния для интерфейса.
    pub fn view(&self) -> GameView {
        GameView {
            selected: self.menu.selected,
            message: self.menu.message.clone(),
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
        assert_eq!(view.selected, 0);
        assert!(view.message.is_none());
    }

    #[test]
    fn select_jumps_to_item() {
        let mut game = Game::new();
        game.select(2); // «Моды»
        let view = game.view();
        assert_eq!(view.selected, 2);
        assert!(game.is_running());

        game.handle_action(Action::Confirm);
        assert!(game.view().message.is_some());
    }

    #[test]
    fn select_out_of_range_is_ignored() {
        let mut game = Game::new();
        game.select(99);
        assert_eq!(game.view().selected, 0);
    }

    #[test]
    fn input_after_exit_is_ignored() {
        let mut game = Game::new();
        game.handle_action(Action::Back);
        game.handle_action(Action::Up);
        game.select(0);
        game.handle_action(Action::Confirm);
        assert!(!game.is_running());
    }
}
