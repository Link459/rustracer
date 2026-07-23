use std::{ffi::CStr, ops::Deref};

use anyhow::Result;
use ash::{ext::debug_utils, khr, vk, Entry};
use winit::{
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, HasWindowHandle},
};

use super::{command_pool::CommandPool, swapchain::Swapchain};

pub struct Instance {
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub pdevice: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub queue_index: u32,
    swapchain: Swapchain,
    surface_loader: khr::surface::Instance,
    //command_pool: CommandPool,
}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    user_data: *mut std::ffi::c_void,
) -> u32 {
    return 0;
}

impl Instance {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
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
            ash_window::enumerate_required_extensions(window.raw_display_handle()?)?.to_vec();
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());

        let appinfo = vk::ApplicationInfo::default()
            .application_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .application_version(0)
            .engine_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = vk::InstanceCreateFlags::default();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Instance creation error")
        };

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle()?,
                window.raw_window_handle()?,
                None,
            )
        }?;

        let surface_loader = khr::surface::Instance::new(&entry, &instance);
        let (device, pdevice, graphics_queue, queue_family_index) =
            Self::init_device(&instance, &entry, surface, &surface_loader)
                .expect("Failed to create Device");

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
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
            .pfn_user_callback(Some(debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };

        let swapchain = Swapchain::new(
            &instance,
            &device,
            &pdevice,
            //&command_pool.get_buffers()[0],
            &surface_loader,
            &surface,
            window,
        )?;

        //let command_pool = CommandPool::new(&true_instance, queue_family_index)?;
        return Ok(Self {
            instance,
            device,
            pdevice,
            graphics_queue,
            queue_index: queue_family_index,
            swapchain,
            surface_loader,
            //command_pool,
        });
    }

    pub fn init_device(
        instance: &ash::Instance,
        entry: &ash::Entry,
        surface: vk::SurfaceKHR,
        surface_loader: &khr::surface::Instance,
    ) -> Result<(ash::Device, vk::PhysicalDevice, vk::Queue, u32)> {
        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };
        let (pdevice, queue_family_index) = pdevices
            .iter()
            .find_map(|pdevice| unsafe {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .expect("Couldn't find suitable device.");

        let queue_family_index = queue_family_index as u32;
        let device_extension_names_raw = [
            khr::swapchain::NAME.as_ptr(),
            /*khr::ray_tracing_pipeline::NAME.as_ptr(),
            khr::acceleration_structure::NAME.as_ptr(),
            khr::deferred_host_operations::NAME.as_ptr(),*/
        ];
        let mut features_12 =
            vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut features_12);
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .push_next(&mut features);
        //.enabled_features(&features);

        let device: ash::Device =
            unsafe { instance.create_device(pdevice, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok((device, pdevice, graphics_queue, queue_family_index))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader
                .destroy_surface(self.swapchain.surface, None);
            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.swapchain, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
