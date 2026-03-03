use std::os::raw::c_void;
use std::ptr::null_mut;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, GetDC, HBITMAP, HDC, HGDIOBJ, PAINTSTRUCT, ReleaseDC, SRCCOPY,
    SelectObject,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle, Win32WindowHandle};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
    raw_window_bitmap: Option<RawWindowBitmap>,
}

struct RawWindowBitmap {
    hwnd: HWND,
    window_dc: HDC,
    memory_dc: HDC,
    bitmap_info: Option<BITMAPINFO>,
    bitmap: Option<windows::core::Result<HBITMAP>>,
    pixel_data: Option<*mut c_void>,
    width: i32,
    height: i32,
}

impl RawWindowBitmap {
    fn create_bitmap(&mut self, width: i32, height: i32) -> Result<(), &str> {
        unsafe {
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: height, // invert?
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut pixels: *mut core::ffi::c_void = null_mut();

            let hbitmap = CreateDIBSection(
                Some(self.memory_dc),
                &bmi,
                DIB_RGB_COLORS,
                &mut pixels,
                None,
                0,
            );

            if let Ok(hbitresult) = hbitmap {
                SelectObject(self.memory_dc, HGDIOBJ(hbitresult.0));

                self.bitmap_info = Some(bmi);
                self.bitmap = Some(hbitmap);
                self.pixel_data = Some(pixels);
                self.width = width;
                self.height = height;

                return Ok(());
            }

            Err("Could not create DIB section")
        }
    }

    fn set_pixel(&mut self, x: isize, y: isize, color: u32) {
        unsafe {
            let data = self.pixel_data.unwrap().offset(x * y) as *mut u32;
            *data = color;
        }
    }

    fn present(&self) {
        unsafe {
            let _ = BitBlt(
                self.window_dc,
                0,
                0,
                self.width,
                self.height,
                Some(self.memory_dc),
                0,
                0,
                SRCCOPY,
            );
        }
    }
}

impl From<isize> for RawWindowBitmap {
    fn from(value: isize) -> Self {
        unsafe {
            let hwnd = windows::Win32::Foundation::HWND(value as *mut c_void);
            let window_dc = GetDC(Some(hwnd));
            let memory_dc = CreateCompatibleDC(Some(window_dc));

            Self {
                hwnd,
                window_dc,
                memory_dc,
                bitmap_info: None,
                bitmap: None,
                pixel_data: None,
                width: 0,
                height: 0,
            }
        }
    }
}

impl Drop for RawWindowBitmap {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.memory_dc);
            ReleaseDC(Some(self.hwnd), self.window_dc);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let handle = window.window_handle().unwrap().as_raw();
        let raw_window_bitmap: Option<RawWindowBitmap> = match handle {
            RawWindowHandle::Win32(w32_handle) => {
                let mut bitmap: RawWindowBitmap = w32_handle.hwnd.get().into();
                let _ = bitmap.create_bitmap(200, 200);
                Some(bitmap)
            }
            _ => None,
        };

        self.window = Some(window);
        self.raw_window_bitmap = raw_window_bitmap;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(bitmap) = &mut self.raw_window_bitmap {
                    bitmap.set_pixel(100, 100, 0x00FF0000);
                    bitmap.present();
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
