use std::{ffi::CStr, ops::Deref};

use anyhow::Result;
use ash::{
    extensions::{ext::DebugUtils, khr},
    vk, Entry,
};
use winit::{
    event_loop::EventLoop,
    window::{
        raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle},
        WindowBuilder,
    },
};

use super::{command_pool::CommandPool, device::Device, swapchain::Swapchain};

pub struct Instance {
    pub instance: ash::Instance,
    pub device: Device,
    swapchain: Swapchain,
    command_pool: CommandPool,
}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Instance {
    pub fn new() -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title("rustracer")
            .build(&EventLoop::new().unwrap())?;
        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0",
            )]
        };
        let entry = unsafe { Entry::load()? };
        let layers_names_raw: Vec<*const std::os::raw::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())?.to_vec();
        extension_names.push(DebugUtils::name().as_ptr());

        let appinfo = vk::ApplicationInfo::builder()
            .application_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .application_version(0)
            .engine_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = vk::InstanceCreateFlags::default();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Instance creation error")
        };

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(None);

        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_call_back =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }?;

        let surface_loader = khr::Surface::new(&entry, &instance);
        let device = Device::new(&instance, &entry, surface, &surface_loader)?;
        let command_pool = CommandPool::new(&device, device.queue_index)?;
        let swapchain = Swapchain::new(
            &instance,
            &device,
            &command_pool.get_buffers()[0],
            &surface_loader,
            &surface,
            window,
        )?;

        todo!()
    }
}
