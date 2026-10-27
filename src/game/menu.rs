//! Игровое меню: чистая логика без привязки к графике.

use crate::config;

/// Состояние меню.
pub struct Menu {
    pub selected: usize,
    pub message: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        Self { selected: 0, message: None }
    }

    pub fn up(&mut self) {
        self.selected = (self.selected + config::MENU_ITEMS.len() - 1) % config::MENU_ITEMS.len();
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1) % config::MENU_ITEMS.len();
    }

    /// Активация выбранного пункта. Возвращает `true`, если выбран выход
    /// из приложения.
    pub fn confirm(&mut self) -> bool {
        match self.selected {
            0 => self.message = Some(config::MESSAGES[0].into()),
            1 => self.message = Some(config::MESSAGES[1].into()),
            2 => self.message = Some(config::MESSAGES[2].into()),
            _ => return true, // Выход
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_around() {
        let mut menu = Menu::new();
        menu.up();
        assert_eq!(menu.selected, config::MENU_ITEMS.len() - 1);
        menu.down();
        assert_eq!(menu.selected, 0);
        menu.down();
        assert_eq!(menu.selected, 1);
    }

    #[test]
    fn exit_on_last_item() {
        let mut menu = Menu::new();
        menu.selected = config::MENU_ITEMS.len() - 1; // «Выход»
        assert!(menu.confirm());
    }

    #[test]
    fn no_exit_on_other_items() {
        let mut menu = Menu::new();
        menu.selected = 0;
        assert!(!menu.confirm());
        assert!(menu.message.is_some());
    }
}
