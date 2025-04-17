mod web;
pub use web::*;

pub trait PlatformRenderer {
    fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32);
}
