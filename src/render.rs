#![allow(unused_imports)]
#![allow(dead_code)]



use std::error::Error;



use std::fs::File;
use std::path::Path;




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
    pub fn hex(color: u32) -> Self {
        Color { value: color }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color {
            value: hex_to_argb(r as u32, g as u32, b as u32, 0xFF000000),
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            value: hex_to_argb(r as u32, g as u32, b as u32, a as u32),
        }
    }
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image::from_color(width, height, Color::rgb(0, 0, 0))
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

    pub fn set(&mut self, color: Color) {
        let data = &mut self.color_data[..];
        let destination = data.as_mut_ptr() as *mut u32;
        unsafe {
            fast_set32(destination, color.value, data.len());
        }
    }

    pub fn clean(&mut self) {
        self.set(Color::rgb(0, 0, 0));
    }

    pub fn fill(&mut self, color: Color) {
        self.set(color);
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
            //let bg_a = (*bg_pixel >> 24) & 0xFF;
            let bg_r = (*bg_pixel >> 16) & 0xFF;
            let bg_g = (*bg_pixel >> 8) & 0xFF;
            let bg_b = (*bg_pixel >> 0) & 0xFF;

            let fg_r = (fg_pixel >> 16) & 0xFF;
            let fg_g = (fg_pixel >> 8) & 0xFF;
            let fg_b = (fg_pixel >> 0) & 0xFF;

            let bg_a = 255 - fg_a;
            let res_r = (fg_r * fg_a + bg_r * bg_a) >> 8;
            let res_g = (fg_g * fg_a + bg_g * bg_a) >> 8;
            let res_b = (fg_b * fg_a + bg_b * bg_a) >> 8;

            *bg_pixel = hex_to_argb(res_r, res_g, res_b, 0xFF00000);
        }
    }

    pub fn line(&mut self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color) {
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

    pub fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color) {
        let self_width = self.width;
        let self_height = self.height;

        let start_y = imax(0, imin(self_width - 1, y));
        let end_y = imax(start_y, imin(self_height, y + height));

        let start_x = imax(0, imin(self_width - 1, x));
        let len = imax(start_x, imin(self_width, x + width)) - start_x;

        let alpha = (color.value >> 24) & 0xFF;
        if alpha > 0 {
            if alpha == 255 {
                let data = &mut self.color_data[..];
                for y in start_y..end_y {
                    unsafe {
                        let offset = (y * self_width + start_x) as isize;
                        let destination = data.as_mut_ptr().offset(offset) as *mut u32;
                        fast_set32(destination, color.value, len as usize);
                    }
                }
            } else {
                let end_x = start_x + len;
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        self.pixel_over(x, y, color);
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn hex_to_argb(r: u32, g: u32, b: u32, a: u32) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
