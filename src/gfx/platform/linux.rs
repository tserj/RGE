//! Linux: принудительный бэкенд Vulkan.

use wgpu::Backends;

pub fn backends() -> Backends {
    Backends::VULKAN
}

pub const BACKEND_NAME: &str = "Vulkan";

/// Конкретный бэкенд wgpu для сверки с адаптером.
pub fn backend() -> wgpu::Backend {
    wgpu::Backend::Vulkan
}
