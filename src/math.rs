#![allow(dead_code)]

use std::ops::{Add, Mul, Sub};

pub const PI: f64 = 3.14159265358979323846;

#[derive(Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy)]
pub struct Rect2i {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl Vector2 {
    pub const ORIGIN: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    pub const ONE: Vector2 = Vector2 { x: 1.0, y: 1.0 };

    pub fn new(new_x: f64, new_y: f64) -> Self {
        Vector2 { x: new_x, y: new_y }
    }

    pub fn new_i32(new_x: i32, new_y: i32) -> Self {
        Vector2 { x: new_x as f64, y: new_y as f64 }
    }
}

impl Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Add<f64> for Vector2 {
    type Output = Vector2;
    fn add(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub<f64> for Vector2 {
    type Output = Vector2;
    fn sub(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Add for Vector2 {
    type Output = Vector2;
    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Vector3 {
    const ORIGIN: Vector2 = Vector2 { x: 0.0, y: 0.0 };
}

#[inline(always)]
pub fn hadamard(a: Vector2, b: Vector2) -> Vector2 {
    Vector2::new(a.x * b.x, a.y * b.y)
}

#[inline(always)]
pub fn inner_v2(a: Vector2, b: Vector2) -> f64 {
    a.x * b.x + a.y * b.y
}

#[inline(always)]
pub fn length_sq_v2(a: Vector2) -> f64 {
    inner_v2(a, a)
}

#[inline(always)]
pub fn length_v2(a: Vector2) -> f64 {
    square_root(length_sq_v2(a))
}

#[inline(always)]
pub fn lerp_f64(a: f64, t: f64, b: f64) -> f64 {
    a * (1.0 - t) + b * t
}

#[inline(always)]
pub fn lerp_v2(a: Vector2, t: f64, b: Vector2) -> Vector2 {
    let v1 = a * (1.0 - t);
    let v2 = b * t;
    return v1 + v2;
}

#[inline(always)]
pub fn sign_f64(a: f64) -> f64 {
    if a < 0.0 {
        -1.0
    } else {
        1.0
    }
}

#[inline(always)]
pub fn sign_i32(a: i32) -> i32 {
    if a < 0 {
        -1
    } else {
        1
    }
}

#[inline(always)]
pub fn abs_i32(a: i32) -> i32 {
    if a < 0 {
        -a
    } else {
        a
    }
}

#[inline(always)]
pub fn abs_f64(a: f64) -> f64 {
    if a < 0.0 {
        -a
    } else {
        a
    }
}

#[inline(always)]
pub fn clamp_f64(min: f64, a: f64, max: f64) -> f64 {
    max_f64(min, min_f64(max, a))
}

#[inline(always)]
pub fn max_f64(a: f64, b: f64) -> f64 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn min_f64(a: f64, b: f64) -> f64 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn max_u32(a: u32, b: u32) -> u32 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn min_u32(a: u32, b: u32) -> u32 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn max_i32(a: i32, b: i32) -> i32 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn min_i32(a: i32, b: i32) -> i32 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn clamp01_f64(a: f64) -> f64 {
    max_f64(0.0, min_f64(a, 1.0))
}

#[inline(always)]
pub fn square(a: f64) -> f64 {
    a * a
}

#[inline(always)]
pub fn square_root(a: f64) -> f64 {
    a.sqrt()
}

#[inline(always)]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

#[inline(always)]
pub fn round_f64(real: f64) -> f64 {
    real.round()
}

#[inline(always)]
pub fn round_f64_i32(real: f64) -> i32 {
    real.round() as i32
}

#[inline(always)]
pub fn round_f64_u32(real: f64) -> u32 {
    real.round() as u32
}

#[inline(always)]
pub fn floor_f64_i32(real: f64) -> i32 {
    real.floor() as i32
}

#[inline(always)]
pub fn floor_f64(real: f64) -> f64 {
    real.floor()
}

#[inline(always)]
pub fn floor_f32(real: f32) -> f32 {
    real.floor()
}

#[inline(always)]
pub fn ceil_f64_i32(real: f64) -> i32 {
    real.ceil() as i32
}

#[inline(always)]
pub fn truncate_f64_i32(real: f64) -> i32 {
    real as i32
}

#[inline(always)]
pub fn truncate_f64_u32(real: f64) -> u32 {
    real as u32
}

#[inline(always)]
pub fn truncate_f64(real: f64) -> f64 {
    truncate_f64_i32(real) as f64
}

#[inline(always)]
pub fn fraction_part_f64(value: f64) -> f64 {
    if value < 0.0 {
        1.0 - (value - floor_f64(value))
    } else {
        value - floor_f64(value)
    }
}

#[inline(always)]
pub fn fraction_part_f32(value: f32) -> f32 {
    if value < 0.0 {
        1.0 - (value - floor_f32(value))
    } else {
        value - floor_f32(value)
    }
}
