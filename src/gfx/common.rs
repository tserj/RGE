//! Общие примитивы рендера: формат вершин, квадры, ортографическая проекция.

/// Формат вершины: позиция (пиксели) + цвет + UV-координаты в атласе.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

/// Список квадров одного кадра (4 вершины на квад).
pub type DrawList = Vec<Vertex>;

/// Максимальное число квадров в кадре.
pub const MAX_QUADS: usize = 8192;

pub const VERTEX_ATTRS: [wgpu::VertexAttribute; 3] = [
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 8, shader_location: 1 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
];

pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRS,
    }
}

/// `&[T]` -> `&[u8]` для загрузки в GPU-буферы.
pub fn as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(slice.as_ptr().cast::<u8>(), std::mem::size_of_val(slice))
    }
}

/// Добавляет квад (два треугольника) с раздельными цветами верх/низ.
pub fn push_quad(
    list: &mut DrawList,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    uv: [f32; 4],
    top: [f32; 4],
    bottom: [f32; 4],
) {
    if list.len() / 4 >= MAX_QUADS {
        return;
    }
    let (u0, v0, u1, v1) = (uv[0], uv[1], uv[2], uv[3]);
    list.extend_from_slice(&[
        Vertex { pos: [x, y], color: top, uv: [u0, v0] },
        Vertex { pos: [x + w, y], color: top, uv: [u1, v0] },
        Vertex { pos: [x + w, y + h], color: bottom, uv: [u1, v1] },
        Vertex { pos: [x, y + h], color: bottom, uv: [u0, v1] },
    ]);
}

/// Сплошной прямоугольник (использует белый пиксель атласа).
pub fn push_solid(
    list: &mut DrawList,
    white_uv: [f32; 4],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
) {
    push_quad(list, x, y, w, h, white_uv, color, color);
}

/// Ортографическая проекция пикселей -> NDC (column-major, WGSL mat4x4).
pub fn ortho_mvp(width: u32, height: u32) -> [f32; 16] {
    let w = width as f32;
    let h = height as f32;
    [
        2.0 / w, 0.0, 0.0, 0.0,
        0.0, -2.0 / h, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ]
}
