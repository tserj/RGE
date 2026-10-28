//! Модуль ввода: преобразование физических клавиш Bevy в игровые действия.
//!
//! `KeyCode` в Bevy — физическое расположение клавиш, поэтому навигация
//! работает одинаково на латинской и русской раскладках (W и Ц — одна
//! физическая клавиша).
//!
//! Защита от «проскока» ввода обеспечивается самим `ButtonInput` Bevy:
//! `just_pressed` срабатывает ровно один раз на физическое нажатие
//! (`ButtonInput::press` игнорирует повторный «нажат» уже зажатой клавиши,
//! а bevy-winit не пропускает autorepeat-события).

use bevy::input::keyboard::KeyCode;

/// Игровое действие, независимое от раскладки и привязок.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Confirm,
    Back,
}

/// Клавиши, участвующие в навигации меню (для систем ввода).
pub const KEYS: [KeyCode; 7] = [
    KeyCode::KeyW,
    KeyCode::ArrowUp,
    KeyCode::KeyS,
    KeyCode::ArrowDown,
    KeyCode::Enter,
    KeyCode::NumpadEnter,
    KeyCode::Escape,
];

/// Раскладконезависимое преобразование клавиши в действие.
pub fn map_key(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::KeyW | KeyCode::ArrowUp => Some(Action::Up),
        KeyCode::KeyS | KeyCode::ArrowDown => Some(Action::Down),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(Action::Confirm),
        KeyCode::Escape => Some(Action::Back),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_keys() {
        assert_eq!(map_key(KeyCode::KeyW), Some(Action::Up));
        assert_eq!(map_key(KeyCode::ArrowUp), Some(Action::Up));
        assert_eq!(map_key(KeyCode::KeyS), Some(Action::Down));
        assert_eq!(map_key(KeyCode::ArrowDown), Some(Action::Down));
    }

    #[test]
    fn confirm_and_back() {
        assert_eq!(map_key(KeyCode::Enter), Some(Action::Confirm));
        assert_eq!(map_key(KeyCode::NumpadEnter), Some(Action::Confirm));
        assert_eq!(map_key(KeyCode::Escape), Some(Action::Back));
    }

    #[test]
    fn unmapped_key_is_none() {
        assert_eq!(map_key(KeyCode::KeyA), None);
        assert_eq!(map_key(KeyCode::Space), None);
        assert_eq!(map_key(KeyCode::F5), None);
    }

    #[test]
    fn keys_table_covers_all_mapped_keys() {
        // Каждая клавиша из таблицы ввода должна давать действие.
        for &key in &KEYS {
            assert!(map_key(key).is_some(), "{key:?} не даёт действия");
        }
    }
}
