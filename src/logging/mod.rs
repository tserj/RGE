//! Модуль логирования.
//!
//! На нативных платформах сообщения пишутся в лог-файл рядом с исполняемым
//! файлом; если лог недоступен — показывается диалог и приложение завершается
//! (см. подмодуль `platform`).
//!
//! В web-сборке (wasm) лог-файла нет: модуль вырождается в no-op,
//! диагностику в консоль браузера пишет сам Bevy (`LogPlugin`).

// ---------------------------------------------------------------------------
// Нативные платформы: файл рядом с exe
// ---------------------------------------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::platform;

    /// Имя лог-файла. Создаётся рядом с исполняемым файлом.
    const LOG_FILE_NAME: &str = "game_menu.log";

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Level {
        Info,
        Error,
    }

    impl Level {
        fn as_str(self) -> &'static str {
            match self {
                Level::Info => "INFO ",
                Level::Error => "ERROR",
            }
        }
    }

    struct Logger {
        file: Mutex<Option<File>>,
    }

    static LOGGER: OnceLock<Logger> = OnceLock::new();

    /// Инициализация: открывает лог-файл рядом с exe (в режиме дозаписи).
    /// Если лог недоступен — диалоговое окно и завершение процесса.
    pub fn init() {
        let path = match log_path() {
            Ok(p) => p,
            Err(e) => fatal(&e),
        };
        let file = match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(f) => f,
            Err(e) => fatal(&format!("не удалось открыть лог {}: {e}", path.display())),
        };

        let _ = LOGGER.set(Logger { file: Mutex::new(Some(file)) });
        info(&format!("=== запуск, лог: {} ===", path.display()));
    }

    pub fn info(msg: &str) {
        write(Level::Info, msg);
    }

    pub fn error(msg: &str) {
        write(Level::Error, msg);
    }

    fn fatal(msg: &str) -> ! {
        platform::show_error_box("Критическая ошибка", msg);
        std::process::exit(1);
    }

    fn log_path() -> Result<PathBuf, String> {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .map(|dir| dir.join(LOG_FILE_NAME))
            .ok_or_else(|| "не удалось определить папку exe".to_string())
    }

    fn write(level: Level, msg: &str) {
        let line = format!("[{} {}] {}", timestamp(), level.as_str(), msg);

        // Дублируем в stderr: в debug-сборках с консолью это полезно,
        // в GUI-подсистеме просто игнорируется.
        eprintln!("{line}");

        if let Some(logger) = LOGGER.get() {
            if let Ok(mut guard) = logger.file.lock() {
                if let Some(file) = guard.as_mut() {
                    let _ = writeln!(file, "{line}");
                    let _ = file.flush();
                }
            }
        }
    }

    /// UTC-время в формате `2025-10-27 08:41:23` (без внешних зависимостей).
    fn timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs() as i64;
        let days = secs.div_euclid(86_400);
        let tod = secs.rem_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!(
            "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
            tod / 3600,
            (tod % 3600) / 60,
            tod % 60
        )
    }

    /// Алгоритм Хиннанта: дни от эпохи -> (год, месяц, день).
    fn civil_from_days(z: i64) -> (i64, u32, u32) {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = (z - era * 146_097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        (if m <= 2 { y + 1 } else { y }, m, d)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn civil_date() {
            // 0 секунд от эпохи = 1970-01-01
            assert_eq!(civil_from_days(0), (1970, 1, 1));
            // 2025-10-27 = 20388 дней от эпохи
            assert_eq!(civil_from_days(20_388), (2025, 10, 27));
        }

        #[test]
        fn timestamp_format() {
            let ts = timestamp();
            assert_eq!(ts.len(), 19);
            assert_eq!(&ts[4..5], "-");
            assert_eq!(&ts[10..11], " ");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{error, info, init};

#[cfg(not(target_arch = "wasm32"))]
pub mod platform;

// ---------------------------------------------------------------------------
// Web (wasm): без лог-файла; консоль браузера пишет Bevy
// ---------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
pub fn init() {}

#[cfg(target_arch = "wasm32")]
pub fn info(_msg: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn error(_msg: &str) {}
