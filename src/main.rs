#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::cmp::{max, min};
use std::error::Error;
use std::ops::{Deref, RangeBounds, RangeInclusive};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use pixels::{Pixels, SurfaceTexture};
use wgpu::hal::DynQueue;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Icon, Window, WindowId};

extern crate nalgebra as na;

#[path = "lib/colorspace.rs"]
mod colorspace;

const DEF_WIDTH: u32 = 256;
const DEF_HEIGHT: u32 = 192;
const BOX_SIZE: i16 = 64;

struct App<'win> {
    window: Option<Arc<Window>>, // see Option<Box<dyn Window>>... 0.30.x changed Window to struct over trait?
    ctx: Option<Arc<Mutex<Pixels<'win>>>>,
    smgr: StateMgr,
    icon: Icon,
}

struct StateMgr {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl<'win> ApplicationHandler for App<'win> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("COLOR ME SURPRISED IT WORKS");
            let win = Arc::new(
                event_loop.create_window(win_attr).unwrap()
            );

            self.window = Some(win.clone());
            self.ctx = {
                let win_size = win.inner_size();
                let tex = SurfaceTexture::new(win_size.width, win_size.height, win.clone());
                Some(Arc::new(Mutex::new(Pixels::new(DEF_WIDTH, DEF_HEIGHT, tex).unwrap())))
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        let smgr = &mut self.smgr;

        match event {
            WindowEvent::Resized(new_size) => {
                if let (mut pix, Some(win)) = (self.ctx.clone().unwrap().lock().unwrap(), self.window.as_ref()) {
                    pix.resize_surface(new_size.width, new_size.height);
                    win.request_redraw();
                    println!("OK: window resize (={}, ={})", new_size.width, new_size.height);
                } else {
                    println!("ERR: ctx/win")
                }


            },
            WindowEvent::CloseRequested => {
                println!("OK: stopping.");
                event_loop.exit();
            },
            // WindowEvent::Focused(focused) => {
            //     println!("OK: ({:?})", focused);
            // },
            WindowEvent::MouseWheel { device_id: _, delta: _, phase: _ } => {
                println!("OK: wheee!");
            },
            WindowEvent::RedrawRequested => { // !!!! STANDARD LOOP !!!!
                if let (mut pix, Some(win)) = (self.ctx.clone().unwrap().lock().unwrap(), self.window.as_ref()) {
                    smgr.update();
                    smgr.draw(pix.frame_mut());
                    win.request_redraw();
                    pix.render();
                }
            },
            _ => {
                println!("clear!");
            }
        }
    }
}

impl<'win> App<'win> {
    fn new(event_loop: &EventLoop<()>) -> Self {
        let icon = load_icon(include_bytes!("../data/allory_1024x1024x32.png"));

        Self {
            window: Default::default(),
            ctx: Default::default(),
            smgr: StateMgr::new(),
            icon,
        }
    }
}

impl StateMgr {
    fn new() -> Self {
        StateMgr { box_x: 30, box_y: 30, velocity_x: 1, velocity_y: 1 }
    }

    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > DEF_WIDTH as i16 {
            self.velocity_x *= -1;
        }

        if self.box_y <= 0 || self.box_y + BOX_SIZE > DEF_HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        let wind = duclamp(self.box_x, -3..=3);
        self.box_x += self.velocity_x;// + wind;
        self.box_y += self.velocity_y;// + wind;
    }

    fn draw(&self, frame: &mut [u8] ) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % DEF_WIDTH as usize) as i16;
            let y = (i / DEF_WIDTH as usize) as i16;

            let is_inbox = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if is_inbox {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
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

fn duclamp(val: i16, bounds: RangeInclusive<i16>) -> i16 {
    let (mi, ma) = bounds.into_inner();
    max(min(val, ma), mi)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app);

}