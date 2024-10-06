use std::num::NonZeroU32;
use std::rc::Rc;

use anyhow::Result;
/*use glfw::{fail_on_errors, Action, Context, Key};

pub fn present() -> Result<()> {
    let mut glfw = glfw::init(fail_on_errors!())?;
    let (mut window, events) = glfw
        .create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    window.set_key_polling(true);

    // Loop until the user closes the window
    while !window.should_close() {
        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
    }
    Ok(())
}*/

use image::Rgb;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::image::Image;

#[derive(Default)]
struct Presentation {
    window: Option<Window>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    image: image::ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl Presentation {
    fn new(image: Image) -> Self {
        Presentation {
            image: image.into_image_buffer(),
            ..Default::default()
        }
    }
}

impl ApplicationHandler for Presentation {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(
                    winit::dpi::PhysicalSize::new(self.image.width(), self.image.height()),
                ))
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
                let window = surface.window();
                let (width, height) = { (self.image.width(), self.image.height()) };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                let width = self.image.width() as usize;
                for (x, y, pixel) in self.image.enumerate_pixels() {
                    let red = pixel.0[0] as u32;
                    let green = pixel.0[1] as u32;
                    let blue = pixel.0[2] as u32;

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
