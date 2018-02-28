#![allow(unused_imports)]
#![allow(dead_code)]

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::ops::{Add, Div, Sub};

use random::*;
use tools::*;
use math::*;

#[derive(Clone, Copy)]
pub struct Color {
    value: u32,
}

pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub struct Image {
    pub width: i32,
    pub height: i32,
    pub color_data: Box<[Color]>,
}

pub struct Sprite {
    pub image: Image,
    pub position: Vector2,
}

pub struct BitmapHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    bitmap_offset: u32,
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    size_of_bitmap: u32,
    horz_resolution: i32,
    vert_resolution: i32,
    colors_used: u32,
    colors_important: u32,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

//#[inline(always)]
//fn rgba_from_u32(color: u32) -> (u8, u8, u8, u8) {}



impl Color {
    /// AARRGGBB
    pub fn random() -> Self {
        let (r, g, b, _) = separate_to_rgba(random());
        Color {
            value: pack_to_argb(r, g, b, 255),
        }
    }

    pub fn from_u32(color: u32) -> Self {
        Color { value: color }
    }

    pub fn from_rgb(r: u32, g: u32, b: u32) -> Self {
        Color {
            value: pack_to_argb(r, g, b, 255),
        }
    }

    pub fn from_rgba(r: u32, g: u32, b: u32, a: u32) -> Self {
        Color {
            value: pack_to_argb(r, g, b, a),
        }
    }

    pub fn separate_to_rgba(self) -> (u32, u32, u32, u32) {
        return separate_to_rgba(self.value);
    }

    pub fn set_rgba(&mut self, r: u32, g: u32, b: u32, a: u32) {
        self.value = pack_to_argb(r, g, b, a);
    }

    pub fn set_rgb(&mut self, r: u32, g: u32, b: u32) {
        self.value = pack_to_argb(r, g, b, 255);
    }
}

#[inline(always)]
fn pack_to_argb(r: u32, g: u32, b: u32, a: u32) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline(always)]
fn separate_to_rgba(value: u32) -> (u32, u32, u32, u32) {
    (
        (value >> 16) & 0xFF,
        (value >> 8) & 0xFF,
        (value >> 0) & 0xFF,
        (value >> 24) & 0xFF,
    )
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image::from_color(width, height, Color::from_u32(0xFF000000))
    }

    pub fn from_color(width: i32, height: i32, color: Color) -> Image {
        Image::from_data(
            width,
            height,
            vec![color; width as usize * height as usize].into_boxed_slice(),
        )
    }

    pub fn from_data(width: i32, height: i32, data: Box<[Color]>) -> Image {
        Image {
            width: width,
            height: height,
            color_data: data,
        }
    }

    pub fn from_horisontal_gradient(
        width: i32,
        height: i32,
        color_start: Color,
        color_end: Color,
    ) -> Image {
        let color = Color::from_u32(0xFFFFFFFF);
        let mut data = vec![color; width as usize * height as usize].into_boxed_slice();

        let row_len = width;

        let start_y = 0;
        let end_y = height;

        let (r_s, g_s, b_s, _) = color_start.separate_to_rgba();
        let (r_e, g_e, b_e, _) = color_end.separate_to_rgba();

        let r_range = (r_e as f64 - r_s as f64) as f64;
        let g_range = (g_e as f64 - g_s as f64) as f64;
        let b_range = (b_e as f64 - b_s as f64) as f64;

        let r_step = r_range / (height as f64);
        let g_step = g_range / (height as f64);
        let b_step = b_range / (height as f64);

        let mut r_new = r_s as f64;
        let mut g_new = g_s as f64;
        let mut b_new = b_s as f64;

        for y in start_y..end_y {
            let color = Color::from_rgb(
                round_f64_u32(r_new),
                round_f64_u32(g_new),
                round_f64_u32(b_new),
            );

            unsafe {
                let offset = get_index(0, y, width) as isize;
                let destination = data.as_mut_ptr().offset(offset) as *mut u32;
                fast_set32(destination, color.value, row_len as usize);
            }

            r_new = r_new + r_step;
            g_new = g_new + g_step;
            b_new = b_new + b_step;

            r_new = fmin(255.0, fmax(0.0, r_new));
            g_new = fmin(255.0, fmax(0.0, g_new));
            b_new = fmin(255.0, fmax(0.0, b_new));
        }

        Image::from_data(width, height, data)
    }

    pub fn from_vectical_gradient(
        width: i32,
        height: i32,
        color_start: Color,
        color_end: Color,
    ) -> Image {
        let color = Color::from_u32(0xFFFFFFFF);
        let mut data = vec![color; width as usize * height as usize].into_boxed_slice();

        let (r_s, g_s, b_s, _) = color_start.separate_to_rgba();
        let (r_e, g_e, b_e, _) = color_end.separate_to_rgba();

        let r_range = (r_e as f64 - r_s as f64) as f64;
        let g_range = (g_e as f64 - g_s as f64) as f64;
        let b_range = (b_e as f64 - b_s as f64) as f64;

        let r_step = r_range / (width as f64);
        let g_step = g_range / (width as f64);
        let b_step = b_range / (width as f64);

        let mut r_new = r_s as f64;
        let mut g_new = g_s as f64;
        let mut b_new = b_s as f64;

        for x in 0..width {
            let color = Color::from_rgb(
                round_f64_u32(r_new),
                round_f64_u32(g_new),
                round_f64_u32(b_new),
            );
            for y in 0..height {
                let offset = get_index(x, y, width);
                data[offset] = color;
            }

            r_new = r_new + r_step;
            g_new = g_new + g_step;
            b_new = b_new + b_step;

            r_new = fmin(255.0, fmax(0.0, r_new));
            g_new = fmin(255.0, fmax(0.0, g_new));
            b_new = fmin(255.0, fmax(0.0, b_new));
        }

        Image::from_data(width, height, data)
    }

    pub fn fill(&mut self, color: Color) {
        let data = &mut self.color_data[..];
        let destination = data.as_mut_ptr() as *mut u32;
        unsafe {
            fast_set32(destination, color.value, data.len());
        }
    }

    pub fn clear(&mut self) {
        self.fill(Color::from_u32(0xFF00000));
    }

    #[inline]
    pub fn pixel_over(&mut self, x: i32, y: i32, color: Color) {
        let width = self.width;
        let height = self.height;
        let data = &mut self.color_data[..];

        if x <= 0 || y <= 0 || x >= width || y >= height {
            return;
        }

        let fg_pixel = color.value;
        let fg_a = (fg_pixel >> 24) & 0xFF;

        if fg_a == 0 {
            return;
        }

        let bg_pixel = &mut data[y as usize * width as usize + x as usize].value;

        if fg_a == 255 {
            *bg_pixel = fg_pixel;
        } else {
            let (bg_r, bg_g, bg_b, _) = separate_to_rgba(*bg_pixel);
            let (fg_r, fg_g, fg_b, _) = separate_to_rgba(fg_pixel);

            let bg_a = 255 - fg_a;
            let res_r = (fg_r * fg_a + bg_r * bg_a) >> 8;
            let res_g = (fg_g * fg_a + bg_g * bg_a) >> 8;
            let res_b = (fg_b * fg_a + bg_b * bg_a) >> 8;

            *bg_pixel = pack_to_argb(res_r, res_g, res_b, 255);
        }
    }

    pub fn draw_line(&mut self, start: &Vector2, end: &Vector2, color: Color) {
        
        let argx1 = round_f64_i32(start.x);
        let argy1 = round_f64_i32(start.y);
        let argx2 = round_f64_i32(end.x);
        let argy2 = round_f64_i32(end.y);
        
        let mut x = argx1;
        let mut y = argy1;

        let dx = if argx1 > argx2 {
            argx1 - argx2
        } else {
            argx2 - argx1
        };
        let dy = if argy1 > argy2 {
            argy1 - argy2
        } else {
            argy2 - argy1
        };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else { -dy } / 2;
        let mut err_tolerance;

        loop {
            self.pixel_over(x, y, color);

            if x == argx2 && y == argy2 {
                break;
            };

            err_tolerance = 2 * err;

            if err_tolerance > -dx {
                err -= dy;
                x += sx;
            }
            if err_tolerance < dy {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_bitmap(&mut self, position: &Vector2, image: &Image) {
        let self_width = self.width;
        let self_height = self.height;

        let x = round_f64_i32(position.x);
        let y = round_f64_i32(position.y);

        let source_width = image.width;
        let source_height = image.height;

        let start_x = imax(0, imin(self_width - 1, x));
        let end_x = imax(start_x, imin(self_width, x + source_width));

        let start_y = imax(0, imin(self_height - 1, y));
        let end_y = imax(start_y, imin(self_height, y + source_height));

        let mut source_x = 0;
        let mut source_y = 0;

        let dest_data = &mut self.color_data[..];

        for y in start_y..end_y {
            for x in start_x..end_x {
                let source_offset = get_index(source_x, source_y, source_width);
                let source = image.color_data[source_offset].value;
                let source_a = (source >> 24) & 0xFF;
                if source_a > 0 {
                    let dest_offset = get_index(x, y, self_width);
                    let destination = &mut dest_data[dest_offset].value;
                    *destination = source;
                }
                source_x = source_x + 1;
            }
            source_x = 0;
            source_y = source_y + 1;
        }
    }

    pub fn draw_rect(&mut self, v_min: Vector2, v_max: Vector2, color: Color) {
        let self_width = self.width;
        let self_height = self.height;

        let x = round_f64_i32(v_min.x);
        let y = round_f64_i32(v_min.y);

        let width = round_f64_i32(v_max.x); // - x;
        let height = round_f64_i32(v_max.y); // - y;

        let start_y = imax(0, imin(self_width - 1, y));
        let end_y = imax(start_y, imin(self_height, y + height));

        let start_x = imax(0, imin(self_width - 1, x));
        let row_len = imax(start_x, imin(self_width, x + width)) - start_x;

        let alpha = (color.value >> 24) & 0xFF;
        if alpha > 0 {
            if alpha == 255 {
                let data = &mut self.color_data[..];
                for y in start_y..end_y {
                    unsafe {
                        let offset = get_index(x, y, self_width) as isize;
                        let destination = data.as_mut_ptr().offset(offset) as *mut u32;
                        fast_set32(destination, color.value, row_len as usize);
                    }
                }
            } else {
                let end_x = start_x + row_len;
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        self.pixel_over(x, y, color);
                    }
                }
            }
        }
    }
}
