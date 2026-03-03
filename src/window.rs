use std::os::raw::c_void;
use std::ptr::null_mut;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, GetDC, HBITMAP, HDC, HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
};

pub struct RawWindowBitmap {
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
    pub fn create_bitmap(&mut self, width: i32, height: i32) -> Result<(), &str> {
        unsafe {
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
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

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        unsafe {
            let data = self
                .pixel_data
                .unwrap()
                .add(y * 4 * self.width as usize + x * 4) as *mut u32;
            *data = color;
        }
    }

    pub fn present(&self) {
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
