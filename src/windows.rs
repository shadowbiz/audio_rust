//extern crate gdi32;
//extern crate kernel32;
extern crate ole32;
extern crate user32;
extern crate winapi;
//extern crate winmm;

use std::mem;
use std::ptr;
use std::fmt;
use std::thread;
use std::time::Duration;

use render::*;
use tools::*;

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn as_usize(&self) -> usize {
        let result = match *self {
            MouseButton::Left => 0 as usize,
            MouseButton::Right => 1 as usize,
            MouseButton::Middle => 2 as usize,
        };
        result
    }
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            MouseButton::Left => "Left",
            MouseButton::Right => "Right",
            MouseButton::Middle => "Middle",
        };
        write!(f, "{}", printable)
    }
}

pub enum Message {
    Quit,
    Silence,
    KeyDown(u32),
    KeyUp(u32),
    MouseUp(MouseButton, i32, i32),
    MouseDown(MouseButton, i32, i32),
    MouseMove(i32, i32),
}

pub struct WindowBuffer {
    pub info: winapi::BITMAPINFO,
    pub image: Image,
    pub resized: bool,
}

pub unsafe extern "system" fn window_proc(
    window: winapi::HWND,
    msg: winapi::UINT,
    w_param: winapi::WPARAM,
    l_param: winapi::LPARAM,
) -> winapi::LRESULT {
    use winapi::{GWLP_USERDATA, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_PAINT, WM_SIZE};
    use user32::{DefWindowProcW, GetWindowLongPtrW, PostQuitMessage, SendMessageW};
    let user_data = GetWindowLongPtrW(window, GWLP_USERDATA);

    if user_data == 0 {
        return DefWindowProcW(window, msg, w_param, l_param);
    }

    let buffer: &mut WindowBuffer = mem::transmute(user_data);

    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
        }

        WM_CREATE => {
            println!("WM_CREATE");
            create_menus(window);
        }

        WM_SIZE => {
            let width = (l_param as i32) & 0xffff;
            let height = ((l_param as i32) >> 16) & 0xffff;
            println!("WM_SIZE");
            resize_dib_section(buffer, width, height);
        }

        WM_PAINT => {
            //let mut paint: PAINTSTRUCT = mem::zeroed();
            //let device_context = BeginPaint(window, &mut paint);
            //win32_display_buffer_in_window(&mut app.buffer, &window);
            //EndPaint(window, &paint);
        }

        WM_COMMAND => {
            let low = (w_param as i32) & 0xffff;
            match low {
                2 => {
                    SendMessageW(window, WM_CLOSE, 0, 0);
                }
                _ => {}
            }
        }

        _ => (),
    }

    DefWindowProcW(window, msg, w_param, l_param)
}

fn create_menus(window: winapi::HWND) {
    use winapi::{MF_SEPARATOR, MF_STRING};
    use user32::{AppendMenuW, CreateMenu, SetMenu};
    unsafe {
        //let menu_bar = CreateMenu();
        let menu = CreateMenu();

        let new_str = to_wstring("&New");
        let quit_str = to_wstring("&Quit");

        AppendMenuW(menu, MF_STRING, 0, new_str.as_ptr());
        //AppendMenuW(hMenu, MF_STRING, 1, to_wstring("&Open").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null_mut());
        AppendMenuW(menu, MF_STRING, 2, quit_str.as_ptr());

        //AppendMenuW(hMenubar, 3, hMenu, to_wstring("&File").as_ptr());

        //SetMenu(result, menu_bar);
        SetMenu(window, menu);
    }
}

unsafe fn hide_console_window() {
    use kernel32::GetConsoleWindow;
    use user32::ShowWindow;
    use winapi::SW_HIDE;

    let window = GetConsoleWindow();

    if window != ptr::null_mut() {
        ShowWindow(window, SW_HIDE);
    }
}

pub fn resize_dib_section(buffer: &mut WindowBuffer, width: i32, height: i32) {
    println!("RESIZE {0}, {1}", width, height);
    use winapi::{BITMAPINFOHEADER, BI_RGB};

    buffer.resized = true;

    buffer.image.width = width;
    buffer.image.height = height;

    buffer.info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    buffer.info.bmiHeader.biWidth = buffer.image.width;
    buffer.info.bmiHeader.biHeight = -buffer.image.height;
    buffer.info.bmiHeader.biPlanes = 1;
    buffer.info.bmiHeader.biBitCount = 32;
    buffer.info.bmiHeader.biCompression = BI_RGB;
}

pub fn get_window_dimension(window: winapi::HWND) -> (i32, i32) {
    use winapi::RECT;
    use user32::GetClientRect;

    unsafe {
        let mut client_rect: RECT = mem::uninitialized();
        GetClientRect(window, &mut client_rect);
        (client_rect.right as i32, client_rect.bottom as i32)
    }
}

pub fn sleep(
    last_counter: &mut i64,
    target_seconds_per_frame: f64,
    global_perf_count_frequency: i64,
    sleep_is_granular: bool,
) -> f64 {
    let work_counter = unsafe { get_wall_clock() };
    let work_seconds_elapsed =
        get_seconds_elapsed(global_perf_count_frequency, *last_counter, work_counter);

    let mut seconds_elapsed_for_frame = work_seconds_elapsed;

    if seconds_elapsed_for_frame < target_seconds_per_frame {
        if sleep_is_granular {
            let sleep_ms: u64 =
                ((target_seconds_per_frame - seconds_elapsed_for_frame) * 1000.0) as u64;
            if sleep_ms > 0 {
                thread::sleep(Duration::from_millis(sleep_ms));
            }
        }

        while seconds_elapsed_for_frame < target_seconds_per_frame {
            seconds_elapsed_for_frame = unsafe {
                get_seconds_elapsed(global_perf_count_frequency, *last_counter, get_wall_clock())
            };
        }
    }

    let end_counter = unsafe { get_wall_clock() };
    let delta_time = get_seconds_elapsed(global_perf_count_frequency, *last_counter, end_counter);

    *last_counter = end_counter;

    delta_time
}

pub fn get_monitor_refresh_rate(window: &winapi::HWND) -> f64 {
    use winapi::VREFRESH;
    use user32::{GetDC, ReleaseDC};
    use gdi32::GetDeviceCaps;

    let refresh_dc = unsafe { GetDC(*window) };
    let refresh_rate = unsafe { GetDeviceCaps(refresh_dc, VREFRESH) };
    unsafe {
        ReleaseDC(*window, refresh_dc);
    };

    if refresh_rate > 1 {
        refresh_rate as f64
    } else {
        60.0
    }
}

pub unsafe fn enable_com() {
    use winapi::{COINIT_DISABLE_OLE1DDE, COINIT_APARTMENTTHREADED};

    use windows::ole32::CoInitializeEx;
    CoInitializeEx(
        ptr::null_mut(),
        COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
    );
}

pub unsafe fn create_window(class_name: String, window_name: String) -> winapi::HWND {
    use winapi::{CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HBRUSH, HINSTANCE, IDI_APPLICATION,
                 WNDCLASSEXW, WS_OVERLAPPEDWINDOW, WS_SYSMENU, WS_VISIBLE};
    use user32::{CreateWindowExW, RegisterClassExW};

    use kernel32::GetModuleHandleW;
    hide_console_window();

    let class_name = to_wstring(&class_name);
    let window_name = to_wstring(&window_name);

    let hmod = GetModuleHandleW(ptr::null_mut());
    if hmod.is_null() {
        return ptr::null_mut() as winapi::HWND;
    }

    let wnd_class = WNDCLASSEXW {
        lpszClassName: class_name.as_ptr(),
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hmod,
        hIcon: user32::LoadIconW(0 as HINSTANCE, IDI_APPLICATION),
        hCursor: user32::LoadCursorW(0 as HINSTANCE, IDI_APPLICATION),
        hbrBackground: 16 as HBRUSH,
        lpszMenuName: 0 as *const u16,
        hIconSm: ptr::null_mut(),
    };

    if RegisterClassExW(&wnd_class) == 0 {
        return ptr::null_mut() as winapi::HWND;
    }

    let result = CreateWindowExW(
        0,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_SYSMENU,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    );

    create_menus(result);

    enable_com();
    result
}

#[inline]
pub fn process_pending_messages(message: &mut winapi::MSG) -> Message {
    use winapi::{HWND, PM_REMOVE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
                 WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_QUIT, WM_RBUTTONDOWN,
                 WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP};
    use user32::{DispatchMessageW, PeekMessageW, TranslateMessage};

    unsafe {
        while PeekMessageW(message, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
            match message.message {
                WM_DESTROY | WM_QUIT => return Message::Quit,
                WM_SYSKEYDOWN => return Message::KeyDown(message.wParam as u32),
                WM_SYSKEYUP => return Message::KeyUp(message.wParam as u32),
                WM_KEYDOWN => return Message::KeyDown(message.wParam as u32),
                WM_KEYUP => return Message::KeyUp(message.wParam as u32),
                WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_LBUTTONDOWN | WM_RBUTTONDOWN
                | WM_MBUTTONDOWN => {
                    return parse_mouse_click(message.message, message.lParam);
                }
                WM_MOUSEMOVE => {
                    let (x, y) = parse_mouse_position(message.lParam);
                    return Message::MouseMove(x, y);
                }
                _ => {
                    TranslateMessage(message);
                    DispatchMessageW(message);
                }
            }
        }
    }

    Message::Silence
}

#[inline]
fn parse_mouse_position(l_param: winapi::LPARAM) -> (i32, i32) {
    use winapi::{GET_X_LPARAM, GET_Y_LPARAM};

    let x = GET_X_LPARAM(l_param) as i32;
    let y = GET_Y_LPARAM(l_param) as i32;
    (x, y)
}

fn parse_mouse_click(msg: u32, l_param: winapi::LPARAM) -> Message {
    use winapi::{WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN,
                 WM_RBUTTONUP};

    let (x, y) = parse_mouse_position(l_param);

    match msg {
        WM_LBUTTONUP => return Message::MouseUp(MouseButton::Left, x, y),
        WM_RBUTTONUP => return Message::MouseUp(MouseButton::Right, x, y),
        WM_MBUTTONUP => return Message::MouseUp(MouseButton::Middle, x, y),
        WM_LBUTTONDOWN => return Message::MouseDown(MouseButton::Left, x, y),
        WM_RBUTTONDOWN => return Message::MouseDown(MouseButton::Right, x, y),
        WM_MBUTTONDOWN => return Message::MouseDown(MouseButton::Middle, x, y),
        _ => return Message::Silence,
    };
}

pub fn display_buffer_in_window(buffer: &WindowBuffer, window: &winapi::HWND) {
    use gdi32::StretchDIBits;
    use winapi::{DIB_RGB_COLORS, SRCCOPY};
    use user32::{GetDC, ReleaseDC};

    let image = &buffer.image;

    unsafe {
        let device_context = GetDC(*window);

        StretchDIBits(
            device_context,
            0,
            0,
            image.width,
            image.height,
            0,
            0,
            image.width,
            image.height,
            mem::transmute(image.color_data.as_ptr()),
            &buffer.info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );

        ReleaseDC(*window, device_context);
    }
}

#[inline]
pub unsafe fn get_wall_clock() -> i64 {
    use kernel32::QueryPerformanceCounter;
    use winapi::LARGE_INTEGER;
    let mut result: LARGE_INTEGER = 0;
    if QueryPerformanceCounter(&mut result) != 0 {
        result
    } else {
        0
    }
}

#[inline]
pub fn get_seconds_elapsed(global_perf_count_frequency: i64, start: i64, end: i64) -> f64 {
    let result = (end - start) as f64 / global_perf_count_frequency as f64;
    return result;
}
