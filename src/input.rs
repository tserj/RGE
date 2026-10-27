//! Модуль ввода: преобразование событий winit в игровые действия.
//!
//! Защита от «проскока» ввода:
//!  * события autorepeat (удержание клавиши) игнорируются;
//!  * повторное Pressed без промежуточного Released игнорируется —
//!    клавиша «защёлкивается» и не сработает снова, пока не будет отпущена.

use std::collections::HashSet;

use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey};

/// Игровое действие, независимое от раскладки и привязок.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Confirm,
    Back,
}

pub struct InputState {
    /// Клавиши, ожидающие отпускания (защёлка).
    latched: HashSet<Key>,
}

impl Default for InputState {
    fn default() -> Self {
        Self { latched: HashSet::new() }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Обрабатывает событие клавиатуры. Возвращает действие,
    /// если это было «свежее» нажатие значимой клавиши.
    pub fn process(&mut self, key: &Key, state: ElementState, repeat: bool) -> Option<Action> {
        match state {
            ElementState::Pressed => {
                // autorepeat и повторные Pressed без отпускания не проходят
                if repeat || !self.latched.insert(key.clone()) {
                    return None;
                }
                match map_key(key) {
                    Some(action) => Some(action),
                    None => {
                        // незначимая клавиша — защёлка не нужна
                        self.latched.remove(key);
                        None
                    }
                }
            }
            ElementState::Released => {
                self.latched.remove(key);
                None
            }
        }
    }
}

/// Раскладконезависимое преобразование клавиши в действие
/// (поддерживаются и латинская, и русская раскладки).
fn map_key(key: &Key) -> Option<Action> {
    match key {
        Key::Named(NamedKey::ArrowUp) => Some(Action::Up),
        Key::Named(NamedKey::ArrowDown) => Some(Action::Down),
        Key::Named(NamedKey::Enter) => Some(Action::Confirm),
        Key::Named(NamedKey::Escape) => Some(Action::Back),
        Key::Character(c) => match c.as_str() {
            "w" | "W" | "ц" | "Ц" => Some(Action::Up),
            "s" | "S" | "ы" | "Ы" => Some(Action::Down),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_w() -> Key {
        Key::Character("w".into())
    }

    fn key_enter() -> Key {
        Key::Named(NamedKey::Enter)
    }

    #[test]
    fn autorepeat_ignored() {
        let mut input = InputState::new();
        let pressed = ElementState::Pressed;

        assert_eq!(input.process(&key_w(), pressed, false), Some(Action::Up));
        // удержание: repeat-события не проходят
        assert_eq!(input.process(&key_w(), pressed, true), None);
        assert_eq!(input.process(&key_w(), pressed, true), None);
    }

    #[test]
    fn double_press_without_release_ignored() {
        let mut input = InputState::new();
        let pressed = ElementState::Pressed;
        let released = ElementState::Released;

        assert_eq!(input.process(&key_enter(), pressed, false), Some(Action::Confirm));
        // «проскок»: второй Pressed до Released — игнорируется
        assert_eq!(input.process(&key_enter(), pressed, false), None);
        // после отпускания клавиша снова работает
        assert_eq!(input.process(&key_enter(), released, false), None);
        assert_eq!(input.process(&key_enter(), pressed, false), Some(Action::Confirm));
    }

    #[test]
    fn russian_layout() {
        let mut input = InputState::new();
        let pressed = ElementState::Pressed;
        let ts = Key::Character("ц".into());
        let er = Key::Character("ы".into());

        assert_eq!(input.process(&ts, pressed, false), Some(Action::Up));
        assert_eq!(input.process(&er, pressed, false), Some(Action::Down));
    }

    #[test]
    fn unmapped_key_not_latched() {
        let mut input = InputState::new();
        let pressed = ElementState::Pressed;
        let a = Key::Character("a".into());

        assert_eq!(input.process(&a, pressed, false), None);
        // значимые клавиши продолжают работать
        assert_eq!(input.process(&key_w(), pressed, false), Some(Action::Up));
    }
}
