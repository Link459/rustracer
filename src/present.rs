use std::num::NonZeroU32;
use std::rc::Rc;

use anyhow::Result;

use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::utils;
use crate::vec3::Vec3;

#[derive(Debug)]
pub struct PresentationEvent {
    pub color: Vec3,
    pub x: u32,
    pub y: u32,
}

pub struct Presentation {
    window: Option<Window>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    width: u32,
    height: u32,
    samples: f64,
}

impl Presentation {
    pub fn new(width: u32, height: u32, samples: f64) -> Self {
        Presentation {
            window: None,
            surface: None,
            width,
            height,
            samples,
        }
    }
}

impl ApplicationHandler<PresentationEvent> for Presentation {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height))
                        .with_position(winit::dpi::Position::Logical(LogicalPosition::new(
                            600.0, 600.0,
                        )))
                        .with_title("rustracer")
                        .with_resizable(false),
                )
                .unwrap(),
        );

        let ctx = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&ctx, window.clone()).unwrap();
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: PresentationEvent) {
        let mut r = event.color.x;
        let mut g = event.color.y;
        let mut b = event.color.z;

        let scale = 1.0 / self.samples;
        r = f64::sqrt(scale * r);
        g = f64::sqrt(scale * g);
        b = f64::sqrt(scale * b);
        let r = (256.0 * r.clamp(0.0, 0.999)) as u32;
        let g = (256.0 * g.clamp(0.0, 0.999)) as u32;
        let b = (256.0 * b.clamp(0.0, 0.999)) as u32;

        let surface = self.surface.as_mut().unwrap();
        let (width, height) = { (self.width, self.height) };
        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        let mut buffer = surface.buffer_mut().unwrap();

        let color = b | (g << 8) | (r << 16);
        let index = utils::linear_plane_index(buffer.len(), self.width, event.y, event.x) - 1; // - (event.y * self.image.width + event.x);
        buffer[index] = color;
        buffer.present().unwrap();
    }
}

pub fn create_present_loop() -> Result<EventLoop<PresentationEvent>> {
    let event_loop = EventLoop::<PresentationEvent>::with_user_event().build()?;

    event_loop.set_control_flow(ControlFlow::Poll);
    return Ok(event_loop);
}
