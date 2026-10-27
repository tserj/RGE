//! Выбор графического бэкенда по целевой платформе:
//! Windows -> DirectX 12, Linux -> Vulkan, macOS -> Metal.

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{backend, backends, BACKEND_NAME};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::{backend, backends, BACKEND_NAME};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{backend, backends, BACKEND_NAME};

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
compile_error!("неподдерживаемая платформа: нет графического бэкенда");
