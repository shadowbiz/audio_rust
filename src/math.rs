#![allow(dead_code)]

pub struct Vector2 {
    x: f64,
    y: f64,
}

pub struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector2 {
    fn zero() -> Self {
        Vector2 { x: 0.0, y: 0.0 }
    }

    fn one() -> Self {
        Vector2 { x: 1.0, y: 1.0 }
    }

    fn create(new_x: f64, new_y: f64) -> Self {
        Vector2 { x: new_x, y: new_y }
    }
}

pub struct BitScanResult {
    found: bool,
    index: u32,
}

#[inline]
pub fn find_leastsignificant_setbit(value: u32) -> BitScanResult {
    for t in 0..32 {
        if value & (1 << t) != 0 {
            return BitScanResult {
                found: true,
                index: t,
            };
        }
    }
    BitScanResult {
        found: false,
        index: 0,
    }
}

#[inline(always)]
pub fn fmax(a: f32, b: f32) -> f32 {
    if b <= a {
        a
    } else {
        b
    }
}

#[inline(always)]
pub fn fmin(a: f32, b: f32) -> f32 {
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
