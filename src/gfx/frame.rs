//! Построение кадра: снимок игры (`GameView`) -> список квадров.
//!
//! Здесь живёт вся «вёрстка» экрана меню: панель, заголовок, пункты,
//! выделение, сообщения и подсказка.

use super::common::{push_quad, push_solid, DrawList};
use super::text::TextRasterizer;
use crate::game::GameView;

/// Цвета интерфейса.
mod colors {
    pub const WHITE: [f32; 4] = [0.95, 0.96, 1.0, 1.0];
    pub const ITEM: [f32; 4] = [0.68, 0.71, 0.78, 1.0];
    pub const ITEM_SELECTED: [f32; 4] = [1.0, 0.82, 0.25, 1.0];
    pub const UNDERLINE: [f32; 4] = [1.0, 0.82, 0.25, 0.9];
    pub const MESSAGE: [f32; 4] = [0.45, 0.9, 0.6, 1.0];
    pub const HINT: [f32; 4] = [0.55, 0.58, 0.68, 1.0];
    pub const PANEL: [f32; 4] = [0.0, 0.0, 0.0, 0.45];
    pub const BORDER: [f32; 4] = [0.35, 0.45, 0.8, 0.8];
}

/// Строит весь кадр меню.
pub fn build_menu_frame(
    r: &mut TextRasterizer,
    view: &GameView,
    width: u32,
    height: u32,
    scale: f32,
) -> DrawList {
    let w = width as f32;
    let h = height as f32;
    let mut v: DrawList = Vec::with_capacity(4096);

    // Фон: полностью чёрный.
    let black = [0.0, 0.0, 0.0, 1.0];
    push_quad(&mut v, 0.0, 0.0, w, h, r.white_uv, black, black);

    // Полупрозрачная панель с рамкой по центру.
    let panel_w = 760.0 * scale;
    let panel_h = 700.0 * scale;
    let px0 = (w - panel_w) * 0.5;
    let py0 = (h - panel_h) * 0.5 - 30.0 * scale;
    push_solid(&mut v, r.white_uv, px0, py0, panel_w, panel_h, colors::PANEL);

    let border = 2.0 * scale;
    let bx0 = px0;
    let by0 = py0;
    let bx1 = px0 + panel_w - border;
    let by1 = py0 + panel_h - border;
    push_solid(&mut v, r.white_uv, bx0, by0, panel_w, border, colors::BORDER);
    push_solid(&mut v, r.white_uv, bx0, by1, panel_w, border, colors::BORDER);
    push_solid(&mut v, r.white_uv, bx0, by0, border, panel_h, colors::BORDER);
    push_solid(&mut v, r.white_uv, bx1, by0, border, panel_h, colors::BORDER);

    let px_title = (72.0 * scale).round() as u32;
    let px_item = (48.0 * scale).round() as u32;
    let px_msg = (32.0 * scale).round() as u32;
    let px_hint = (24.0 * scale).round() as u32;

    // Заголовок.
    let title_w = r.text_width(view.title, px_title);
    draw_text(r, &mut v, (w - title_w) * 0.5, h * 0.22, px_title, view.title, colors::WHITE);

    // Пункты меню.
    let spacing = 100.0 * scale;
    let items_top = h * 0.40;
    for (i, item) in view.items.iter().enumerate() {
        let baseline = items_top + i as f32 * spacing + px_item as f32 * 0.8;
        let is_sel = i == view.selected;
        let label = if is_sel {
            format!("> {item} <")
        } else {
            (*item).to_string()
        };
        let color = if is_sel { colors::ITEM_SELECTED } else { colors::ITEM };
        let item_w = r.text_width(&label, px_item);
        let item_x = (w - item_w) * 0.5;
        draw_text(r, &mut v, item_x, baseline, px_item, &label, color);

        if is_sel {
            push_solid(
                &mut v,
                r.white_uv,
                item_x,
                baseline + 18.0 * scale,
                item_w,
                3.0 * scale,
                colors::UNDERLINE,
            );
        }
    }

    // Сообщение о действии.
    if let Some(msg) = &view.message {
        let msg_w = r.text_width(msg, px_msg);
        let baseline = items_top + view.items.len() as f32 * spacing + 40.0 * scale;
        draw_text(r, &mut v, (w - msg_w) * 0.5, baseline, px_msg, msg, colors::MESSAGE);
    }

    // Подсказка по управлению.
    let hint_w = r.text_width(view.hint, px_hint);
    draw_text(r, &mut v, (w - hint_w) * 0.5, h - 48.0 * scale, px_hint, view.hint, colors::HINT);

    v
}

/// Рисует строку текста, начиная от `x`, с базовой линией `baseline`.
fn draw_text(
    r: &mut TextRasterizer,
    v: &mut DrawList,
    x: f32,
    baseline: f32,
    px: u32,
    s: &str,
    color: [f32; 4],
) {
    let mut pen = x;
    for ch in s.chars() {
        if let Some(g) = r.glyph(ch, px) {
            if g.w > 0.0 && g.h > 0.0 {
                let gx = pen + g.xmin;
                let gy = baseline - (g.ymin + g.h); // ymin растёт вверх, экранный Y — вниз
                push_quad(v, gx, gy, g.w, g.h, [g.u0, g.v0, g.u1, g.v1], color, color);
            }
            pen += g.advance;
        } else {
            pen += px as f32 * 0.3;
        }
    }
}
