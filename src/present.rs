use std::num::NonZeroU32;
use std::rc::Rc;

use anyhow::Result;

use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::vec3::Vec3;
use crate::{utils, Float};

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
    samples: Float,
}

impl Presentation {
    pub fn new(width: u32, height: u32, samples: Float) -> Self {
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
        /*let builder = egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(self.width as f32, self.height as f32))
            .with_title("rustracer")
            .with_resizable(false);
        let window = Rc::new(egui_winit::create_window(&self.ctx, event_loop, &builder).unwrap());*/

        let ctx = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&ctx, window.clone()).unwrap();

        surface
            .resize(
                NonZeroU32::new(self.width).unwrap(),
                NonZeroU32::new(self.height).unwrap(),
            )
            .unwrap();

        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Destroyed => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();

                //std::process::exit(0);
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().unwrap();

                let buffer = surface.buffer_mut().unwrap();

                buffer.present().unwrap();
            }

            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let surface = self.surface.as_mut().unwrap();

        let buffer = surface.buffer_mut().unwrap();

        buffer.present().unwrap();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: PresentationEvent) {
        let r = Float::sqrt(event.color.x);
        let g = Float::sqrt(event.color.y);
        let b = Float::sqrt(event.color.z);

        let r = (256.0 * r.clamp(0.0, 0.999)) as u32;
        let g = (256.0 * g.clamp(0.0, 0.999)) as u32;
        let b = (256.0 * b.clamp(0.0, 0.999)) as u32;

        let surface = self.surface.as_mut().unwrap();

        let mut buffer = surface.buffer_mut().unwrap();

        let color = b | (g << 8) | (r << 16);
        let area = buffer.len();

        let x = event.x;
        let index = utils::linear_plane_index(area, self.width, event.y, self.width - x);
        //let index = utils::linear_plane_index(area, self.width, event.y, x) - 1;
        buffer[index] = color;
    }
}

pub fn create_present_loop() -> Result<EventLoop<PresentationEvent>> {
    let event_loop = EventLoop::<PresentationEvent>::with_user_event().build()?;

    event_loop.set_control_flow(ControlFlow::Poll);
    return Ok(event_loop);
}
