#![cfg_attr(stdsimd_strict, deny(warnings))]
#![feature(stdsimd)]
#![allow(dead_code)]

extern crate stdsimd;

use stdsimd::simd::*;

use random::*;
use audio::*;
use tools::*;
use math::*;

pub enum Colors {
    Empty = 0x00000000,
    Amber = 0xFFFFBF00,
    Black = 0xFF000000,
    White = 0xFFFFFFFF,
    DarkGrey = 0xFF222222,
}

#[derive(Clone, Copy)]
pub struct Color {
    value: u32,
}

#[derive(Clone, Copy)]
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
    pub need_update: bool,
    pub layer: LayerID,
    pub children: Vec<Box<Sprite>>,
}

pub enum LayerID {
    Base,
    Background,
    Wave,
    GUI,
    Last,
}

pub struct BitmapHeader {
    file_t_ype: u16,
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
        let (r, g, b, _) = separate_to_rgba(random_u32());
        Color {
            value: pack_to_argb(r, g, b, 255),
        }
    }

    pub fn from_u32(color: u32) -> Self {
        Color { value: color }
    }

    pub fn from_rgba(r: u32, g: u32, b: u32, a: u32) -> Self {
        Color {
            value: pack_to_argb(r, g, b, a),
        }
    }

    pub fn separate(self) -> (u32, u32, u32, u32) {
        return separate_to_rgba(self.value);
    }

    pub fn separate_f32(self) -> (f32, f32, f32, f32) {
        return separate_to_rgba_f32(self.value);
    }

    pub fn set_rgba(&mut self, r: u32, g: u32, b: u32, a: u32) {
        self.value = pack_to_argb(r, g, b, a);
    }

    pub fn set_rgb(&mut self, r: u32, g: u32, b: u32) {
        self.value = pack_to_argb(r, g, b, 255);
    }
}

#[inline(always)]
fn color_set_red(color: Color, value: u32) -> Color {
    let new_value = change_bytes(color.value, 2, value);
    Color::from_u32(new_value)
}

#[inline(always)]
fn color_set_green(color: Color, value: u32) -> Color {
    let new_value = change_bytes(color.value, 1, value);
    Color::from_u32(new_value)
}

#[inline(always)]
fn color_set_blue(color: Color, value: u32) -> Color {
    let new_value = change_bytes(color.value, 0, value);
    Color::from_u32(new_value)
}

#[inline(always)]
fn color_set_alpha(color: Color, value: u32) -> Color {
    let new_value = change_bytes(color.value, 3, value);

    // let shift = value << 24;
    // let mask = 0xFF << shift;
    // let new_value = (!mask & color.value) | shift;

    Color::from_u32(new_value)
}

#[inline(always)]
fn color_set_alpha_f64(color: Color, alpha: f64) -> Color {
    let a = (alpha * 255.0) as u32;
    color_set_alpha(color, a)
}

#[inline(always)]
fn pack_to_argb(r: u32, g: u32, b: u32, a: u32) -> u32 {
    (a << 24) | (r << 16) | (g << 8) | (b << 0)
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

#[inline(always)]
fn separate_to_rgba_f32(value: u32) -> (f32, f32, f32, f32) {
    let one255 = 1.0 / 255.0;
    (
        one255 * (((value >> 16) & 0xFF) as f32),
        one255 * (((value >> 8) & 0xFF) as f32),
        one255 * (((value >> 0) & 0xFF) as f32),
        one255 * (((value >> 24) & 0xFF) as f32),
    )
}

fn create_vectical_gradient(
    width: i32,
    height: i32,
    color_start: Color,
    color_end: Color,
    data: &mut Box<[Color]>,
) {
    let (r_s, g_s, b_s, _) = color_start.separate();
    let (r_e, g_e, b_e, _) = color_end.separate();

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
        let color = Color::from_rgba(
            round_f64_u32(r_new),
            round_f64_u32(g_new),
            round_f64_u32(b_new),
            255,
        );
        for y in 0..height {
            let offset = get_index(x, y, width);
            data[offset] = color;
        }

        r_new = r_new + r_step;
        g_new = g_new + g_step;
        b_new = b_new + b_step;

        r_new = min_f64(255.0, max_f64(0.0, r_new));
        g_new = min_f64(255.0, max_f64(0.0, g_new));
        b_new = min_f64(255.0, max_f64(0.0, b_new));
    }
}

fn create_curve(points: &Vec<Vector2>) -> Vec<Vector2> {
    let array_len = points.len();
    let mut pts: Vec<Vector2> = Vec::with_capacity(array_len + 2);
    pts.push(points[0]);
    for i in 0..array_len {
        pts.push(points[i]);
    }
    pts.push(points[array_len - 1]);

    let tension = 0.1;
    let num_of_segments = 16;

    let mut result: Vec<Vector2> = Vec::with_capacity(array_len + 2);

    let stop_index = pts.len() - 2;
    for i in 1..stop_index {
        for t in 0..num_of_segments {
            // calc tension vectors
            let t1 = (pts[i + 1] - pts[i - 1]) * tension;
            let t2 = (pts[i + 2] - pts[i]) * tension;

            // calc step
            let step = t as f64 / num_of_segments as f64;

            // calc cardinals
            let c1 = 2.0 * f64::powf(step, 3.0) - 3.0 * f64::powf(step, 2.0) + 1.0;
            let c2 = -(2.0 * f64::powf(step, 3.0)) + 3.0 * f64::powf(step, 2.0);
            let c3 = f64::powf(step, 3.0) - 2.0 * f64::powf(step, 2.0) + step;
            let c4 = f64::powf(step, 3.0) - f64::powf(step, 2.0);

            // calc x and y cords with common control vectors
            let x = c1 * pts[i].x + c2 * pts[i + 1].x + c3 * t1.x + c4 * t2.x;
            let y = c1 * pts[i].y + c2 * pts[i + 1].y + c3 * t1.y + c4 * t2.y;

            //store points in array
            result.push(Vector2::new(x, y));
        }
    }

    result
}

fn create_horisontal_gradient(
    width: i32,
    height: i32,
    color_start: Color,
    color_end: Color,
    data: &mut Box<[Color]>,
) {
    let row_len = width;

    let start_y = 0;
    let end_y = height;

    let (r_s, g_s, b_s, a_s) = color_start.separate();
    let (r_e, g_e, b_e, a_e) = color_end.separate();

    let r_range = (r_e as f64 - r_s as f64) as f64;
    let g_range = (g_e as f64 - g_s as f64) as f64;
    let b_range = (b_e as f64 - b_s as f64) as f64;
    let a_range = (a_e as f64 - a_s as f64) as f64;

    let r_step = r_range / (height as f64);
    let g_step = g_range / (height as f64);
    let b_step = b_range / (height as f64);
    let a_step = a_range / (height as f64);

    let mut r_new = r_s as f64;
    let mut g_new = g_s as f64;
    let mut b_new = b_s as f64;
    let mut a_new = a_s as f64;

    for y in start_y..end_y {
        let color = Color::from_rgba(
            round_f64_u32(r_new),
            round_f64_u32(g_new),
            round_f64_u32(b_new),
            round_f64_u32(a_new),
        );

        unsafe {
            let offset = get_index(0, y, width) as isize;
            let destination = data.as_mut_ptr().offset(offset) as *mut u32;
            fast_set32(destination, color.value, row_len as usize);
        }

        r_new += r_step;
        g_new += g_step;
        b_new += b_step;
        a_new += a_step;

        r_new = clamp_f64(0.0, r_new, 255.0);
        g_new = clamp_f64(0.0, g_new, 255.0);
        b_new = clamp_f64(0.0, b_new, 255.0);
        a_new = clamp_f64(0.0, a_new, 255.0);
    }
}

fn plot_aa(position: Vector2, width: i32, color: Color, data: &mut [Color]) {
    let (x, y) = (position.x, position.y);
    let floor_x = floor_f64_i32(x);
    let ceil_x = ceil_f64_i32(x);
    for rounded_x in floor_x..ceil_x {
        let floor_y = floor_f64_i32(y);
        let ceil_y = ceil_f64_i32(y);
        for rounded_y in floor_y..ceil_y {
            let percent_x = 1.0 - abs_f64(x - rounded_x as f64);
            let percent_y = 1.0 - abs_f64(y - rounded_y as f64);
            let percent = percent_x * percent_y;

            let pixel_index = get_index(rounded_x, rounded_y, width);
            let new_color = color_set_alpha_f64(color, percent);
            data[pixel_index] = new_color;
        }
    }
}

fn plot_point(x: i32, y: i32, width: i32, color: Color, data: &mut [Color]) {
    let pixel_index = get_index(x, y, width);

    if pixel_index >= data.len() {
        return;
    }

    let dest_pixel = &mut data[pixel_index];

    let (s_r, s_g, s_b, s_a) = color.separate_f32();
    let (d_r, d_g, d_b, d_a) = (*dest_pixel).separate_f32();

    //d_a = 1.0 - s_a;

    let source_alpha_wide = f32x4::new(s_a, s_a, s_a, s_a);
    let mut source_wide = f32x4::new(s_r, s_g, s_b, s_a);
    source_wide = source_wide * source_alpha_wide;

    let mut dest_alpha_wide = f32x4::new(d_a, d_a, d_a, d_a);
    let mut dest_wide = f32x4::new(d_r, d_g, d_b, d_a);
    dest_wide = dest_wide * dest_alpha_wide;

    let wide1 = f32x4::new(1.0, 1.0, 1.0, 1.0);
    dest_alpha_wide = wide1 - source_alpha_wide;

    //let out_a = s_a + d_a;
    //let out_wide = source_wide * source_alpha_wide + dest_wide * dest_alpha_wide;
    let out_wide = source_wide + dest_wide * dest_alpha_wide;
    let mut out_alpha_wide = source_alpha_wide + dest_alpha_wide;

    let wide255 = f32x4::new(255.0, 255.0, 255.0, 255.0);

    let result = wide255 * out_wide;
    out_alpha_wide = out_alpha_wide * wide255;

    *dest_pixel = unsafe {
        Color::from_rgba(
            result.extract_unchecked(0) as u32,
            result.extract_unchecked(1) as u32,
            result.extract_unchecked(2) as u32,
            out_alpha_wide.extract_unchecked(3) as u32,
        )
    };
}

fn plot_line(start: &Vector2, end: &Vector2, width: i32, color: Color, data: &mut [Color]) {
    //let new_color = color_set_alpha_f64(color, 0.5);

    //let (r, g, b, mut a) = separate_to_rgba(color.value);
    //a /= 2;
    //let new_color = Color::from_rgba(r, g, b, a);

    //let new_color = color_set_alpha(color, 127);
    //let new_color = Color::from_u32(0xFF220000);
    //plot_line_dda(start, end, width, new_color, data);
    //plot_line_fast(start, end, width, new_color, data);
    //plot_line_dda(start, end, width, color, data);

    plot_line_fast(start, end, width, color, data);
}

fn plot_line_dda(pos0: &Vector2, pos1: &Vector2, width: i32, color: Color, data: &mut [Color]) {
    let x1 = truncate_f64_i32(pos0.x);
    let y1 = truncate_f64_i32(pos0.y);
    let x2 = truncate_f64_i32(pos1.x);
    let y2 = truncate_f64_i32(pos1.y);

    let x2_x1 = x2 - x1;
    let y2_y1 = y2 - y1;

    let length = if y2_y1.abs() > x2_x1.abs() {
        y2_y1.abs()
    } else {
        x2_x1.abs()
    };

    let xincrement = x2_x1 as f64 / length as f64;
    let yincrement = y2_y1 as f64 / length as f64;

    let mut x = 0.5 + x1 as f64;
    let mut y = 0.5 + y1 as f64;

    let mut i = 1;
    while i <= length {
        //plot_point
        //let pixel_index = get_index(x as i32, y as i32, width);
        //data[pixel_index].value = color.value;

        plot_point(x as i32, y as i32, width, color, data);

        x = x + xincrement;
        y = y + yincrement;
        i += 1;
    }
}

fn plot_line_fast(pos0: &Vector2, pos1: &Vector2, width: i32, color: Color, data: &mut [Color]) {
    use std::mem;

    let x1 = truncate_f64_i32(pos0.x);
    let y1 = truncate_f64_i32(pos0.y);
    let x2 = truncate_f64_i32(pos1.x);
    let y2 = truncate_f64_i32(pos1.y);

    let mut y_longer = false;
    let mut short_len = y2 - y1;
    let mut long_len = x2 - x1;

    if short_len.abs() > long_len.abs() {
        mem::swap(&mut short_len, &mut long_len);
        y_longer = true;
    }

    let increment_val: i32;
    let end_val = long_len;

    if long_len < 0 {
        increment_val = -1;
        long_len = -long_len;
    } else {
        increment_val = 1;
    }

    let dec_inc = if long_len == 0 {
        0
    } else {
        (short_len << 16) / long_len
    };

    let mut j = 0;

    let mut i = 0;
    while i != end_val {
        let x: i32;
        let y: i32;
        if y_longer == true {
            x = x1 + (j >> 16);
            y = y1 + i;
        } else {
            x = x1 + i;
            y = y1 + (j >> 16);
        }

        plot_point(x, y, width, color, data);
        //let pixel_index = get_index(x, y, width);
        //data[pixel_index].value = color.value;
        j += dec_inc;
        i += increment_val;
    }
}

fn plot_line_fast_f64(
    pos0: &Vector2,
    pos1: &Vector2,
    width: i32,
    color: Color,
    data: &mut [Color],
) {
    use std::mem;

    let x1 = pos0.x;
    let y1 = pos0.y;
    let x2 = pos1.x;
    let y2 = pos1.y;

    let mut y_longer = false;
    let mut short_len = (y2 - y1) as i32;
    let mut long_len = (x2 - x1) as i32;

    if abs_i32(short_len) > abs_i32(long_len) {
        mem::swap(&mut short_len, &mut long_len);
        y_longer = true;
    }

    let increment_val = if long_len < 0 { -1 } else { 1 };

    let mult_diff = if long_len == 0 {
        short_len as f64
    } else {
        (short_len as f64) / (long_len as f64)
    };

    if y_longer == true {
        let mut i = 0;
        while i != long_len {
            plot_aa(
                Vector2::new(x1 + mult_diff * i as f64, y1 + i as f64),
                width,
                color,
                data,
            );
            i += increment_val;
        }
    } else {
        let mut i = 0;
        while i != long_len {
            plot_aa(
                Vector2::new(x1 + i as f64, y1 + mult_diff * i as f64),
                width,
                color,
                data,
            );
            i += increment_val;
        }
    }
}

fn plot_square(pos0: &Vector2, pos1: &Vector2, width: i32, color: Color, data: &mut [Color]) {
    let (x0, y0) = (pos0.x, pos0.y);
    let (x1, y1) = (pos1.x, pos1.y);

    plot_line(
        &Vector2::new(x0, y0),
        &Vector2::new(x0, y1),
        width,
        color,
        data,
    );
    plot_line(
        &Vector2::new(x0, y1),
        &Vector2::new(x1, y1),
        width,
        color,
        data,
    );
    plot_line(
        &Vector2::new(x1, y1),
        &Vector2::new(x1, y0),
        width,
        color,
        data,
    );
    plot_line(
        &Vector2::new(x1, y0),
        &Vector2::new(x0, y0),
        width,
        color,
        data,
    );
}

impl Sprite {
    pub fn new(position: Vector2, width: i32, height: i32, layer: LayerID) -> Sprite {
        Sprite {
            image: Image::from_color(width, height, Color::from_u32(Colors::White as u32)),
            position: position,
            layer: layer,
            need_update: true,
            children: Vec::new(),
        }
    }
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image::from_color(width, height, Color::from_u32(Colors::Empty as u32))
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

    pub fn waveform(
        width: i32,
        height: i32,
        wave: &Waveform,
        start: u32,
        range: u32,
        color: Color,
    ) -> Image {
        let center_y = height / 2;

        let color_empty = Color::from_u32(Colors::Empty as u32);
        let mut data = vec![color_empty; width as usize * height as usize].into_boxed_slice();

        let half_height = height as f64 / 2.0;

        let end = (start + range) as usize;

        //let len = wave.points.len();
        let mut last_position = Vector2::new(0.0, center_y as f64 + wave.points[0].y * half_height);

        let mut index = 0;
        for i in start as usize..end {
            let position = Vector2::new(
                index as f64 / range as f64 * width as f64,
                center_y as f64 + wave.points[i].y * half_height,
            );
            plot_line(&last_position, &position, width, color, &mut data);
            last_position = position;
            index += 1;
        }
        Image::from_data(width, height, data)
    }

    pub fn from_horisontal_gradient(
        width: i32,
        height: i32,
        color_start: Color,
        color_end: Color,
    ) -> Image {
        let color = Color::from_u32(Colors::White as u32);
        let mut data = vec![color; width as usize * height as usize].into_boxed_slice();
        create_horisontal_gradient(width, height, color_start, color_end, &mut data);
        Image::from_data(width, height, data)
    }

    pub fn from_vectical_gradient(
        width: i32,
        height: i32,
        color_start: Color,
        color_end: Color,
    ) -> Image {
        let color = Color::from_u32(Colors::White as u32);
        let mut data = vec![color; width as usize * height as usize].into_boxed_slice();
        create_vectical_gradient(width, height, color_start, color_end, &mut data);
        Image::from_data(width, height, data)
    }

    pub fn fill(&mut self, color: Color) {
        let data = &mut self.color_data[..];
        let destination = data.as_mut_ptr() as *mut u32;
        fast_set32(destination, color.value, data.len());
    }

    pub fn clear(&mut self) {
        self.fill(Color::from_u32(Colors::Empty as u32));
    }

    /*
    pub fn pixel_overlay(&mut self, position: &Vector2, color: Color) {
        let x = position.x as i32;
        let y = position.y as i32;

        let width = self.width;
        //let height = self.height;

        let data = &mut self.color_data;

        let pixel_index = (y * width + x) as usize;
        let bg_pixel = &mut data[pixel_index].value;
        let fg_pixel = color.value;

        let (bg_r, bg_g, bg_b, _) = separate_to_rgba(*bg_pixel);
        let (fg_r, fg_g, fg_b, fg_a) = separate_to_rgba(fg_pixel);

        let bg_a = 255 - fg_a;

        let bg_a_w = u32x4::new(bg_a, bg_a, bg_a, bg_a);
        let fg_a_w = u32x4::new(fg_a, fg_a, fg_a, fg_a);

        let bg_w = u32x4::new(bg_r, bg_g, bg_b, bg_a);
        let fg_w = u32x4::new(fg_r, fg_g, fg_b, fg_a);

        let res = ((fg_w * fg_a_w + bg_w * bg_a_w) >> 8) as u32x4;

        *bg_pixel = unsafe {
            pack_to_argb(
                res.extract_unchecked(0),
                res.extract_unchecked(1),
                res.extract_unchecked(2),
                255,
            )
        };
    }
*/

    pub fn draw_line(&mut self, start: &Vector2, end: &Vector2, color: Color) {
        let width = self.width;
        let data = &mut self.color_data;
        plot_line(start, end, width, color, data);
    }

    pub fn draw_bitmap(&mut self, sprite: &Sprite) {
        let pos_x = round_f64_i32(sprite.position.x);
        let pos_y = round_f64_i32(sprite.position.y);

        let image = &sprite.image;

        let source_width = image.width;
        let source_height = image.height;

        let self_width = self.width;
        let self_height = self.height;

        let start_x = max_i32(0, pos_x);
        let end_x = min_i32(self_width, pos_x + source_width);

        let start_y = max_i32(0, pos_y);
        let end_y = min_i32(self_height, pos_y + source_height);

        let dest_data = &mut self.color_data;
        let source_image_data = &image.color_data;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let source_offset = get_index(x - pos_x, y - pos_y, source_width);
                let color = source_image_data[source_offset];
                let fg_a = (color.value >> 24) & 0xFF;

                if fg_a == 0 {
                    continue;
                } else if fg_a == 255 {
                    let dest_offset = get_index(x, y, self_width);
                    dest_data[dest_offset] = color;
                } else {
                    plot_point(x, y, self_width, color, dest_data);
                }
            }
        }
    }

    pub fn draw_rect(&mut self, position: &Vector2, size: &Vector2, color: Color) {
        let self_width = self.width;
        let self_height = self.height;

        let x = round_f64_i32(position.x);
        let y = round_f64_i32(position.y);

        let width = round_f64_i32(size.x); // - x;
        let height = round_f64_i32(size.y); // - y;

        let start_y = max_i32(0, min_i32(self_width, y));
        let start_x = max_i32(0, min_i32(self_width, x));

        let row_len = max_i32(start_x, min_i32(self_width, x + width)) - start_x;

        let end_x = start_x + row_len;
        let end_y = max_i32(start_y, min_i32(self_height, y + height));

        let data = &mut self.color_data;

        let alpha = (color.value >> 24) & 0xFF;
        if alpha > 0 {
            if alpha == 255 {
                for y in start_y..end_y {
                    let offset = get_index(x, y, self_width) as isize;
                    let destination = unsafe { data.as_mut_ptr().offset(offset) as *mut u32 };
                    fast_set32(destination, color.value, row_len as usize);
                }
            } else {
                let self_width = self.width;
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        plot_point(x, y, self_width, color, data);
                    }
                }
            }
        }
    }
}
