//! Windows: принудительный бэкенд DirectX 12.

use wgpu::Backends;

pub fn backends() -> Backends {
    Backends::DX12
}

pub const BACKEND_NAME: &str = "DirectX 12";

/// Конкретный бэкенд wgpu для сверки с адаптером.
pub fn backend() -> wgpu::Backend {
    wgpu::Backend::Dx12
}
