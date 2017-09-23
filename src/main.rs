
#![feature(asm)]

extern crate gdi32;
extern crate image;
extern crate kernel32;
extern crate user32;
extern crate winapi;
extern crate winmm;

use image::GenericImage;

use std::fs::File;
use std::path::Path;

use std::mem;

mod random;
mod tools;
mod render;
mod math;
mod windows;

use windows::*;
use render::*;

struct ScreenPoint {
    x: i32,
    y: i32,
}

struct MouseControls {
    button: [bool; 3],
    point: ScreenPoint,
}

struct KeyboardControls {
    key: [bool; 256],
}

pub struct Application {
    is_running: bool,
    pos1: f64,
    pos2: f64,
    delta_time: f64,
    mouse: MouseControls,
    mouse_start: ScreenPoint,
    keyboard: KeyboardControls,
    data: image::DynamicImage,
}

fn main() {
    use winapi::{GWLP_USERDATA, MSG, TIMERR_NOERROR};
    use user32::SetWindowLongPtrW;
    use kernel32::QueryPerformanceFrequency;
    use winmm::timeBeginPeriod;

    let mut global_perf_count_frequency: i64 = 60;

    unsafe {
        if QueryPerformanceFrequency(&mut global_perf_count_frequency) == 0 {
            global_perf_count_frequency = 60;
        }
    };

    let desired_scheduler_ms: u32 = 1;
    let sleep_is_granular: bool =
        unsafe { (timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR) };

    let img = image::open(&Path::new("../../images/logo.bmp")).unwrap();


    let window = unsafe { create_window(String::from("ShadeClass"), String::from("Shade")) };
    if window != std::ptr::null_mut() {
        let target_seconds_per_frame = 1.0 / get_monitor_refresh_rate(&window);

        let mut buffer: WindowBuffer = unsafe { mem::zeroed() };

        let mut app: Box<Application> = Box::new(Application {
            is_running: true,
            pos1: 0.0,
            pos2: 0.0,
            delta_time: 0.0,
            mouse: MouseControls {
                button: [false; 3],
                point: ScreenPoint { x: 0, y: 0 },
            },
            mouse_start: ScreenPoint { x: 0, y: 0 },
            keyboard: KeyboardControls { key: [false; 256] },
            data: img,
        });


        buffer.image = Image::from_color(1900, 1200, Color::hex(0xff0000ff));
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, mem::transmute(&buffer)) };

        let (win_width, win_height) = get_window_dimension(window);
        resize_dib_section(&mut buffer, win_width, win_height);


        let mut msg: MSG = unsafe { mem::uninitialized() };

        let mut last_counter = unsafe { get_wall_clock() };

        while app.is_running {
            let input = process_pending_messages(&mut msg);

            app.process_input(input);
            app.update_and_render(&mut buffer);

            app.delta_time = sleep(
                &mut last_counter,
                target_seconds_per_frame,
                global_perf_count_frequency,
                sleep_is_granular,
            );

            display_buffer_in_window(&buffer, &window);

            print_debug(
                target_seconds_per_frame,
                app.delta_time,
                buffer.image.width,
                buffer.image.height,
            );
        }
    }
}

fn print_debug(target: f64, delta_time: f64, width: i32, height: i32) {
    return;
    println!(
        "Target {0:0.4}ms - frame {1:0.4}ms - {2:0.2} FPS,  width {3}, height {4}",
        target * 1000.0,
        delta_time * 1000.0,
        1.0 / delta_time,
        width,
        height
    );
}

impl Application {
    fn update_and_render(&mut self, buffer: &mut WindowBuffer) {
        self.pos1 += 10.0 * self.delta_time;
        self.pos2 += 50.0 * self.delta_time;

        let image = &mut buffer.image;
        let width = image.width;
        let height = image.height;


        if self.pos1 > width as f64 {
            self.pos1 = 0.0;
        }

        if self.pos2 > width as f64 {
            self.pos2 = 0.0;
        }


        image.fill(Color::hex(0xFF161616));
        image.rect(0, 0, width, 700, Color::hex(0xFF333333));
        image.rect(self.pos1 as i32, 0, 20, 100, Color::hex(0xFFA08563));
        image.rect(self.pos2 as i32, 500, 100, 100, Color::hex(0xFFA08563));

        let img = &self.data;
        for x in 0..img.width() {
            for y in 0..img.height() {
                let rgba = img.get_pixel(x, y);
                let color = Color::rgba(rgba[0] as u8, rgba[1] as u8, rgba[2] as u8, rgba[3] as u8);
                image.pixel_over(x as i32, y as i32, color);
            }
        }

        let mouse = &self.mouse.point;

        if mouse.y <= 700 {
            let mouse_down = self.mouse.button[0];
            if mouse_down {
                let start_x = self.mouse_start.x;
                if mouse.x < start_x {
                    image.rect(mouse.x, 0, start_x - mouse.x, 700, Color::hex(0xFFA08563));
                } else {
                    image.rect(start_x, 0, mouse.x - start_x, 700, Color::hex(0xFFA08563));
                }

                image.line(start_x, 0, start_x, 700, Color::hex(0xFF880000));
            }
            image.line(mouse.x, 0, mouse.x, 700, Color::hex(0xFF880000));
        }
    }

    fn process_input(&mut self, message: windows::Message) {
        match message {
            windows::Message::Quit => self.is_running = false,
            windows::Message::KeyDown(key) => {
                self.keyboard.key[key as usize] = true;
                println!("{} KEY DOWN", key);
            }
            windows::Message::KeyUp(key) => {
                self.keyboard.key[key as usize] = false;
                println!("{} KEY UP", key);
            }
            windows::Message::MouseDown(button, x, y) => {
                self.mouse.button[button.as_usize()] = true;
                self.mouse.point.x = x;
                self.mouse.point.y = y;
                self.mouse_start.x = x;
                self.mouse_start.y = y;
                println!("MOUSE BUTTON {0} DOWN AT {1},{2}", button, x, y)
            }
            windows::Message::MouseUp(button, x, y) => {
                self.mouse.button[button.as_usize()] = false;
                self.mouse.point.x = x;
                self.mouse.point.y = y;
                println!("MOUSE BUTTON {0} UP AT {1},{2}", button, x, y);
            }

            windows::Message::MouseMove(x, y) => {
                self.mouse.point.x = x;
                self.mouse.point.y = y;
                if self.mouse.button[0] || self.mouse.button[1] || self.mouse.button[2] {
                    println!("MOUSE MOVING AT {0},{1}", x, y);
                }
            }
            _ => {}
        }
    }
}
