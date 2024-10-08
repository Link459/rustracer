use std::num::NonZeroU32;
use std::rc::Rc;

use anyhow::Result;

use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::image::Image;

#[derive(Default)]
pub struct Presentation {
    window: Option<Window>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    image: Image,
}

impl Presentation {
    pub fn new(image: Image) -> Self {
        Presentation {
            image,
            ..Default::default()
        }
    }
}

impl ApplicationHandler for Presentation {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(
                            self.image.width,
                            self.image.height,
                        ))
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
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_mut().unwrap();
                let (width, height) = { (self.image.width, self.image.height) };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                let width = self.image.width as usize;
                for (x, y, pixel) in self.image.iter_pixels() {
                    let red = pixel[0] as u32;
                    let green = pixel[1] as u32;
                    let blue = pixel[2] as u32;

                    let color = blue | (green << 8) | (red << 16);
                    buffer[y as usize * width + x as usize] = color;
                }

                buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

pub fn present(image: Image) -> Result<()> {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Presentation::new(image);
    event_loop.run_app(&mut app)?;
    Ok(())
}
