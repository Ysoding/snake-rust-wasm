use web_sys::CanvasRenderingContext2d;

use super::PlatformRenderer;

pub struct WebPlatformRenderer {
    ctx: CanvasRenderingContext2d,
}

impl WebPlatformRenderer {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self { ctx }
    }

    fn color_hex(&self, color: u32) -> String {
        let r = (color >> 0) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = (color >> 16) & 0xFF;
        let a = (color >> 24) & 0xFF;
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }
}

impl PlatformRenderer for WebPlatformRenderer {
    fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let hex = self.color_hex(color);
        self.ctx.set_fill_style_str(&hex);
        self.ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    }
}

// pub fn platform_stroke_rect(
//     ctx: &CanvasRenderingContext2d,
//     x: f64,
//     y: f64,
//     w: f64,
//     h: f64,
//     color: u32,
// ) {
//     let hex = format!("#{:08X}", color);
//     ctx.set_fill_style_str(&hex);
//     ctx.stroke_rect(x, y, w, h);
// }

// pub fn platform_fill_text(
//     ctx: &CanvasRenderingContext2d,
//     x: f64,
//     y: f64,
//     text: &str,
//     size: u32,
//     color: u32,
// ) {
//     let hex = format!("#{:08X}", color);
//     ctx.set_fill_style_str(&hex);
//     ctx.set_font(&format!("{}px", size));
//     ctx.fill_text(text, x, y).unwrap();
// }
