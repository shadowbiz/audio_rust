#![feature(asm)]

extern crate coresimd;
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate winapi;
extern crate winmm;

use std::mem;

mod math;
mod random;
mod tools;
mod render;
mod windows;

use random::*;
use coresimd::vendor::*;
use windows::*;
use render::*;
use math::*;

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
    delta_time: f64,
    mouse: MouseControls,
    mouse_start: ScreenPoint,
    keyboard: KeyboardControls,
    window_buffer: WindowBuffer,
    sprites: Vec<Sprite>,
}

fn main() {
    use winapi::{GWLP_USERDATA, MSG, TIMERR_NOERROR};
    use user32::SetWindowLongPtrW;
    use kernel32::QueryPerformanceFrequency;
    use winmm::timeBeginPeriod;

    let mut global_perf_count_frequency: i64 = 60;

    let query_performance_frequency =
        unsafe { QueryPerformanceFrequency(&mut global_perf_count_frequency) };
    if query_performance_frequency == 0 {
        global_perf_count_frequency = 60;
    }

    let desired_scheduler_ms: u32 = 1;
    let sleep_is_granular: bool =
        unsafe { (timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR) };

    let window = unsafe { create_window(String::from("ShadeClass"), String::from("Shade")) };
    if window != std::ptr::null_mut() {
        let target_seconds_per_frame = 1.0 / get_monitor_refresh_rate(&window);

        let mut app: Box<Application> = Box::new(Application {
            is_running: true,
            delta_time: 0.0,
            mouse: MouseControls {
                button: [false; 3],
                point: ScreenPoint { x: 0, y: 0 },
            },
            mouse_start: ScreenPoint { x: 0, y: 0 },
            keyboard: KeyboardControls { key: [false; 256] },
            window_buffer: unsafe { mem::zeroed() },
            sprites: Vec::new(),
        });

        app.window_buffer.image = Image::from_color(1900, 1200, Color::from_u32(0xFFFF0000));
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, mem::transmute(&app.window_buffer)) };

        let (win_width, win_height) = get_window_dimension(window);
        resize_dib_section(&mut app.window_buffer, win_width, win_height);

        let mut msg: MSG = unsafe { mem::uninitialized() };
        let mut last_counter = unsafe { get_wall_clock() };

        let rect_width = 100;
        let rect_height = 200;

        for _ in 0..15 {
            let color_start = Color::random();
            let color_end = Color::random();

            if random_bool() {
                app.sprites.push(Sprite {
                    image: Image::from_horisontal_gradient(
                        rect_width,
                        rect_height,
                        color_start,
                        color_end,
                    ),
                    position: Vector2::ORIGIN,
                });
            } else {
                app.sprites.push(Sprite {
                    image: Image::from_vectical_gradient(
                        rect_width,
                        rect_height,
                        color_start,
                        color_end,
                    ),
                    position: Vector2::ORIGIN,
                });
            }
        }

        while app.is_running {
            let app_cycle_count = unsafe { _rdtsc() };

            let input = process_pending_messages(&mut msg);
            let os_input_cycles = unsafe { _rdtsc() } - app_cycle_count;

            let mut app_input_cycles = unsafe { _rdtsc() };
            app.process_input(input);
            app_input_cycles = unsafe { _rdtsc() } - app_input_cycles;

            let render_cycle_count = unsafe { _rdtsc() };
            app.update_and_render();

            app.delta_time = sleep(
                &mut last_counter,
                target_seconds_per_frame,
                global_perf_count_frequency,
                sleep_is_granular,
            );

            display_buffer_in_window(&app.window_buffer, &window);

            let end_cycles_elapsed = unsafe { _rdtsc() };

            let app_cycles = end_cycles_elapsed - app_cycle_count;
            let render_cycles = end_cycles_elapsed - render_cycle_count;

            println!(
                "CYCLES APP {0}, INPUT OS {1}, INPUT APP {2}, RENDER {3}",
                app_cycles, os_input_cycles, app_input_cycles, render_cycles
            );
        }
    }
}

/*
fn print_debug(cycles: u64, target: f64, delta_time: f64, width: i32, height: i32) {
    println!(
        "Cycles {0} - Target {1:0.4}ms - frame {2:0.4}ms - {3:0.2} FPS,  width {4}, height {5}",
        cycles,
        target * 1000.0,
        delta_time * 1000.0,
        1.0 / delta_time,
        width,
        height
    );
}
*/

impl Application {
    fn update_and_render(&mut self) {
        let window_buffer = &mut self.window_buffer.image;
        //let width = window_buffer.width;
        let height = window_buffer.height;

        //Clear BG
        window_buffer.fill(Color::from_u32(0xFF111111));

        let buffers_count = self.sprites.len();
        for i in 0..buffers_count {
            let sprite = &mut self.sprites[i];

            sprite.position = Vector2::new(i as f64 * sprite.image.width as f64 * 1.1, 50.0);
            window_buffer.draw_bitmap(&sprite.position, &sprite.image);
        }

        let mouse = &self.mouse.point;
        let mouse_color = Color::from_u32(0xFFA08563);
        let mouse_fill_color = Color::from_u32(0xFF880000);

        if mouse.y <= 700 {
            let mouse_down = self.mouse.button[0];
            if mouse_down {
                let start_x = self.mouse_start.x;
                if mouse.x < start_x {
                    window_buffer.draw_rect(
                        Vector2 {
                            x: mouse.x as f64,
                            y: 0.0,
                        },
                        Vector2 {
                            x: (start_x - mouse.x) as f64,
                            y: height as f64,
                        },
                        mouse_color,
                    );
                } else {
                    window_buffer.draw_rect(
                        Vector2 {
                            x: start_x as f64,
                            y: 0.0,
                        },
                        Vector2 {
                            x: (mouse.x - start_x) as f64,
                            y: height as f64,
                        },
                        mouse_color,
                    );
                }

                window_buffer.draw_line(&Vector2::new(start_x as f64, 0.0), &Vector2::new(start_x as f64, height as f64), mouse_fill_color);
            }
            window_buffer.draw_line(&Vector2::new(mouse.x as f64, 0.0), &Vector2::new(mouse.x as f64, height as f64), mouse_fill_color);
        }
    }

    fn process_input(&mut self, message: windows::Message) {
        match message {
            windows::Message::Quit => self.is_running = false,
            windows::Message::KeyDown(key) => {
                self.keyboard.key[key as usize] = true;
            }
            windows::Message::KeyUp(key) => {
                self.keyboard.key[key as usize] = false;
            }
            windows::Message::MouseDown(button, x, y) => {
                self.mouse.button[button.as_usize()] = true;
                self.mouse.point.x = x;
                self.mouse.point.y = y;
                self.mouse_start.x = x;
                self.mouse_start.y = y;
            }
            windows::Message::MouseUp(button, x, y) => {
                self.mouse.button[button.as_usize()] = false;
                self.mouse.point.x = x;
                self.mouse.point.y = y;
            }

            windows::Message::MouseMove(x, y) => {
                self.mouse.point.x = x;
                self.mouse.point.y = y;
            }
            _ => {}
        }
    }
}
