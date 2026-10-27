//! Текст: растеризация шрифта (fontdue) и упаковка глифов в R8-атлас.
//!
//! Модуль полностью «CPU-шный»: GPU-текстуру создаёт и наполняет рендерер.

use std::collections::HashMap;

use crate::logging;

pub const ATLAS_SIZE: u32 = 2048;
pub const ATLAS_PAD: u32 = 2;

/// Базовая оптимизационная шкала шрифта (px per em).
const FONT_SCALE: f32 = 96.0;

#[derive(Clone, Copy)]
pub struct Glyph {
    /// Нормированные UV-координаты в атласе.
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    /// Размер битмапы (пиксели).
    pub w: f32,
    pub h: f32,
    /// Смещения относительно пера/базовой линии.
    pub xmin: f32,
    /// От нижней границы битмапы до базовой линии (вверх — положительно).
    pub ymin: f32,
    pub advance: f32,
}

/// Поиск подходящего TTF по системным папкам текущей ОС.
#[cfg(target_os = "windows")]
const FONT_DIRS: &[&str] = &["C:\\Windows\\Fonts"];
#[cfg(target_os = "windows")]
const FONT_FILES: &[&str] =
    &["segoeui.ttf", "arial.ttf", "tahoma.ttf", "calibri.ttf", "consola.ttf"];

#[cfg(target_os = "linux")]
const FONT_DIRS: &[&str] =
    &["/usr/share/fonts", "/usr/local/share/fonts", "/usr/share/fonts/truetype"];
#[cfg(target_os = "linux")]
const FONT_FILES: &[&str] =
    &["DejaVuSans.ttf", "LiberationSans-Regular.ttf", "Ubuntu-R.ttf", "FreeSans.ttf"];

#[cfg(target_os = "macos")]
const FONT_DIRS: &[&str] = &["/System/Library/Fonts/Supplemental", "/Library/Fonts", "/System/Library/Fonts"];
#[cfg(target_os = "macos")]
const FONT_FILES: &[&str] = &["Arial.ttf", "Verdana.ttf", "Geneva.ttf", "Helvetica.ttf"];

fn load_font_bytes() -> Result<Vec<u8>, String> {
    // 1. Известные имена файлов в известных папках.
    for dir in FONT_DIRS {
        for name in FONT_FILES {
            let path = std::path::Path::new(dir).join(name);
            if let Ok(data) = std::fs::read(&path) {
                logging::info(&format!("шрифт: {}", path.display()));
                return Ok(data);
            }
        }
    }
    // 2. Первый попавшийся .ttf/.otf в системных папках.
    for dir in FONT_DIRS {
        let dir = std::path::Path::new(dir);
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let ok_ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
                .unwrap_or(false);
            if ok_ext {
                if let Ok(data) = std::fs::read(&path) {
                    logging::warn(&format!("шрифт не из списка, используем: {}", path.display()));
                    return Ok(data);
                }
            }
        }
    }
    Err(format!(
        "не найден TTF-шрифт (искировали папки: {})",
        FONT_DIRS.join(", ")
    ))
}

pub struct TextRasterizer {
    font: fontdue::Font,
    /// R8-данные атласа; рендерер копирует их в GPU-текстуру.
    pub atlas: Vec<u8>,
    pub atlas_dirty: bool,
    pack_x: u32,
    pack_y: u32,
    pack_row_h: u32,
    /// UV 1×1 белого пикселя для плашек и подчёркиваний.
    pub white_uv: [f32; 4],
    cache: HashMap<(char, u32), Option<Glyph>>,
}

impl TextRasterizer {
    pub fn new() -> Result<Self, String> {
        let data = load_font_bytes()?;
        let font = fontdue::Font::from_bytes(
            data,
            fontdue::FontSettings {
                collection_index: 0,
                scale: FONT_SCALE,
                ..Default::default()
            },
        )
        .map_err(|e| format!("fontdue: {e}"))?;

        let mut this = Self {
            font,
            atlas: vec![0; (ATLAS_SIZE * ATLAS_SIZE) as usize],
            atlas_dirty: false,
            pack_x: ATLAS_PAD,
            pack_y: ATLAS_PAD,
            pack_row_h: 0,
            white_uv: [0.0; 4],
            cache: HashMap::new(),
        };

        // Первый тайл — белый пиксель 1×1.
        let (x, y) = this.pack(1, 1).ok_or("атлас переполнен")?;
        this.atlas[(y * ATLAS_SIZE + x) as usize] = 255;
        let s = ATLAS_SIZE as f32;
        this.white_uv = [x as f32 / s, y as f32 / s, (x + 1) as f32 / s, (y + 1) as f32 / s];
        Ok(this)
    }

    /// Размеры шрифта (px), которые рендерер будет использовать.
    pub fn font_sizes(scale: f32) -> [u32; 4] {
        [
            (72.0 * scale).round() as u32, // заголовок
            (48.0 * scale).round() as u32, // пункты меню
            (32.0 * scale).round() as u32, // сообщение
            (24.0 * scale).round() as u32, // подсказка
        ]
    }

    fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if self.pack_x + w + ATLAS_PAD > ATLAS_SIZE {
            self.pack_x = ATLAS_PAD;
            self.pack_y += self.pack_row_h + ATLAS_PAD;
            self.pack_row_h = 0;
        }
        if self.pack_y + h + ATLAS_PAD > ATLAS_SIZE {
            return None;
        }
        let pos = (self.pack_x, self.pack_y);
        self.pack_x += w + ATLAS_PAD;
        self.pack_row_h = self.pack_row_h.max(h);
        Some(pos)
    }

    /// Глиф из кэша; при первом обращении растеризуется.
    pub fn glyph(&mut self, ch: char, px: u32) -> Option<Glyph> {
        let key = (ch, px);
        if let Some(g) = self.cache.get(&key) {
            return *g;
        }
        let g = self.rasterize(ch, px);
        self.cache.insert(key, g);
        g
    }

    fn rasterize(&mut self, ch: char, px: u32) -> Option<Glyph> {
        let (metrics, coverage) = if ch == ' ' {
            let idx = self.font.lookup_glyph_index(' ');
            self.font.rasterize_indexed(idx, px as f32)
        } else {
            let idx = self.font.lookup_glyph_index(ch);
            if idx == 0 {
                return None; // глифа нет в шрифте
            }
            self.font.rasterize_indexed(idx, px as f32)
        };

        let w = metrics.width as u32;
        let h = metrics.height as u32;
        let advance = if ch == ' ' {
            metrics.advance_width.max(px as f32 * 0.3)
        } else {
            metrics.advance_width
        };

        if w == 0 || h == 0 {
            return Some(Glyph {
                u0: self.white_uv[0],
                v0: self.white_uv[1],
                u1: self.white_uv[2],
                v1: self.white_uv[3],
                w: 0.0,
                h: 0.0,
                xmin: metrics.xmin as f32,
                ymin: metrics.ymin as f32,
                advance,
            });
        }

        let (x, y) = self.pack(w, h)?;
        for row in 0..h {
            let src = &coverage[(row * w) as usize..((row + 1) * w) as usize];
            let dst = ((y + row) * ATLAS_SIZE + x) as usize;
            self.atlas[dst..dst + w as usize].copy_from_slice(src);
        }
        self.atlas_dirty = true;

        let s = ATLAS_SIZE as f32;
        Some(Glyph {
            u0: x as f32 / s,
            v0: y as f32 / s,
            u1: (x + w) as f32 / s,
            v1: (y + h) as f32 / s,
            w: w as f32,
            h: h as f32,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
            advance,
        })
    }

    /// Пре-растеризация всех символов интерфейса во всех нужных размерах.
    pub fn prerasterize(&mut self, strings: &[&str], extra_chars: &str, sizes_px: &[u32]) {
        let mut chars = std::collections::BTreeSet::new();
        for s in strings {
            chars.extend(s.chars());
        }
        chars.extend(extra_chars.chars());
        chars.extend('a'..='z');
        chars.extend('A'..='Z');
        chars.extend('0'..='9');
        chars.extend(" .,:;!_-()<>[]/\\|@#%^&*+=?\"'`~—–«»".chars());

        let mut keys = Vec::new();
        for &px in sizes_px {
            for &c in &chars {
                keys.push((c, px));
            }
        }
        for (c, px) in keys {
            self.glyph(c, px);
        }
    }

    /// Ширина строки в пикселях.
    pub fn text_width(&mut self, s: &str, px: u32) -> f32 {
        let mut x = 0.0;
        for ch in s.chars() {
            x += self
                .glyph(ch, px)
                .map(|g| g.advance)
                .unwrap_or(px as f32 * 0.3);
        }
        x
    }
}
