use std::{ffi::CStr, ops::Deref};

use anyhow::Result;
use ash::{ext::debug_utils, khr, vk, Entry};

pub struct Instance {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub queue_family_index: u32,
    pub pipeline_layout: vk::PipelineLayout,

    #[cfg(debug_assertions)]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Deref for Instance {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

unsafe extern "system" fn debug_callback(
    _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    _data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::ffi::c_void,
) -> u32 {
    return 0;
}

impl Instance {
    pub fn new(display_handle: winit::raw_window_handle::RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { Entry::load()? };
        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());

        let appinfo = vk::ApplicationInfo::default()
            .application_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .application_version(0)
            .engine_name(&CStr::from_bytes_with_nul(b"rustracer\0")?)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        let create_flags = vk::InstanceCreateFlags::default();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Instance creation error")
        };

        let (device, pdevice, graphics_queue, queue_family_index) = Self::init_device(&instance)?;

        let debug_messenger = if cfg!(debug_assertions) {
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
            Some(unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? })
        } else {
            None
        };

        let push_constant_range = vk::PushConstantRange::default()
            .size(128)
            .offset(0)
            .stage_flags(vk::ShaderStageFlags::ALL);
        let push_ranges = [push_constant_range];
        let layout_create_info =
            vk::PipelineLayoutCreateInfo::default().push_constant_ranges(&push_ranges);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_create_info, None)? };

        return Ok(Self {
            entry,
            instance,
            device,
            physical_device: pdevice,
            graphics_queue,
            queue_family_index,
            pipeline_layout,

            #[cfg(debug_assertions)]
            debug_messenger,
        });
    }

    fn init_device(
        instance: &ash::Instance,
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
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                        /*&& surface_loader
                        .get_physical_device_surface_support(
                            *pdevice,
                            index as u32,
                            surface,
                        )
                        .unwrap();*/
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
        let mut features_13 = vk::PhysicalDeviceVulkan13Features::default().synchronization2(true);
        let mut features_12 = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .descriptor_indexing(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_update_unused_while_pending(true)
            .descriptor_binding_partially_bound(true).runtime_descriptor_array(true);
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut features_13)
            .push_next(&mut features_12);
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

    pub fn destroy(&mut self) {
        unsafe {
            if cfg!(debug_assertions) {
                if let Some(messenger) = self.debug_messenger {
                    let debug_utils_loader =
                        debug_utils::Instance::new(&self.entry, &self.instance);
                    debug_utils_loader.destroy_debug_utils_messenger(messenger, None);
                }
            }
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
