pub fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

pub fn ilerpf(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}
