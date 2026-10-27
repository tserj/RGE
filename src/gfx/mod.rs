//! Модуль графики: весь рендер приложения.
//!
//! Наружу торчит только [`Renderer`] — фасад с методами
//! `new` / `render` / `resize`. Внутри:
//!  * `platform` — выбор бэкенда по ОС (DX12 / Vulkan / Metal);
//!  * `common`   — вершины, квадры, проекция;
//!  * `pipeline` — WGSL-шейдер и пайплайн;
//!  * `text`     — растеризация шрифта и атлас глифов;
//!  * `frame`    — сборка кадра из снимка игры.

pub mod common;
pub mod frame;
pub mod pipeline;
pub mod platform;
pub mod text;

use std::sync::Arc;

use pollster::block_on;
use wgpu::util::DeviceExt;

use crate::game::GameView;
use crate::logging;

/// Рендерер: владеет устройством, свопчейном и ресурсами кадра.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_uniforms: wgpu::BindGroup,
    bind_texture: wgpu::BindGroup,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    rasterizer: text::TextRasterizer,
    adapter_info: String,
}

impl Renderer {
    /// Инициализация графики для окна. При неудаче возвращает Err
    /// (звонщик решает, как сообщить пользователю).
    pub fn new(window: &Arc<winit::window::Window>) -> Result<Self, String> {
        let expected_backend = platform::backends();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: expected_backend,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("create_surface: {e}"))?;

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|e| format!("request_adapter: {e}"))?;

        let info = adapter.get_info();
        let adapter_info = format!("{} ({:?})", info.name, info.backend);

        // Проверка: работаем ли через заявленный для платформы бэкенд.
        if info.backend != platform::backend() {
            return Err(format!(
                "ожидается бэкенд {}, а получен {:?} (адаптер: {})",
                platform::BACKEND_NAME,
                info.backend,
                info.name
            ));
        }
        logging::info(&format!("GPU: {adapter_info}"));
        logging::info(&format!("бэкенд: {}", platform::BACKEND_NAME));

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| format!("request_device: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // vsync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let bundle = pipeline::create_pipeline(&device, format);

        // Шрифт и атлас глифов.
        let mut rasterizer = text::TextRasterizer::new()?;
        let scale = window.scale_factor() as f32;
        // «ц ы» — русские буквы на месте W/S для текущей раскладки.
        rasterizer.prerasterize(&menu_strings(), "ц ы", &text::TextRasterizer::font_sizes(scale));
        rasterizer.atlas_dirty = false;

        let atlas_view = Self::upload_atlas_texture(&device, &queue, &rasterizer.atlas);
        let sampler = Self::create_sampler(&device);

        // Uniform-буфер: MVP-матрица 4×4 (64 байта).
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_uniforms = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind-uniforms"),
            layout: &bundle.uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let bind_texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind-texture"),
            layout: &bundle.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
            ],
        });

        // Динамический вершинный буфер + статический индексный.
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: (common::MAX_QUADS * 4 * std::mem::size_of::<common::Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices: Vec<u32> = Vec::with_capacity(common::MAX_QUADS * 6);
        for i in 0..common::MAX_QUADS as u32 {
            let base = i * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices"),
            contents: common::as_bytes(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline: bundle.pipeline,
            uniform_buf,
            bind_uniforms,
            bind_texture,
            vertex_buf,
            index_buf,
            rasterizer,
            adapter_info,
        })
    }

    /// Информация об адаптере (для логов).
    pub fn adapter_info(&self) -> &str {
        &self.adapter_info
    }

    /// Рендерит кадр со снимком игрового состояния.
    pub fn render(&mut self, view: &GameView, window: &winit::window::Window) {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        if self.config.width != size.width || self.config.height != size.height {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }

        let scale = window.scale_factor() as f32;
        let vertices = frame::build_menu_frame(&mut self.rasterizer, view, size.width, size.height, scale);
        if vertices.is_empty() {
            return;
        }

        let mvp = common::ortho_mvp(size.width, size.height);
        self.queue.write_buffer(&self.uniform_buf, 0, common::as_bytes(&mvp));
        self.queue.write_buffer(&self.vertex_buf, 0, common::as_bytes(&vertices));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(&self.device, &self.config);
                t
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return, // Timeout / Occluded / Validation — кадр пропускается
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("menu-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_uniforms, &[]);
            pass.set_bind_group(1, &self.bind_texture, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..(vertices.len() as u32 / 4 * 6), 0, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }

    fn upload_atlas_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &[u8],
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: wgpu::Extent3d {
                width: text::ATLAS_SIZE,
                height: text::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(text::ATLAS_SIZE),
                rows_per_image: Some(text::ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: text::ATLAS_SIZE,
                height: text::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }
}

// --- вспомогательное: строки интерфейса для пре-растеризации ---

fn menu_strings() -> Vec<&'static str> {
    let mut v = vec![crate::config::GAME_TITLE, crate::config::HINT];
    v.extend(crate::config::MENU_ITEMS);
    v.extend(crate::config::MESSAGES);
    v
}
