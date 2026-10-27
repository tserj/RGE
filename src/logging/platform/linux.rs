//! Linux: диалогового окна без внешних зависимостей нет — выводим в stderr.

pub fn show_error_box(caption: &str, text: &str) {
    eprintln!("=== {caption} ===\n{text}");
}
