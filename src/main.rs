mod renderer;
mod window;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowId};

use crate::renderer::{Color, Renderer};
use crate::window::RawWindowBitmap;

#[derive(Default)]
struct App {
    window: Option<Window>,
    raw_window_bitmap: Option<RawWindowBitmap>,
    renderer: Option<Renderer>,
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
                let size = window.inner_size();
                let _ = bitmap.create_bitmap(size.width as i32, size.height as i32);
                Some(bitmap)
            }
            _ => None,
        };

        self.renderer = Some(Renderer::new());

        self.window = Some(window);
        self.raw_window_bitmap = raw_window_bitmap;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(bitmap) = &mut self.raw_window_bitmap {
                    let _ = bitmap.create_bitmap(size.width as i32, size.height as i32);
                    self.renderer.as_mut().unwrap().resize(bitmap);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(bitmap) = &mut self.raw_window_bitmap {
                    self.renderer.as_mut().unwrap().render_scene(bitmap);
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
