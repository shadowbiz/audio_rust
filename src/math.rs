#![allow(dead_code)]

const PI: f64 = 3.14159265358979323846;

pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Rect2i
{
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl Vector2 {
    const ORIGIN: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    const ONE: Vector2 = Vector2 { x: 1.0, y: 1.0 };

    pub fn new(new_x: f64, new_y: f64) -> Self {
        Vector2 { x: new_x, y: new_y }
    }

}

impl Vector3 {
    const ORIGIN: Vector2 = Vector2 { x: 0.0, y: 0.0 };
}

#[inline(always)]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

#[inline(always)]
pub fn fmax(a: f64, b: f64) -> f64 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn fmin(a: f64, b: f64) -> f64 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn umax(a: u32, b: u32) -> u32 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn umin(a: u32, b: u32) -> u32 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn imax(a: i32, b: i32) -> i32 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn imin(a: i32, b: i32) -> i32 {
    if b >= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn round_f32_to_i32(value: f32) -> i32 {
    value.round() as i32
}

#[inline(always)]
pub fn round_f64_i32(real32: f64) -> i32 {
    real32.round() as i32
}

#[inline(always)]
pub fn round_f32_to_u32(value: f32) -> u32 {
    value.round() as u32
}

#[inline(always)]
pub fn round_f64_u32(real32 : f64) -> u32 {
    real32.round() as u32
}

#[inline(always)]
pub fn floor_f64_i32(real32 : f64) -> i32 {
    real32.floor() as i32
}

#[inline(always)]
pub fn ceil_f64_i32(real32 : f64) -> i32 {
    real32.ceil() as i32
}

#[inline(always)] 
pub fn truncate_f64_i32(real32 : f64) -> i32 {
    real32 as i32
}


