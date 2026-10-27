//! macOS: принудительный бэкенд Metal.

use wgpu::Backends;

pub fn backends() -> Backends {
    Backends::METAL
}

pub const BACKEND_NAME: &str = "Metal";

/// Конкретный бэкенд wgpu для сверки с адаптером.
pub fn backend() -> wgpu::Backend {
    wgpu::Backend::Metal
}
