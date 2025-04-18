mod web;
pub use web::*;

pub trait PlatformRenderer: Clone {
    fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32);
    fn stroke_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32);
    fn fill_text(&self, x: i32, y: i32, text: &str, font_size: u32, color: u32);
}
