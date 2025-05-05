use std::error::Error;
use wgpu::hal::DynQueue;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Icon, Window, WindowId};

struct App {
    window: Option<Window>, // Option<Box<dyn Window>> had breaking change?
    icon: Icon,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        match event {
            WindowEvent::Moved(pos) => {
                println!("window moved (+{}, +{})", pos.x, pos.y);
            },
            WindowEvent::CloseRequested => {
                println!("Stopping");
                event_loop.exit();
            },
            WindowEvent::Focused(focused) => {
                println!("::: ({:?})", focused);
            },
            WindowEvent::MouseWheel { device_id: _, delta: _, phase: _ } => {
                println!("wheeled");
            },
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                window.request_redraw();
            },
            _ => {}
        }
    }
}

// Plucked from https://github.com/rust-windowing/winit/blob/master/examples/application.rs verbatim
fn load_icon(bytes: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap()
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let icon = load_icon(include_bytes!("data/icon.icns"));


    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app);
}