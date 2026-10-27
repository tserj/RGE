//! Модуль логирования.
//!
//! Все сообщения пишутся в `.log`-файл рядом с исполняемым файлом.
//! Критические ошибки дополнительно показываются пользователю
//! через диалоговое окно (см. подмодуль `platform`).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

pub mod platform;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

struct Logger {
    file: Mutex<Option<File>>,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Инициализация: открывает лог-файл рядом с exe (в режиме дозаписи).
pub fn init() -> Result<PathBuf, String> {
    let path = log_path()?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("не удалось открыть лог {}: {e}", path.display()))?;

    let _ = LOGGER.set(Logger { file: Mutex::new(Some(file)) });
    info(&format!("=== запуск, лог: {} ===", path.display()));
    Ok(path)
}

fn log_path() -> Result<PathBuf, String> {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .ok_or("не удалось определить папку exe")?;
    Ok(dir.join(config::LOG_FILE_NAME))
}

pub fn info(msg: &str) {
    write(Level::Info, msg);
}

pub fn warn(msg: &str) {
    write(Level::Warn, msg);
}

pub fn error(msg: &str) {
    write(Level::Error, msg);
}

/// Критическая ошибка: запись в лог + диалоговое окно + завершение процесса.
pub fn critical(msg: &str) -> ! {
    error(msg);
    write(Level::Error, "завершение работы из-за критической ошибки");
    platform::show_error_box("Критическая ошибка", msg);
    std::process::exit(1);
}

/// Запись с явным уровнем.
pub fn log(level: Level, msg: &str) {
    write(level, msg);
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
