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

    fn stroke_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let hex = self.color_hex(color);
        self.ctx.set_stroke_style_str(&hex);
        self.ctx.stroke_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn fill_text(&self, x: i32, y: i32, text: &str, font_size: u32, color: u32) {
        let hex = self.color_hex(color);
        self.ctx.set_fill_style_str(&hex);

        self.ctx.set_font(&format!("{}px", font_size));
        self.ctx.fill_text(text, x as f64, y as f64).unwrap();
    }
}
