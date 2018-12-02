#![feature(stdsimd)]
#![feature(asm)]

extern crate stdsimd;

extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate winapi;
extern crate winmm;

use std::mem;

mod math;
mod audio;
mod random;
mod tools;
mod render;
mod windows;

use audio::*;
//use coresimd::vendor::*;
use windows::*;
use render::*;
use math::*;
//use random::*;

struct ScreenPoint {
    x: i32,
    y: i32,
}

struct MouseControls {
    button: [bool; 3],
    point: ScreenPoint,
}

struct KeyboardControls {
    key: [bool; 512],
}

pub struct Application {
    is_running: bool,
    delta_time: f64,
    mouse: MouseControls,
    mouse_start: ScreenPoint,
    keyboard: KeyboardControls,
    window_buffer: WindowBuffer,
    background: Sprite,
    sprites: Vec<Box<Sprite>>,
    wave: Waveform,
    position: u32,
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
            keyboard: KeyboardControls { key: [false; 512] },
            window_buffer: unsafe { mem::zeroed() },
            background: unsafe { mem::zeroed() },
            sprites: Vec::new(),
            wave: unsafe { mem::zeroed() },
            position: 0,
        });

        app.window_buffer = WindowBuffer {
            image: Image::from_color(1900, 1200, Color::from_u32(0xFFFF0000)),
            resized: true,
            info: unsafe { mem::zeroed() },
        };

        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, mem::transmute(&app.window_buffer)) };

        let (win_width, win_height) = get_window_dimension(window);
        resize_dib_section(&mut app.window_buffer, win_width, win_height);

        let mut msg: MSG = unsafe { mem::uninitialized() };
        let mut last_counter = unsafe { get_wall_clock() };

        app.background = Sprite {
            image: (Image::from_color(
                win_width,
                win_height,
                Color::from_u32(Colors::White as u32),
            )),
            position: Vector2::ORIGIN,
            layer: LayerID::Base,
            need_update: false,
            children: Vec::new(),
        };

        let bg = Box::new(Sprite {
            image: (Image::from_color(
                win_width,
                win_height - 100,
                Color::from_u32(Colors::DarkGrey as u32),
            )),
            position: Vector2::new(0.0, 50.0),
            layer: LayerID::Background,
            need_update: true,
            children: Vec::new(),
        });

        let waveform = Waveform::noise(20000, 44100.0);
        //let waveform = Waveform::osc(400.0, 4000, 44100.0);

        let wave_sprite = Box::new(Sprite {
            image: (Image::from_color(
                win_width,
                win_height - 100,
                Color::from_u32(Colors::Amber as u32),
            )),
            position: Vector2::new(0.0, 50.0),
            layer: LayerID::Wave,
            need_update: true,
            children: Vec::new(),
        });

        app.sprites.push(bg);
        app.sprites.push(wave_sprite);

        app.wave = waveform;

        while app.is_running {
            //let app_cycle_count = unsafe { _rdtsc() };

            let input = process_pending_messages(&mut msg);
            //let os_input_cycles = unsafe { _rdtsc() } - app_cycle_count;

            //let mut app_input_cycles = unsafe { _rdtsc() };
            app.process_input(input);
            //app_input_cycles = unsafe { _rdtsc() } - app_input_cycles;

            // let render_cycle_count = unsafe { _rdtsc() };
            app.update_and_render();

            app.delta_time = sleep(
                &mut last_counter,
                target_seconds_per_frame,
                global_perf_count_frequency,
                sleep_is_granular,
            );

            display_buffer_in_window(&app.window_buffer, &window);

            //let end_cycles_elapsed = unsafe { _rdtsc() };

            //let app_cycles = end_cycles_elapsed - app_cycle_count;
            //let render_cycles = end_cycles_elapsed - render_cycle_count;

            // println!(
            //     "CYCLES APP {0}, INPUT OS {1}, INPUT APP {2}, RENDER {3}",
            //     app_cycles, os_input_cycles, app_input_cycles, render_cycles
            // );
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
        let window_buffer = &mut self.window_buffer;
        let width = window_buffer.image.width;
        let height = window_buffer.image.height;

        window_buffer.image.clear();

        let sprites = &mut self.sprites;

        let buffer_length = 500;

        let buffers_count = sprites.len();
        if window_buffer.resized == true {
            let bg = Image::from_color(width, height, Color::from_u32(Colors::Black as u32));
            self.background.image = bg;
            window_buffer.resized = false;
            for i in 0..buffers_count {
                sprites[i].need_update = true;
            }
        }

        for i in 0..buffers_count {
            if sprites[i].need_update {
                if sprites[i].position.x < width as f64 && sprites[i].position.y < height as f64 {
                    match sprites[i].layer {
                        LayerID::Background => {
                            let mut bg = Image::from_color(
                                width,
                                height - 100,
                                Color::from_u32(Colors::DarkGrey as u32),
                            );
                            sprites[i].image = bg;
                            sprites[i].need_update = true;
                        }
                        LayerID::Wave => {
                            let wave_image = Image::waveform(
                                width,
                                height - 100,
                                &self.wave,
                                self.position,
                                buffer_length,
                                Color::from_u32(Colors::Amber as u32),
                            );
                            sprites[i].image = wave_image;
                            sprites[i].need_update = true;
                        }
                        _ => {}
                    }
                    self.background.image.draw_bitmap(&sprites[i]);
                }
            }
        }

        if self.position < self.wave.sample_count as u32 - buffer_length {
            self.position += 1;
        }

        window_buffer.image.draw_bitmap(&self.background);

        let mouse = &self.mouse.point;
        let mouse_fill_color = Color::from_u32(0x55A08563);
        let mouse_line_color = Color::from_u32(0xFF880000);

        if mouse.y <= 700 {
            let mouse_down = self.mouse.button[0];
            if mouse_down {
                let start_x = self.mouse_start.x;
                if mouse.x < start_x {
                    window_buffer.image.draw_rect(
                        &Vector2 {
                            x: mouse.x as f64,
                            y: 50.0,
                        },
                        &Vector2 {
                            x: (start_x - mouse.x) as f64,
                            y: height as f64 - 100.0,
                        },
                        mouse_fill_color,
                    );
                } else {
                    window_buffer.image.draw_rect(
                        &Vector2 {
                            x: start_x as f64,
                            y: 50.0,
                        },
                        &Vector2 {
                            x: (mouse.x - start_x) as f64,
                            y: height as f64 - 100.0,
                        },
                        mouse_fill_color,
                    );
                }
            }
            window_buffer.image.draw_line(
                &Vector2::new(mouse.x as f64, 50.0),
                &Vector2::new(mouse.x as f64, height as f64 - 50.0),
                mouse_line_color,
            );
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
