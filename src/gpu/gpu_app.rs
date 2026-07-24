use crate::gpu::{instance::Instance, swapchain::Swapchain};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalPosition,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::HasDisplayHandle,
    window::Window,
};

pub struct GpuRaytracer {
    window: Option<Window>,
    width: u32,
    height: u32,
    instance: Instance,
    swapchain: Option<Swapchain>,
}

impl GpuRaytracer {
    pub fn new(width: u32, height: u32, event_loop: &EventLoop<()>) -> Self {
        GpuRaytracer {
            window: None,
            width,
            height,
            instance: Instance::new(event_loop.display_handle().unwrap().as_raw()).expect(""),
            swapchain: None,
        }
    }
}

impl ApplicationHandler for GpuRaytracer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height))
                    .with_position(winit::dpi::Position::Logical(LogicalPosition::new(
                        600.0, 600.0,
                    )))
                    .with_title("rustracer")
                    .with_resizable(false),
            )
            .unwrap();

        if let Some(swapchain) = &self.swapchain {
            swapchain.destroy(&self.instance);
        }

        self.swapchain =
            Some(Swapchain::new(&self.instance, &window).expect("Failed to create Swapchain"));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed => {}
            WindowEvent::CloseRequested => {
                if let Some(swapchain) = &self.swapchain {
                    swapchain.destroy(&self.instance);
                }
                self.instance.destroy();
                event_loop.exit();

                //std::process::exit(0);
            }
            WindowEvent::RedrawRequested => {
                // Drawing loop

                let Some(swapchain) = &self.swapchain else {
                    return;
                };

                //swapchain.acquire(semaphore);
                //swapchain.present(present_queue, image).unwrap();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

pub fn create_present_loop() -> anyhow::Result<EventLoop<()>> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);
    return Ok(event_loop);
}
