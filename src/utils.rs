use std::collections::VecDeque;

pub fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

pub fn ilerpf(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

const RAND_A: u64 = 6364136223846793005;
const RAND_C: u64 = 1442695040888963407;

pub fn rand() -> u32 {
    static mut RAND_STATE: u64 = 0;
    unsafe {
        RAND_STATE = RAND_STATE.wrapping_mul(RAND_A).wrapping_add(RAND_C);
        ((RAND_STATE >> 32) & 0xFFFFFFFF) as u32
    }
}

pub fn emod(a: i32, b: i32) -> i32 {
    (a % b + b) % b
}

pub fn ring_displace_back<T>(ring: &mut VecDeque<T>, item: T, capacity: usize) {
    if ring.len() == capacity {
        ring.pop_front();
    }
    ring.push_back(item);
}
